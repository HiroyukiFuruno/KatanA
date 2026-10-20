import json
import importlib.util
from pathlib import Path
import sys
import tempfile
import textwrap
import unittest

MODULE_PATH = Path(__file__).with_name("run-nextest-shadow.py")
SPEC = importlib.util.spec_from_file_location("run_nextest_shadow", MODULE_PATH)
RUNNER = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(RUNNER)
collect_shadow = RUNNER.collect_shadow
run_phase = RUNNER.run_phase


class NextestShadowRunnerTests(unittest.TestCase):
    def child(self, root: Path) -> Path:
        child = root / "child.py"
        child.write_text(textwrap.dedent("""
            import os
            import pathlib
            import sys
            if "build" in sys.argv:
                worker = pathlib.Path(os.environ["CARGO_TARGET_DIR"]) / "debug/kdv-office-worker"
                worker.parent.mkdir(parents=True, exist_ok=True)
                worker.write_bytes(b"worker")
                worker.chmod(worker.stat().st_mode | 0o100)
        """), encoding="utf-8")
        return child

    def test_successful_child_without_store_writes_raw_phase_receipt(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            child = self.child(root)
            output = root / "output"
            output.mkdir()
            phase = {"phase": "phase-a", "argv": []}
            result = run_phase(root, output, [sys.executable, str(child)], {}, phase)
            receipt = json.loads((output / "phase-a/phase.json").read_text())
            self.assertEqual(result["exit_code"], 0)
            self.assertEqual(receipt["exit_code"], 0)
            self.assertEqual(receipt["argv"], [sys.executable, str(child)])
            self.assertIn("record_unavailable", receipt)

    def test_collect_shadow_returns_two_and_keeps_raw_success_stage(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            child = self.child(root)
            output = root / "output"
            output.mkdir()
            receipt = {"stages": []}
            environment = {"CARGO_TARGET_DIR": str(output / "coverage-target")}
            status = collect_shadow(root, output, [sys.executable, str(child)], environment, receipt)
            self.assertEqual(status, 2)
            self.assertEqual(receipt["stages"][1]["exit_code"], 0)
            self.assertIn("record_unavailable", receipt["stages"][1])

    def test_nonzero_child_exit_is_preserved_in_phase_receipt(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            child = root / "fail.py"
            child.write_text("raise SystemExit(7)\n", encoding="utf-8")
            output = root / "output"
            output.mkdir()
            result = run_phase(root, output, [sys.executable, str(child)], {},
                               {"phase": "phase-fail", "argv": []})
            receipt = json.loads((output / "phase-fail/phase.json").read_text())
            self.assertEqual(result["exit_code"], 7)
            self.assertEqual(receipt["exit_code"], 7)
            self.assertIn("record_unavailable", receipt)


if __name__ == "__main__":
    unittest.main()
