"""Bounded private files for one admitted cleanup operation."""

import os
from pathlib import Path
import stat

from release_cleanup_audit import CleanupRefused

MAX_INTENT_BYTES = 4096
INTENT_NAME = "request.json"


def _private_metadata(metadata: os.stat_result, directory: bool) -> None:
    valid_kind = stat.S_ISDIR(metadata.st_mode) if directory else stat.S_ISREG(metadata.st_mode)
    mode = 0o700 if directory else 0o600
    if not valid_kind or stat.S_IMODE(metadata.st_mode) != mode or metadata.st_uid != os.getuid():
        raise CleanupRefused("cleanup intent requires a private owned directory and regular file")


def _directory(path: Path) -> int:
    if not all(hasattr(os, name) for name in ("O_NOFOLLOW", "O_DIRECTORY", "getuid")):
        raise CleanupRefused("private cleanup intents require supported native file protection")
    if os.open not in os.supports_dir_fd:
        raise CleanupRefused("private cleanup intents require directory-relative open")
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        _private_metadata(os.fstat(descriptor), True)
    except (OSError, CleanupRefused):
        os.close(descriptor)
        raise
    return descriptor


def read_private(path: Path) -> bytes:
    parent = _directory(path.parent)
    try:
        descriptor = os.open(path.name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=parent)
    finally:
        os.close(parent)
    with os.fdopen(descriptor, "rb") as stream:
        metadata = os.fstat(stream.fileno())
        _private_metadata(metadata, False)
        if metadata.st_size > MAX_INTENT_BYTES:
            raise CleanupRefused("cleanup intent exceeds the input bound")
        payload = stream.read(MAX_INTENT_BYTES + 1)
    if len(payload) > MAX_INTENT_BYTES:
        raise CleanupRefused("cleanup intent grew beyond the input bound")
    return payload


def write_private(directory: Path, payload: bytes) -> Path:
    if len(payload) > MAX_INTENT_BYTES:
        raise CleanupRefused("cleanup intent exceeds the input bound")
    parent = _directory(directory)
    try:
        descriptor = os.open(INTENT_NAME, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                             mode=0o600, dir_fd=parent)
    finally:
        os.close(parent)
    with os.fdopen(descriptor, "wb") as stream:
        os.fchmod(stream.fileno(), 0o600)
        stream.write(payload)
    return directory / INTENT_NAME
