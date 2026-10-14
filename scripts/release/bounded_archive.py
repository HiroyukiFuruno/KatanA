"""Bound gzip expansion before tarfile parses archive metadata."""

from __future__ import annotations

import gzip
import time
from io import BytesIO

CHUNK_BYTES = 64 * 1024

class ArchiveLimitError(ValueError):
    """The compressed archive exceeded its bounded read contract."""


class BoundedArchiveReader:
    def __init__(self, compressed: bytes, limit: int, deadline: float | None = None) -> None:
        self._gzip = gzip.GzipFile(fileobj=BytesIO(compressed), mode="rb")
        self._limit = limit
        self._deadline = deadline
        self._total = 0

    def read(self, size: int = -1) -> bytes:
        if self._deadline is not None and time.monotonic() >= self._deadline:
            raise ArchiveLimitError("archive deadline exceeded")
        request = CHUNK_BYTES if size < 0 else min(size, CHUNK_BYTES)
        data = self._gzip.read(request)
        self._total += len(data)
        if self._total > self._limit:
            raise ArchiveLimitError("archive expanded size exceeds limit")
        return data

    def close(self) -> None:
        self._gzip.close()
