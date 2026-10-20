import hashlib
import os
from pathlib import Path
import tempfile
import unittest

from nextest_shadow_records import _ensure_unchanged, collect_records


class NextestShadowRecordsTests(unittest.TestCase):
    def write_store(self, root: Path, relative: str, content: bytes = b"store") -> Path:
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
        return path

    def test_collects_relative_size_and_sha256_without_reading_zip(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "state"
            path = self.write_store(root, "phase-a/.nextest/store.zip", b"not-a-zip")
            expected = hashlib.sha256(b"not-a-zip").hexdigest()
            self.assertEqual(collect_records(root.resolve()), [{
                "path": "phase-a/.nextest/store.zip",
                "size": 9,
                "sha256": expected,
            }])

    def test_empty_state_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(ValueError):
                collect_records(Path(directory).resolve())

    def test_rejects_symlink_root_ancestor_and_alias(self):
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            real = base / "real"
            self.write_store(real, "phase/store.zip")
            root_alias = base / "alias"
            root_alias.symlink_to(real, target_is_directory=True)
            with self.assertRaises(ValueError):
                collect_records(root_alias)
            nested = base / "nested"
            nested.symlink_to(real, target_is_directory=True)
            with self.assertRaises(ValueError):
                collect_records(nested / "phase")
            child_alias = real / "phase" / "alias.zip"
            child_alias.symlink_to(real / "phase" / "store.zip")
            with self.assertRaises(ValueError):
                collect_records(real.resolve())

    def test_rejects_nonregular_store_and_file_root(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "state"
            (root / "phase").mkdir(parents=True)
            (root / "phase" / "store.zip").mkdir()
            with self.assertRaises(ValueError):
                collect_records(root.resolve())
            with self.assertRaises(ValueError):
                collect_records((root / "phase" / "store.zip").resolve())

    def test_identity_helper_rejects_real_file_change(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "state"
            path = self.write_store(root, "phase/store.zip")
            before = os.stat(path)
            path.write_bytes(b"changed")
            after = os.stat(path)
            with self.assertRaises(ValueError):
                _ensure_unchanged(before, after, after)

    def test_rejects_depth_and_count_limits(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "state"
            self.write_store(root, "a/b/c/d/e/f/g/h/i/store.zip")
            with self.assertRaises(ValueError):
                collect_records(root.resolve())
            root = Path(directory) / "many"
            for index in range(17):
                self.write_store(root, f"phase-{index}/store.zip", str(index).encode())
            with self.assertRaises(ValueError):
                collect_records(root.resolve())

    def test_rejects_total_size_limit_before_hashing(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "state"
            path = self.write_store(root, "phase/store.zip")
            with path.open("r+b") as stream:
                stream.truncate(64 * 1024 * 1024 + 1)
            with self.assertRaises(ValueError):
                collect_records(root.resolve())

    def test_rejects_nonstore_entry_flood_and_fifo(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "state"
            phase = root / "phase"
            phase.mkdir(parents=True)
            for index in range(4097):
                (phase / f"entry-{index}").write_bytes(b"x")
            with self.assertRaises(ValueError):
                collect_records(root.resolve())
            fifo_root = Path(directory) / "fifo"
            fifo_root.mkdir()
            os.mkfifo(fifo_root / "store.zip")
            with self.assertRaises(ValueError):
                collect_records(fifo_root.resolve())

    def test_rejects_relative_and_parent_traversal_roots(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "state"
            self.write_store(root, "phase/store.zip")
            with self.assertRaises(ValueError):
                collect_records(Path("relative-state"))
            with self.assertRaises(ValueError):
                collect_records((root / "phase" / ".."))


if __name__ == "__main__":
    unittest.main()
