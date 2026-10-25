import importlib.util
import os
from pathlib import Path
import sys
import tempfile
import unittest

from nextest_shadow_test_support import phase, recording_child

MODULE_PATH = Path(__file__).with_name("run-nextest-shadow.py")
SPEC = importlib.util.spec_from_file_location("run_nextest_shadow_junit", MODULE_PATH)
RUNNER = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(RUNNER)
run_phase = RUNNER.run_phase


class NextestShadowJunitRunnerTests(unittest.TestCase):
    def test_success_exit_cannot_hide_failed_cases_and_failure_exit_is_preserved(self):
        for status, failed_report in ((0, True), (7, False)):
            with self.subTest(status=status), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                output = root / "output"
                output.mkdir()
                child = recording_child(root, status, failed_report)
                result = run_phase(root, output, [sys.executable, str(child)],
                                   {**os.environ, "KATANA_SHADOW_PROFILE_ID": "run"},
                                   phase(root, output, "phase-result"))
                self.assertEqual(result["exit_code"], status)
                self.assertIn("junit_unavailable", result)

    def test_initial_receipt_tracks_the_junit_reader_source(self):
        root = Path(__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            environment = {"CARGO_TARGET_DIR": str(output / "target"),
                           "KATANA_SHADOW_PROFILE_ID": "run"}
            receipt = RUNNER.initial_receipt(root, output, ["cargo"], environment, {}, {})
            self.assertEqual(receipt["producer_sha256"]["nextest_shadow_junit_io.py"],
                             RUNNER.file_digest(MODULE_PATH.with_name("nextest_shadow_junit_io.py")))

    def test_phase_receipt_binds_xml_identity_to_recorded_run(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            output = (root / "output").resolve()
            output.mkdir()
            phase_data = phase(root, output, "phase-junit")
            result = run_phase(root, output, [sys.executable, str(recording_child(root))],
                               {**os.environ, "KATANA_SHADOW_PROFILE_ID": "test-run"}, phase_data)
            self.assertEqual(result["exit_code"], 0)
            self.assertNotIn("junit_unavailable", result)
            self.assertEqual(result["junit"]["run_uuid"], "123e4567-e89b-12d3-a456-426614174000")
            self.assertEqual(result["junit"]["testcases"][0]["status"], "passed")
