"""Actual child processes remain mandatory on compiled-cache hits."""

import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


CLI = Path(__file__).resolve().parent / "observe-ci-command.py"
KEY = "windows-target-v1-" + "a" * 64


class ObserveCompiledCacheTests(unittest.TestCase):
    def run_child(self, root, status, eligible="true", key=KEY, hit="true"):
        counter = root / "counter"
        code = ("from pathlib import Path; p=Path(" + repr(str(counter)) + "); "
                "p.write_text(p.read_text()+'x' if p.exists() else 'x'); "
                f"raise SystemExit({status})")
        return subprocess.run([
            sys.executable, str(CLI), "--output", str(root / "timing.json"),
            "--cache-hit", "false", "--compiled-cache-eligible", eligible,
            "--compiled-cache-key", key, "--compiled-cache-hit", hit,
            "--", sys.executable, "-c", code,
        ], cwd=root, capture_output=True, text=True, timeout=30)

    def test_cache_hit_executes_every_child_and_preserves_failure(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            for status in (0, 7):
                result = self.run_child(root, status)
                self.assertEqual(result.returncode, status, result.stderr)
                payload = json.loads((root / "timing.json").read_text())
                self.assertEqual(payload["compiled_target_cache"],
                                 {"eligible": True, "key": KEY, "hit": True})
                self.assertFalse(payload["reuse_authorized"])
                self.assertEqual(payload["cold_warm_equivalence"], "not_evaluated")
            self.assertEqual((root / "counter").read_text(), "xx")

    def test_cache_ineligible_still_runs_fresh(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            result = self.run_child(root, 0, "false", "", "")
            self.assertEqual(result.returncode, 0, result.stderr)
            cache = json.loads((root / "timing.json").read_text())["compiled_target_cache"]
            self.assertEqual(cache, {"eligible": False, "key": None, "hit": None})
            self.assertEqual((root / "counter").read_text(), "x")

    def test_invalid_cache_metadata_never_hides_child_failure(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            for status, expected in ((0, 2), (9, 9)):
                result = self.run_child(root, status, key="\n### injected")
                self.assertEqual(result.returncode, expected, result.stderr)
            self.assertEqual((root / "counter").read_text(), "xx")
            self.assertFalse((root / "timing.json").exists())

    def test_unknown_hit_is_rejected_after_running_child(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            for status, expected in ((0, 2), (9, 9)):
                result = self.run_child(root, status, hit="unknown")
                self.assertEqual(result.returncode, expected, result.stderr)
            self.assertEqual((root / "counter").read_text(), "xx")
            self.assertFalse((root / "timing.json").exists())


if __name__ == "__main__":
    unittest.main()
