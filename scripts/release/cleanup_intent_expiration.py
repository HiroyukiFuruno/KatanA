"""Expire only the unchanged record issued by this operation."""

import hashlib
import os
import stat

from cleanup_intent_io import INTENT_NAME, MAX_INTENT_BYTES
from release_cleanup_audit import CleanupRefused


def record_identity(metadata: os.stat_result) -> tuple[int, int, int]:
    return metadata.st_dev, metadata.st_ino, metadata.st_mode


def expire_record(directory: int, identity: tuple[int, int, int] | None,
                  digest: str | None) -> None:
    try:
        metadata = os.stat(INTENT_NAME, dir_fd=directory, follow_symlinks=False)
    except FileNotFoundError:
        return
    if identity is None or record_identity(metadata) != identity or not stat.S_ISREG(metadata.st_mode):
        raise CleanupRefused("cleanup record was replaced; retaining unknown state")
    descriptor = os.open(INTENT_NAME, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK,
                         dir_fd=directory)
    with os.fdopen(descriptor, "rb") as stream:
        if record_identity(os.fstat(stream.fileno())) != identity:
            raise CleanupRefused("cleanup record changed while expiring")
        payload = stream.read(MAX_INTENT_BYTES + 1)
    if hashlib.sha256(payload).hexdigest() != digest:
        raise CleanupRefused("cleanup record content changed; retaining unknown state")
    latest = os.stat(INTENT_NAME, dir_fd=directory, follow_symlinks=False)
    if record_identity(latest) != identity:
        raise CleanupRefused("cleanup record changed before expiration")
    os.unlink(INTENT_NAME, dir_fd=directory)
