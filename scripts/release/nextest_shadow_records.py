"""Collect non-authoritative digests emitted by isolated nextest state roots."""

from __future__ import annotations

import hashlib
import os
from pathlib import Path
import stat


MAX_RECORDS = 16
MAX_DEPTH = 8
MAX_ENTRIES = 4096
MAX_TOTAL_BYTES = 64 * 1024 * 1024
CHUNK_SIZE = 1024 * 1024


def _validate_root(state_root: Path) -> Path:
    root = Path(state_root)
    if not root.is_absolute() or ".." in root.parts:
        raise ValueError("state root must be an absolute canonical directory")
    absolute = Path(os.path.abspath(root))
    if absolute != absolute.resolve() or not absolute.is_dir():
        raise ValueError("state root must be a canonical directory")
    for parent in absolute.parents:
        if parent.is_symlink():
            raise ValueError("state root has a symlink ancestor")
    return absolute


def _walk(root: Path, current: Path, depth: int, counts: dict[str, int]) -> list[Path]:
    if depth > MAX_DEPTH:
        raise ValueError("nextest state tree is too deep")
    found: list[Path] = []
    try:
        with os.scandir(current) as entries:
            for entry in entries:
                counts["entries"] += 1
                if counts["entries"] > MAX_ENTRIES:
                    raise ValueError("nextest state tree has too many entries")
                path = Path(entry.path)
                if entry.is_symlink():
                    raise ValueError("symlink in nextest state tree")
                if entry.is_dir(follow_symlinks=False):
                    found.extend(_walk(root, path, depth + 1, counts))
                elif entry.name == "store.zip":
                    if not entry.is_file(follow_symlinks=False):
                        raise ValueError("store.zip must be a regular file")
                    counts["stores"] += 1
                    if counts["stores"] > MAX_RECORDS:
                        raise ValueError("nextest store.zip count is outside the supported range")
                    found.append(path)
    except OSError as error:
        raise ValueError("nextest state tree cannot be read") from error
    return found


def _signature(stat_result: os.stat_result) -> tuple[int, int, int, int, int]:
    return (stat_result.st_dev, stat_result.st_ino, stat_result.st_size,
            stat_result.st_mtime_ns, stat_result.st_ctime_ns)


def _ensure_unchanged(before: os.stat_result, after: os.stat_result,
                      path_after: os.stat_result) -> None:
    if (_signature(before) != _signature(after) or
            _signature(after) != _signature(path_after)):
        raise ValueError("store.zip changed while hashing")


def _read_hash(stream, remaining: int) -> str:
    digest = hashlib.sha256()
    read_bytes = 0
    for chunk in iter(lambda: stream.read(CHUNK_SIZE), b""):
        read_bytes += len(chunk)
        if read_bytes > remaining:
            raise ValueError("nextest store.zip bytes exceed the supported limit")
        digest.update(chunk)
    return digest.hexdigest()


def _hash_file(path: Path, remaining: int) -> tuple[int, str]:
    if not hasattr(os, "O_NOFOLLOW"):
        raise ValueError("safe no-follow file opening is unavailable")
    flags = os.O_RDONLY | os.O_NONBLOCK | os.O_NOFOLLOW
    descriptor = -1
    try:
        path_before = os.stat(path, follow_symlinks=False)
        if not stat.S_ISREG(path_before.st_mode) or path_before.st_size < 0:
            raise ValueError("store.zip must be a regular file")
        if path_before.st_size > remaining:
            raise ValueError("nextest store.zip bytes exceed the supported limit")
        descriptor = os.open(path, flags)
        with os.fdopen(descriptor, "rb") as stream:
            descriptor = -1
            before = os.fstat(stream.fileno())
            if not stat.S_ISREG(before.st_mode) or _signature(path_before) != _signature(before):
                raise ValueError("store.zip changed before hashing")
            digest = _read_hash(stream, remaining)
            after = os.fstat(stream.fileno())
        path_after = os.stat(path, follow_symlinks=False)
    except OSError as error:
        raise ValueError("store.zip cannot be read") from error
    finally:
        if descriptor >= 0:
            os.close(descriptor)
    _ensure_unchanged(before, after, path_after)
    return before.st_size, digest


def collect_records(state_root: Path) -> list[dict]:
    """Return only file identity records; zip contents never establish test success."""
    root = _validate_root(state_root)
    paths = _walk(root, root, 0, {"entries": 0, "stores": 0})
    if not paths:
        raise ValueError("nextest store.zip count is outside the supported range")
    records = []
    total = 0
    for path in paths:
        size, digest = _hash_file(path, MAX_TOTAL_BYTES - total)
        total += size
        if total > MAX_TOTAL_BYTES:
            raise ValueError("nextest store.zip bytes exceed the supported limit")
        records.append({"path": str(path.relative_to(root)), "size": size, "sha256": digest})
    return records
