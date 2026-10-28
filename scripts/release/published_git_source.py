"""Verify source provenance directly from immutable Git objects."""

from __future__ import annotations

import hashlib
import importlib.util
import posixpath
import re
import subprocess
import tomllib
from pathlib import Path
from typing import Any

_HELPER_SPEC = importlib.util.spec_from_file_location(
    "_document_fidelity_acceptance_evidence",
    Path(__file__).with_name("check-document-fidelity-acceptance-evidence.py"),
)
if _HELPER_SPEC is None or _HELPER_SPEC.loader is None:
    raise ImportError("cannot load acceptance evidence helper")
_HELPER = importlib.util.module_from_spec(_HELPER_SPEC)
_HELPER_SPEC.loader.exec_module(_HELPER)
CRATES_IO_SOURCE = _HELPER.CRATES_IO_SOURCE
EXCLUDED_PARTS = _HELPER.EXCLUDED_PARTS
RENDERER_PACKAGES = _HELPER.RENDERER_PACKAGES
SOURCE_ROOTS = _HELPER.SOURCE_ROOTS
git_environment = _HELPER.git_environment

FULL_OID = re.compile(r"[0-9a-f]{40}\Z")
SHA256 = re.compile(r"[0-9a-f]{64}\Z")
MAX_SYMLINK_DEPTH = 40
GIT_TIMEOUT_SECONDS = 60
MAX_BLOB_BYTES = 512 * 1024 * 1024


class PublishedGitSourceError(ValueError):
    """Raised when Git objects do not prove the requested published identity."""


