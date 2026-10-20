import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from nextest_shadow_cache import _walk_tree, validate_build_cache


TAG = b"Signature: 8a477f597d28d172789f06886806bc55\n"


class NextestShadowCacheTests(unittest.TestCase):
    def cache(self, root: Path, run: str = "run-1") -> Path:
        root = root.resolve()
        target = root / "tmp/nextest-shadow" / run / "coverage-target"
        target.mkdir(parents=True)
        (target / "CACHEDIR.TAG").write_bytes(TAG)
        (target / "debug/.fingerprint").mkdir(parents=True)
        return target

    def test_validates_existing_cargo_cache_and_returns_same_path(self):
        with tempfile.TemporaryDirectory() as directory:
            target = self.cache(Path(directory) / "repo")
            self.assertEqual(validate_build_cache(target.parents[3], target), target)

    def test_rejects_wrong_location_missing_tag_and_noncanonical_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            root = (Path(directory) / "repo").resolve()
            target = self.cache(root)
            for candidate in (root / "target", root / "tmp/nextest-shadow/run-1"):
                with self.assertRaises(ValueError):
                    validate_build_cache(root, candidate)
            (target / "CACHEDIR.TAG").unlink()
            with self.assertRaises(ValueError):
                validate_build_cache(root, target)

    def test_rejects_symlink_tag_and_oversized_tag(self):
        with tempfile.TemporaryDirectory() as directory:
            root = (Path(directory) / "repo").resolve()
            target = self.cache(root)
            tag = target / "CACHEDIR.TAG"
            tag.unlink()
            real = target / "real-tag"
            real.write_bytes(TAG)
            tag.symlink_to(real)
            with self.assertRaises(ValueError):
                validate_build_cache(root, target)
            tag.unlink()
            tag.write_bytes(TAG + b"x" * 4096)
            with self.assertRaises(ValueError):
                validate_build_cache(root, target)
            tag.write_bytes(b"")
            with self.assertRaises(ValueError):
                validate_build_cache(root, target)
            tag.write_bytes(b"not-a-cache-signature")
            with self.assertRaises(ValueError):
                validate_build_cache(root, target)

    def test_rejects_descendant_symlink_fifo_and_excessive_depth(self):
        with tempfile.TemporaryDirectory() as directory:
            root = (Path(directory) / "repo").resolve()
            target = self.cache(root)
            tag = target / "CACHEDIR.TAG"
            alias = target / "debug/alias"
            alias.symlink_to(target / "CACHEDIR.TAG")
            with self.assertRaises(ValueError):
                validate_build_cache(root, target)
            (target / "debug/entry-a").write_bytes(b"a")
            (target / "debug/entry-b").write_bytes(b"b")
            with self.assertRaises(ValueError):
                _walk_tree(target, 0, [0], max_entries=1)
            alias.unlink()
            if hasattr(os, "mkfifo"):
                os.mkfifo(target / "debug/fifo")
                with self.assertRaises(ValueError):
                    validate_build_cache(root, target)
                (target / "debug/fifo").unlink()
            nested = target / "debug"
            for index in range(17):
                nested /= f"level-{index}"
                nested.mkdir()
            with self.assertRaises(ValueError):
                validate_build_cache(root, target)

    def test_rejects_zero_profiles_and_held_profile_lock(self):
        with tempfile.TemporaryDirectory() as directory:
            root = (Path(directory) / "repo").resolve()
            target = self.cache(root)
            profile = target / "debug"
            (profile / ".fingerprint").rmdir()
            profile.rmdir()
            with self.assertRaises(ValueError):
                validate_build_cache(root, target)
            profile.mkdir()
            (profile / ".fingerprint").mkdir()
            lock = profile / ".cargo-lock"
            descriptor = os.open(lock, os.O_CREAT | os.O_RDWR)
            try:
                import fcntl
                fcntl.flock(descriptor, fcntl.LOCK_EX)
                code = "from pathlib import Path; from nextest_shadow_cache import validate_build_cache; validate_build_cache(Path(r'%s'), Path(r'%s'))" % (root, target)
                result = subprocess.run([sys.executable, "-c", code], capture_output=True,
                                        env={**os.environ, "PYTHONPATH": str(Path(__file__).parent)})
                self.assertNotEqual(result.returncode, 0)
            finally:
                os.close(descriptor)


if __name__ == "__main__":
    unittest.main()
