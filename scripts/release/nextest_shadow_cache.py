"""Validate an existing Cargo build cache for diagnostic-only shadow runs."""

from __future__ import annotations

import os
from pathlib import Path
import stat
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "ci"))
from cargo_sweep_guard import acquire_profile_locks, release_profile_locks
from cargo_sweep_target import CleanupUnavailable, profile_directories, require_local_lock_filesystem


TAG_SIGNATURE = b"Signature: 8a477f597d28d172789f06886806bc55\n"
MAX_TAG_BYTES = 4096
MAX_TREE_ENTRIES = 65536
MAX_TREE_DEPTH = 16


def _canonical(path: Path, label: str) -> Path:
    if not path.is_absolute() or ".." in path.parts or path.is_symlink():
        raise ValueError(f"{label} is not a canonical path")
    resolved = path.resolve()
    if path != resolved:
        raise ValueError(f"{label} has a symlink ancestor")
    return path


def _validate_location(repo: Path, target: Path) -> Path:
    repo = _canonical(repo, "repository")
    if not repo.is_dir():
        raise ValueError("repository is not a directory")
    target = _canonical(target, "build cache")
    base = repo / "tmp/nextest-shadow"
    try:
        relative = target.relative_to(base)
    except ValueError as error:
        raise ValueError("build cache is outside nextest shadow runs") from error
    if len(relative.parts) != 2 or relative.parts[1] != "coverage-target":
        raise ValueError("build cache must be run/coverage-target")
    if not target.is_dir():
        raise ValueError("build cache is not a directory")
    return target


def _file_identity(result: os.stat_result) -> tuple[int, int, int, int, int]:
    return (result.st_dev, result.st_ino, result.st_size,
            result.st_mtime_ns, result.st_ctime_ns)


def _read_tag(tag: Path) -> bytes:
    if not hasattr(os, "O_NOFOLLOW"):
        raise ValueError("safe no-follow file opening is unavailable")
    flags = os.O_RDONLY | os.O_NONBLOCK | os.O_NOFOLLOW
    descriptor = -1
    try:
        path_before = os.stat(tag, follow_symlinks=False)
        if not stat.S_ISREG(path_before.st_mode) or path_before.st_size > MAX_TAG_BYTES:
            raise ValueError("Cargo cache signature is not a bounded regular file")
        descriptor = os.open(tag, flags)
        with os.fdopen(descriptor, "rb") as stream:
            descriptor = -1
            before = os.fstat(stream.fileno())
            if not stat.S_ISREG(before.st_mode) or _file_identity(path_before) != _file_identity(before):
                raise ValueError("Cargo cache signature changed before reading")
            content = stream.read(MAX_TAG_BYTES + 1)
            after = os.fstat(stream.fileno())
        path_after = os.stat(tag, follow_symlinks=False)
    except OSError as error:
        raise ValueError("Cargo cache signature cannot be read") from error
    finally:
        if descriptor >= 0:
            os.close(descriptor)
    if (_file_identity(before) != _file_identity(after) or
            _file_identity(after) != _file_identity(path_after) or
            len(content) != before.st_size or len(content) > MAX_TAG_BYTES):
        raise ValueError("Cargo cache signature changed while reading")
    return content


def _validate_tag(target: Path) -> None:
    if not _read_tag(target / "CACHEDIR.TAG").startswith(TAG_SIGNATURE):
        raise ValueError("Cargo cache signature is invalid")


def _walk_tree(path: Path, depth: int, entries: list[int],
               max_entries: int = MAX_TREE_ENTRIES) -> None:
    if depth > MAX_TREE_DEPTH:
        raise ValueError("Cargo cache tree is too deep")
    try:
        with os.scandir(path) as children:
            for child in children:
                entries[0] += 1
                if entries[0] > max_entries:
                    raise ValueError("Cargo cache tree has too many entries")
                child_path = Path(child.path)
                if child.is_symlink():
                    raise ValueError("symlink in Cargo cache tree")
                if child.is_dir(follow_symlinks=False):
                    _walk_tree(child_path, depth + 1, entries, max_entries)
                elif not child.is_file(follow_symlinks=False):
                    raise ValueError("non-regular entry in Cargo cache tree")
    except OSError as error:
        raise ValueError("Cargo cache tree cannot be inspected") from error


def validate_build_cache(repo: Path, target: Path) -> Path:
    """Validate and briefly lock existing profiles; never move, delete, or reuse results."""
    target = _validate_location(repo, target)
    _validate_tag(target)
    _walk_tree(target, 0, [0])
    descriptors = []
    try:
        profiles = profile_directories(target)
        if not profiles:
            raise ValueError("build cache has no Cargo profiles")
        require_local_lock_filesystem(target)
        descriptors = acquire_profile_locks(profiles)
    except (CleanupUnavailable, OSError) as error:
        raise ValueError(f"build cache is unavailable: {error}") from error
    finally:
        release_profile_locks(descriptors)
    return target
