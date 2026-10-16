#!/usr/bin/env python3
"""Generate a deterministic DOCX whose entries use ZIP data descriptors."""

from __future__ import annotations

import argparse
import io
import struct
import zipfile
from pathlib import Path


DATA_DESCRIPTOR_FLAG = 0x0008
LOCAL_FILE_HEADER = struct.Struct("<IHHHHHIIIHH")
LOCAL_FILE_SIGNATURE = 0x04034B50


class NonSeekableBuffer(io.RawIOBase):
    """Write-only stream that forces zipfile to emit data descriptors."""

    def __init__(self) -> None:
        self._data = bytearray()

    def writable(self) -> bool:
        return True

    def seekable(self) -> bool:
        return False

    def write(self, data: bytes | bytearray) -> int:
        self._data.extend(data)
        return len(data)

    def tell(self) -> int:
        return len(self._data)

    def value(self) -> bytes:
        return bytes(self._data)


def generate(source: Path) -> bytes:
    sink = NonSeekableBuffer()
    with zipfile.ZipFile(source) as source_archive:
        with zipfile.ZipFile(
            sink,
            mode="w",
            compression=zipfile.ZIP_DEFLATED,
            compresslevel=9,
            strict_timestamps=True,
        ) as output_archive:
            for source_info in sorted(source_archive.infolist(), key=lambda info: info.filename):
                info = zipfile.ZipInfo(source_info.filename, (2026, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                info.create_system = 3
                info.external_attr = source_info.external_attr
                output_archive.writestr(info, source_archive.read(source_info))
    return sink.value()


def descriptor_entries(payload: bytes) -> list[str]:
    entries = []
    with zipfile.ZipFile(io.BytesIO(payload)) as archive:
        for info in archive.infolist():
            header = LOCAL_FILE_HEADER.unpack_from(payload, info.header_offset)
            signature, flag = header[0], header[2]
            crc32, compressed_size, uncompressed_size = header[6:9]
            if signature != LOCAL_FILE_SIGNATURE:
                raise ValueError(f"invalid local file header for {info.filename}")
            if not flag & DATA_DESCRIPTOR_FLAG:
                raise ValueError(f"data descriptor flag is absent for {info.filename}")
            if (crc32, compressed_size, uncompressed_size) != (0, 0, 0):
                raise ValueError(f"local header unexpectedly contains sizes for {info.filename}")
            if not info.flag_bits & DATA_DESCRIPTOR_FLAG:
                raise ValueError(f"central directory flag is absent for {info.filename}")
            entries.append(info.filename)
    return entries


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()

    payload = generate(args.source)
    entries = descriptor_entries(payload)
    if "word/document.xml" not in entries:
        raise ValueError("source DOCX has no word/document.xml entry")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(payload)
    print(
        f"generated {args.output} with {len(entries)} data-descriptor entries; "
        "word/document.xml local sizes=0"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