def _git(root: Path, *args: str, input: bytes | None = None) -> bytes:
    try:
        result = subprocess.run(
            ["git", "-C", str(root), *args],
            input=input,
            check=False,
            capture_output=True,
            env=git_environment(),
            timeout=GIT_TIMEOUT_SECONDS,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise PublishedGitSourceError(f"cannot read Git objects: {error}") from error
    if result.returncode:
        detail = result.stderr.decode(errors="replace").strip()
        raise PublishedGitSourceError(f"cannot read Git objects: {detail or 'git failed'}")
    return result.stdout


def _commit(root: Path, commit: str) -> str:
    if not isinstance(commit, str) or FULL_OID.fullmatch(commit) is None:
        raise PublishedGitSourceError("commit must be a full lowercase 40-character object ID")
    if _git(root, "cat-file", "-t", commit) != b"commit\n":
        raise PublishedGitSourceError("source object must be a commit")
    resolved = _git(root, "rev-parse", "--verify", f"{commit}^{{commit}}").decode().strip()
    if resolved != commit:
        raise PublishedGitSourceError("commit object ID does not match the supplied full ID")
    return commit


def _tree_entries(root: Path, commit: str) -> dict[bytes, tuple[bytes, bytes, bytes]]:
    raw = _git(root, "ls-tree", "-r", "-z", "--full-tree", commit)
    entries = {}
    for record in raw.split(b"\0"):
        if not record:
            continue
        try:
            metadata, path = record.split(b"\t", 1)
            mode, kind, oid = metadata.split(b" ")
        except ValueError as error:
            raise PublishedGitSourceError("Git tree contains an invalid entry") from error
        entries[path] = (mode, kind, oid)
    return entries


def _source_entries(tree: dict[bytes, tuple[bytes, bytes, bytes]]) -> list[bytes]:
    selected = []
    for path, (mode, kind, _oid) in tree.items():
        try:
            relative = path.decode("utf-8")
        except UnicodeDecodeError as error:
            raise PublishedGitSourceError("source tree path is not UTF-8") from error
        if not any(relative == root or relative.startswith(root + "/") for root in SOURCE_ROOTS):
            continue
        if any(part in EXCLUDED_PARTS for part in Path(relative).parts):
            continue
        if kind != b"blob" or mode not in {b"100644", b"100755", b"120000"}:
            raise PublishedGitSourceError(f"source entry is not a file or symbolic link: {relative}")
        selected.append(path)
    return sorted(selected)


def _parse_blob_batch(raw: bytes, object_ids: list[bytes]) -> dict[bytes, bytes]:
    cursor = 0
    blobs = {}
    for oid in object_ids:
        header_end = raw.find(b"\n", cursor)
        if header_end < 0:
            raise PublishedGitSourceError("truncated Git object header")
        header = raw[cursor:header_end].split()
        cursor = header_end + 1
        if len(header) != 3 or header[0] != oid or header[1] != b"blob":
            raise PublishedGitSourceError("tree entry does not resolve to a blob")
        if not header[2].isdigit():
            raise PublishedGitSourceError("Git blob size is invalid")
        size = int(header[2])
        if size > MAX_BLOB_BYTES:
            raise PublishedGitSourceError("Git blob exceeds the size limit")
        end = cursor + size
        if end >= len(raw) or raw[end:end + 1] != b"\n":
            raise PublishedGitSourceError("truncated Git blob data")
        blobs[oid] = raw[cursor:end]
        cursor = end + 1
    if cursor != len(raw):
        raise PublishedGitSourceError("unexpected trailing Git object data")
    return blobs


def _read_blob_batch(root: Path, object_ids: set[bytes]) -> dict[bytes, bytes]:
    if not object_ids:
        return {}
    ordered = sorted(object_ids)
    request = b"".join(oid + b"\n" for oid in ordered)
    return _parse_blob_batch(_git(root, "cat-file", "--batch", input=request), ordered)


def _normalized_link(current: bytes, content: bytes) -> bytes:
    try:
        link = content.decode("utf-8")
        current_text = current.decode("utf-8")
    except UnicodeDecodeError as error:
        raise PublishedGitSourceError("symbolic link path or target is not UTF-8") from error
    if not link or link.startswith("/") or "\\" in link:
        raise PublishedGitSourceError("symbolic link target is absolute or invalid")
    target = posixpath.normpath(posixpath.join(posixpath.dirname(current_text), link))
    if target in {".", ".."} or target.startswith("../") or target.startswith("/"):
        raise PublishedGitSourceError("symbolic link target escapes the repository")
    return target.encode()


def _blob_for(root: Path, oid: bytes, cache: dict[bytes, bytes]) -> bytes:
    if oid not in cache:
        cache.update(_read_blob_batch(root, {oid}))
    return cache[oid]


def _resolve_source(root: Path, tree: dict[bytes, tuple[bytes, bytes, bytes]], path: bytes, cache: dict[bytes, bytes]) -> bytes:
    current = path
    seen = set()
    for _ in range(MAX_SYMLINK_DEPTH):
        if current in seen:
            raise PublishedGitSourceError(f"symbolic link cycle at {path.decode(errors='replace')}")
        seen.add(current)
        entry = tree.get(current)
        if entry is None:
            raise PublishedGitSourceError(f"symbolic link target is missing: {path.decode(errors='replace')}")
        mode, kind, oid = entry
        if kind != b"blob":
            raise PublishedGitSourceError("symbolic link target is not a blob")
        content = _blob_for(root, oid, cache)
        if mode in {b"100644", b"100755"}:
            return content
        if mode != b"120000":
            raise PublishedGitSourceError("symbolic link target is not a regular blob")
        current = _normalized_link(current, content)
    raise PublishedGitSourceError("symbolic link resolution exceeds maximum depth")


def _read_source_blobs(root: Path, tree: dict[bytes, tuple[bytes, bytes, bytes]], paths: list[bytes]) -> dict[bytes, bytes]:
    cache = _read_blob_batch(root, {tree[path][2] for path in paths})
    return {path: _resolve_source(root, tree, path, cache) for path in paths}


def _source_digest(paths: list[bytes], blobs: dict[bytes, bytes]) -> str:
    digest = hashlib.sha256()
    # 既存producerと同じく、リンク元パスにリンク先の内容を結び付ける。
    for path in paths:
        data = blobs[path]
        digest.update(len(path).to_bytes(8, "big"))
        digest.update(path)
        digest.update(len(data).to_bytes(8, "big"))
        digest.update(data)
    return digest.hexdigest()


def _lock_packages(lock_bytes: bytes) -> dict[str, dict[str, Any]]:
    try:
        lock = tomllib.loads(lock_bytes.decode("utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        raise PublishedGitSourceError("committed Cargo.lock is invalid") from error
    records = [package for package in lock.get("package", []) if isinstance(package, dict) and package.get("name") in RENDERER_PACKAGES]
    if len(records) != len(RENDERER_PACKAGES):
        raise PublishedGitSourceError("Cargo.lock must resolve exactly one KDV, KRR, and KUC package")
    packages = {str(record["name"]): record for record in records}
    if set(packages) != RENDERER_PACKAGES:
        raise PublishedGitSourceError("Cargo.lock must contain exactly KDV, KRR, and KUC packages")
    normalized = {}
    for name, package in packages.items():
        checksum = package.get("checksum")
        version = package.get("version")
        if package.get("source") != CRATES_IO_SOURCE or not isinstance(version, str):
            raise PublishedGitSourceError(f"Cargo.lock {name} must resolve from canonical crates.io")
        if not isinstance(checksum, str) or SHA256.fullmatch(checksum) is None:
            raise PublishedGitSourceError(f"Cargo.lock {name} must declare a canonical checksum")
        normalized[name] = {"version": version, "source": CRATES_IO_SOURCE, "checksum": checksum}
    return normalized


def git_source_identity(root: Path, commit: str) -> dict[str, Any]:
    """Return commit-bound source and dependency identity from Git objects."""
    root = Path(root).resolve()
    commit = _commit(root, commit)
    tree = _tree_entries(root, commit)
    paths = _source_entries(tree)
    blobs = _read_source_blobs(root, tree, paths)
    lock = blobs.get(b"Cargo.lock")
    if lock is None:
        raise PublishedGitSourceError("Cargo.lock is missing from the committed source tree")
    return {
        "source_commit": commit,
        "source_tree_sha256": _source_digest(paths, blobs),
        "cargo_lock_sha256": hashlib.sha256(lock).hexdigest(),
        "published_dependencies": _lock_packages(lock),
    }


def verify_git_provenance(root: Path, value: object, expected_commit: str) -> dict[str, Any]:
    """Require all provenance fields to match the exact expected Git commit."""
    if not isinstance(value, dict):
        raise PublishedGitSourceError("provenance must be an object")
    actual = git_source_identity(root, expected_commit)
    if set(value) != set(actual):
        raise PublishedGitSourceError("provenance fields are incomplete or unexpected")
    if value != actual:
        differing = sorted(key for key in actual if value.get(key) != actual[key])
        raise PublishedGitSourceError(f"provenance does not match committed Git objects: {', '.join(differing)}")
    return actual
