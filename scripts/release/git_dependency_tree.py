"""Read dependency inputs from immutable regular Git blobs, not the worktree."""

from dataclasses import dataclass
from pathlib import Path
import time

from bounded_git_output import bounded_git
from downstream_evidence_fields import _safe_path
from downstream_git_binding import GitBindingError
from push_ref_updates import OID_RE

MAX_DEPENDENCY_FILES = 4096
MAX_DEPENDENCY_BLOB_BYTES = 1_048_576
MAX_DEPENDENCY_TREE_BYTES = 16_777_216
DEPENDENCY_TREE_TIMEOUT_SECONDS = 30


@dataclass(frozen=True)
class GitDependencyFile:
    path: str
    blob: str
    content: bytes


@dataclass(frozen=True)
class GitDependencyDelta:
    path: str
    before: GitDependencyFile | None
    after: GitDependencyFile | None


def _dependency_entry(entry: bytes) -> tuple[str, str] | None:
    metadata, separator, path = entry.partition(b"\t")
    if not separator:
        raise GitBindingError("Git tree listing has invalid metadata")
    if path.rsplit(b"/", 1)[-1] not in (b"Cargo.toml", b"Cargo.lock"):
        return None
    fields = metadata.split(b" ")
    if len(fields) != 3 or fields[0] not in (b"100644", b"100755") or fields[1] != b"blob":
        raise GitBindingError("dependency input is not a regular Git blob")
    try:
        name, oid = path.decode("utf-8"), fields[2].decode("ascii")
    except UnicodeError as error:
        raise GitBindingError("dependency tree identity is not valid text") from error
    _safe_path(name, "dependency input")
    if OID_RE.fullmatch(oid) is None:
        raise GitBindingError("dependency blob identity is not a full OID")
    return name, oid


def _dependency_entries(raw: bytes) -> tuple[tuple[str, str], ...]:
    if raw and not raw.endswith(b"\0"):
        raise GitBindingError("Git tree listing is incomplete")
    result, seen = [], set()
    for entry in raw.split(b"\0")[:-1]:
        record = _dependency_entry(entry)
        if record is None:
            continue
        if record[0] in seen:
            raise GitBindingError("dependency tree path is duplicated")
        seen.add(record[0])
        result.append(record)
        if len(result) > MAX_DEPENDENCY_FILES:
            raise GitBindingError("dependency file count exceeds the limit")
    return tuple(result)


def _read(root: Path, args: list[str], deadline: float,
          limit: int = 2_097_152) -> bytes:
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        raise GitBindingError("dependency tree collection exceeded its deadline")
    return bounded_git(root, args, stdout_limit=limit, timeout=remaining)


def _blob(root: Path, path: str, oid: str, deadline: float) -> GitDependencyFile:
    if _read(root, ["cat-file", "-t", oid], deadline).strip() != b"blob":
        raise GitBindingError("dependency input object is not an actual blob")
    size = _read(root, ["cat-file", "-s", oid], deadline).strip()
    if not size.isdigit() or int(size) > MAX_DEPENDENCY_BLOB_BYTES:
        raise GitBindingError("dependency blob exceeds the size limit")
    content = _read(root, ["cat-file", "blob", oid], deadline, MAX_DEPENDENCY_BLOB_BYTES)
    if len(content) != int(size):
        raise GitBindingError("dependency blob size is inconsistent")
    return GitDependencyFile(path, oid, content)


def _tree(root: Path, commit: str, deadline: float) -> tuple[GitDependencyFile, ...]:
    if not isinstance(commit, str) or OID_RE.fullmatch(commit) is None:
        raise GitBindingError("dependency tree requires a full lowercase commit OID")
    if _read(root, ["rev-parse", "--verify", commit], deadline).strip() != commit.encode("ascii"):
        raise GitBindingError("dependency tree identity is not a full commit OID")
    if _read(root, ["cat-file", "-t", commit], deadline).strip() != b"commit":
        raise GitBindingError("dependency tree source is not a commit")
    listing = _read(root, ["ls-tree", "-r", "-z", "--full-tree", commit], deadline)
    records, total = [], 0
    for path, oid in _dependency_entries(listing):
        record = _blob(root, path, oid, deadline)
        total += len(record.content)
        if total > MAX_DEPENDENCY_TREE_BYTES:
            raise GitBindingError("dependency tree content exceeds the aggregate limit")
        records.append(record)
    return tuple(records)


def read_dependency_tree(root: Path, commit: str) -> tuple[GitDependencyFile, ...]:
    return _tree(root, commit, time.monotonic() + DEPENDENCY_TREE_TIMEOUT_SECONDS)


def dependency_tree_changes(root: Path, before_commit: str | None,
                            after_commit: str) -> tuple[GitDependencyDelta, ...]:
    deadline = time.monotonic() + DEPENDENCY_TREE_TIMEOUT_SECONDS
    before = () if before_commit is None else _tree(root, before_commit, deadline)
    after = _tree(root, after_commit, deadline)
    old, new = {item.path: item for item in before}, {item.path: item for item in after}
    return tuple(GitDependencyDelta(path, old.get(path), new.get(path))
                 for path in sorted(set(old) | set(new)) if old.get(path) != new.get(path))
