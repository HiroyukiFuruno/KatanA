import hashlib
import json
import tempfile
import unittest
from pathlib import Path

import html_fixture_host_contract as contract
from html_fixture_host_contract import validate


class HostContractTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.source = Path(self.directory.name) / "要件 定義.html"
        self.source.write_bytes(b"<html>actual input</html>")
        self.sha256 = hashlib.sha256(self.source.read_bytes()).hexdigest()
        self.request = Path(self.directory.name) / "request.json"
        self.payload = {
            "fixture": {"workspace_files": [{"name": self.source.name, "source": str(self.source)}]},
            "steps": [
                {"type": "launch", "viewport": {"width": 1280, "height": 900}},
                {"type": "open_file", "file_name": self.source.name,
                 "wait_for_html_frame": True, "max_first_frame_seconds": 60},
                {"type": "action", "action": {"open_url": {"url": self.source.as_uri() + "#s15", "timeout_seconds": 60}}},
                {"type": "action", "action": {"close_active_document": {"wait_seconds": 5.0}}},
                {"type": "record_runtime_snapshot", "name": "closed_idle"},
                {"type": "quit"},
            ],
        }

    def check(self):
        self.request.write_text(json.dumps(self.payload), encoding="utf-8")
        return validate(self.source, self.request, self.sha256)

    def test_exact_source_and_percent_encoded_navigation(self):
        evidence = self.check()
        self.assertEqual(evidence["source_sha256"], self.sha256)
        self.assertEqual(evidence["mode"], "in_process_host")
        self.assertEqual(evidence["fragment"], "s15")

    def test_changed_source_is_rejected(self):
        self.source.write_bytes(b"changed input")
        with self.assertRaisesRegex(ValueError, "SHA-256 mismatch"):
            self.check()

    def test_wrong_viewport_is_rejected(self):
        self.payload["steps"][0]["viewport"]["height"] = 800
        with self.assertRaisesRegex(ValueError, "1280x900"):
            self.check()

    def test_wrong_fragment_is_rejected(self):
        self.payload["steps"][2]["action"]["open_url"]["url"] = self.source.as_uri() + "#changelog"
        with self.assertRaisesRegex(ValueError, "#s15"):
            self.check()

    def test_wrong_source_is_rejected(self):
        other = Path(self.directory.name) / "other.html"
        other.write_bytes(self.source.read_bytes())
        self.payload["fixture"]["workspace_files"][0]["source"] = str(other)
        with self.assertRaisesRegex(ValueError, "source does not match"):
            self.check()

    def test_initial_frame_timeout_cannot_be_extended(self):
        self.payload["steps"][1]["max_first_frame_seconds"] = 120
        with self.assertRaisesRegex(ValueError, "60-second"):
            self.check()

    def test_missing_resource_close_is_rejected(self):
        del self.payload["steps"][3]
        with self.assertRaisesRegex(ValueError, "resource close"):
            self.check()

    def test_resource_close_timeout_cannot_be_extended(self):
        self.payload["steps"][3]["action"]["close_active_document"]["wait_seconds"] = 30
        with self.assertRaisesRegex(ValueError, "resource close"):
            self.check()

    def test_missing_closed_idle_snapshot_is_rejected(self):
        del self.payload["steps"][4]
        with self.assertRaisesRegex(ValueError, "closed idle"):
            self.check()

    def test_open_and_navigation_have_independent_deadlines(self):
        first = contract.observe_operation(self.payload, "step 2/6: open_file\n")
        navigation = contract.observe_operation(self.payload, "step 2/6: open_file\nstep 3/6: action\n")
        self.assertEqual(first, (2, "open_file", 60))
        self.assertEqual(navigation, (3, "action", 60))

    def test_close_keeps_its_existing_five_second_deadline(self):
        self.assertEqual(contract.observe_operation(self.payload, "step 4/6: action\n"), (4, "action", 5))

    def test_mismatched_operation_marker_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "operation marker"):
            contract.observe_operation(self.payload, "step 3/6: open_file\n")


class HostLogTests(unittest.TestCase):
    CLOSED = ('  runtime snapshot "closed_idle": RuntimeSnapshot { previews: 0, html_surfaces: 0, '
              'document_surfaces: 0, office_workers: 0, frames: 0, textures: 0, cache_entries: 0 }\n')

    def test_success_requires_frame_and_close(self):
        result = contract.summarize_log('HTML browser first frame ready in 35.125s\n' + self.CLOSED)
        self.assertEqual(result["first_frame_seconds"], 35.125)
        self.assertTrue(result["successful_frame_and_close"])

    def test_typed_failure_is_not_a_successful_frame(self):
        result = contract.summarize_log("HTML browser did not produce an initial frame within 60.00s")
        self.assertTrue(result["typed_failure_observed"])
        self.assertFalse(result["successful_frame_and_close"])

    def test_frame_without_close_is_rejected(self):
        result = contract.summarize_log("HTML browser first frame ready in 1.000s")
        self.assertFalse(result["successful_frame_and_close"])

    def test_late_frame_is_rejected(self):
        result = contract.summarize_log('HTML browser first frame ready in 60.001s\n' + self.CLOSED)
        self.assertFalse(result["successful_frame_and_close"])

    def test_retained_resources_are_rejected(self):
        result = contract.summarize_log(
            'HTML browser first frame ready in 1.000s\n' + self.CLOSED.replace("frames: 0", "frames: 1")
        )
        self.assertFalse(result["successful_frame_and_close"])


if __name__ == "__main__":
    unittest.main()
