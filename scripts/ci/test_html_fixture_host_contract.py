import hashlib
import json
import tempfile
import unittest
from pathlib import Path

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
                {"type": "action", "action": {"open_url": {"url": self.source.as_uri() + "#s15"}}},
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


if __name__ == "__main__":
    unittest.main()
