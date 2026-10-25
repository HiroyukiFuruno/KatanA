"""Verify one pushed change unit against committed inputs and fresh owner checks."""

import hashlib
import os
from pathlib import Path
import stat

from cargo_version_contract import version_matches
from dependency_push_snapshot import DependencyPushSnapshot
from downstream_git_binding import _git, validate_commit_tree_bindings
from downstream_issue_evidence import parse_issue_evidence
from upstream_publication import verify_upstreams

MAX_LOG_BYTES = 64 * 1024 * 1024
LOG_CHUNK_BYTES = 64 * 1024
CRATES_IO_SOURCES = {"registry+https://github.com/rust-lang/crates.io-index", "registry+https://index.crates.io/"}


def _regular_log(root: Path, path: str):
    resolved = (root / path).resolve(strict=True)
    if not resolved.is_relative_to(root.resolve()):
        raise ValueError("verification log leaves the source repository")
    descriptor = os.open(resolved, os.O_RDONLY | getattr(os, "O_NONBLOCK", 0) | getattr(os, "O_NOFOLLOW", 0))
    try:
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode) or not 0 < metadata.st_size <= MAX_LOG_BYTES:
            raise ValueError("verification log is not a bounded nonempty regular file")
        return os.fdopen(descriptor, "rb")
    except Exception:
        os.close(descriptor)
        raise


def _log_digest(root: Path, path: str):
    with _regular_log(root, path) as stream:
        before = os.fstat(stream.fileno())
        if not stat.S_ISREG(before.st_mode) or before.st_size > MAX_LOG_BYTES:
            raise ValueError("verification log is not a bounded regular file")
        digest, total = hashlib.sha256(), 0
        while chunk := stream.read(LOG_CHUNK_BYTES):
            total += len(chunk)
            if total > MAX_LOG_BYTES:
                raise ValueError("verification log exceeds its byte limit")
            digest.update(chunk)
        after = os.fstat(stream.fileno())
        if (before.st_ino, before.st_size, before.st_mtime_ns) != (after.st_ino, after.st_size, after.st_mtime_ns):
            raise ValueError("verification log changed while being read")
    return digest.hexdigest()


def _source_coverage(change, snapshot):
    for label, basename in (("manifests", "Cargo.toml"), ("lockfiles", "Cargo.lock")):
        expected = {path: item.blob for path, item in snapshot.context.files.items() if path.rsplit("/", 1)[-1] == basename}
        declared = {item["path"]: item["blob"] for item in change[label]}
        if declared != expected:
            raise ValueError("downstream evidence does not cover all committed dependency inputs")


def _records(change):
    records = {}
    for record in change["upstream"]:
        key = (record["package"], record["version"])
        if key in records:
            raise ValueError("upstream package/version identity is ambiguous")
        records[key] = record
    return records


def _lock_coverage(snapshot, records):
    for packages in snapshot.lockfiles.values():
        for package in packages:
            if package.source not in CRATES_IO_SOURCES:
                raise ValueError("downstream release contains an unpublished path/git or unsupported registry dependency")
            record = records.get((package.name, package.version))
            if record is None or package.checksum != record["registry_checksum"]:
                raise ValueError("upstream evidence does not match the actual registry lock record")


def _manifest_coverage(snapshot, records):
    requests, groups = [], []
    for declarations in snapshot.manifests.values():
        for entry in declarations:
            declaration = entry.declaration
            if declaration.source.kind != "registry" or declaration.source.locator not in ("", "crates-io"):
                raise ValueError("downstream release contains an external path/git or unsupported registry declaration")
            if declaration.version is None:
                raise ValueError("downstream dependency has no verifiable version requirement")
            start = len(requests)
            requests.extend({"requirement": declaration.version, "version": version}
                            for package, version in records if package == declaration.package)
            groups.append((start, len(requests)))
    matches = version_matches(requests)
    if any(not any(matches[start:end]) for start, end in groups):
        raise ValueError("published upstream versions do not satisfy the committed dependency declarations")


class DownstreamPushAdmission:
    def __init__(self, root, repository, update, baseline, payloads, verified_source):
        self.root, self.repository = root, repository
        self.update, self.payloads, self.verified_source = update, payloads, verified_source
        self.after = DependencyPushSnapshot(root, update.local_oid)
        self.before = DependencyPushSnapshot(root, baseline)

    def _documents(self):
        result = []
        for payload in self.payloads:
            body = payload.get("body")
            if body is not None and not isinstance(body, str):
                raise ValueError("Issue body is not text")
            document = parse_issue_evidence(body or "", self.repository)
            if document is not None:
                result.append(document)
        return result

    def verify(self):
        documents = self._documents()
        delta = self.after.compare(self.before)
        if not any(delta) and not documents:
            return
        if self.verified_source != self.update.local_oid:
            raise ValueError("downstream evidence requires fresh existing-hook success for the exact pushed source")
        if _git(self.root, ["status", "--porcelain", "--untracked-files=no"]).strip():
            raise ValueError("tracked source is dirty after existing-hook verification")
        selected = [(document, change) for document in documents for change in document["changes"]
                    if change["commit"] == self.update.local_oid]
        if len(selected) != 1:
            raise ValueError("downstream push has no unique Issue evidence for its final source")
        document, change = selected[0]
        validate_commit_tree_bindings(document, self.root, self.repository)
        _source_coverage(change, self.after)
        records = _records(change)
        _lock_coverage(self.after, records)
        _manifest_coverage(self.after, records)
        for item in change["verification"]:
            if _log_digest(self.root, item["log_path"]) != item["log_sha256"]:
                raise ValueError("verification log checksum does not match")
        # 作者のcommand/exit_codeは実行証明ではない。fresh owner成功を独立して要求する。
        verify_upstreams(list(records.values()))
