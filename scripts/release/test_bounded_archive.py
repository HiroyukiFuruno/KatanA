from __future__ import annotations

import gzip
import tarfile
import unittest
from io import BytesIO

from bounded_archive import ArchiveLimitError, BoundedArchiveReader


class BoundedArchiveTests(unittest.TestCase):
    def test_compressed_metadata_body_is_bounded_before_tar_parse(self) -> None:
        reader = BoundedArchiveReader(gzip.compress(b"x" * 4097), 4096)
        with self.assertRaises(ArchiveLimitError):
            while reader.read(512):
                pass
        reader.close()

    def test_small_archive_reads_in_chunks(self) -> None:
        reader = BoundedArchiveReader(gzip.compress(b"payload"), 64)
        self.assertEqual(reader.read(1024), b"payload")
        reader.close()

    def test_pax_body_is_bounded_inside_tar_parser(self) -> None:
        header = tarfile.TarInfo("pax-header")
        header.type = tarfile.XHDTYPE
        body = b"9 path=oversized\n" + b"x" * 4080
        header.size = len(body)
        padded = body + b"\0" * ((-len(body)) % 512) + b"\0" * 1024
        compressed = gzip.compress(header.tobuf() + padded)
        reader = BoundedArchiveReader(compressed, 4096)
        with self.assertRaises(ArchiveLimitError):
            with tarfile.open(fileobj=reader, mode="r|") as archive:
                next(archive)
        reader.close()


if __name__ == "__main__":
    unittest.main()
