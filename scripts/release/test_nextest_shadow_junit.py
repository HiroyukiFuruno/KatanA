import hashlib
from pathlib import Path
import tempfile
import unittest

from nextest_shadow_junit import inspect_report, record_run_uuid, write_phase_config


RUN_UUID = "123e4567-e89b-12d3-a456-426614174000"


def report(uuid: str = RUN_UUID, body: str = "") -> str:
    return """<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<testsuites tests=\"2\" skipped=\"1\" failures=\"0\" errors=\"0\" uuid=\"%s\">
  <testsuite name=\"binary-a\" tests=\"2\" skipped=\"1\" failures=\"0\" errors=\"0\">
    <testcase classname=\"binary-a\" name=\"passes\" />
    <testcase classname=\"binary-a\" name=\"ignored\"><skipped /></testcase>
  </testsuite>
</testsuites>
%s""" % (uuid, body)


class NextestShadowJunitTests(unittest.TestCase):
    def test_inspects_bound_cases_and_report_digest(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory).resolve() / "phase.xml"
            path.write_text(report(), encoding="utf-8")
            result = inspect_report(path, RUN_UUID)
            self.assertEqual(result["test_count"], 2)
            self.assertEqual(result["skipped_count"], 1)
            self.assertEqual(result["testcases"][1], {
                "suite": "binary-a", "classname": "binary-a", "name": "ignored",
                "status": "skipped", "retry_count": 0})
            self.assertEqual(result["sha256"], hashlib.sha256(path.read_bytes()).hexdigest())

    def test_rejects_mismatch_duplicate_counter_and_unsafe_xml(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            for name, content in {
                "uuid.xml": report("123e4567-e89b-12d3-a456-426614174001"),
                "duplicate.xml": report().replace("name=\"ignored\"", "name=\"passes\""),
                "count.xml": report().replace("tests=\"2\"", "tests=\"3\"", 1),
                "skip.xml": report().replace("skipped=\"1\"", "skipped=\"0\"", 1),
                "dtd.xml": "<!DOCTYPE test [<!ENTITY xxe SYSTEM \"file:///nope\">]>" + report(),
                "retry.xml": report().replace("<skipped />", "<rerunFailure />"),
            }.items():
                path = root / name
                path.write_text(content, encoding="utf-8")
                with self.assertRaises(ValueError):
                    inspect_report(path, RUN_UUID)
            utf16 = root / "utf16.xml"
            utf16.write_bytes(report().encode("utf-16"))
            with self.assertRaises(ValueError):
                inspect_report(utf16, RUN_UUID)

    def test_records_failure_and_error_outcomes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            for status in ("failure", "error"):
                xml = report().replace('name=\"passes\" />',
                    f'name=\"passes\"><{status} /></testcase>').replace(
                    f'{status}s=\"0\"', f'{status}s=\"1\"')
                path = root / f"{status}.xml"
                path.write_text(xml, encoding="utf-8")
                result = inspect_report(path, RUN_UUID)
                self.assertEqual(result[f"{status}_count"], 1)
                self.assertEqual(result["testcases"][0]["status"], status)

    def test_record_run_uuid_requires_one_canonical_store_path(self):
        records = [{"path": f"projects/project/records/runs/{RUN_UUID}/store.zip",
                    "size": 1, "sha256": "a" * 64}]
        self.assertEqual(record_run_uuid(records), RUN_UUID)
        for invalid in ([], records * 2,
                        [{"path": "records/runs/not-a-uuid/store.zip", "size": 1,
                          "sha256": "a" * 64}]):
            with self.assertRaises(ValueError):
                record_run_uuid(invalid)

    def test_writes_new_phase_specific_config_without_overwrite(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            template = root / "template.toml"
            template.write_text("[experimental]\nrecord = true\n[record]\nenabled = true\n")
            result = write_phase_config(root, "ui-serial", "run-123", template)
            self.assertEqual(result["report_path"], str(root / "junit/ui-serial.xml"))
            contents = Path(result["config_path"]).read_text(encoding="utf-8")
            self.assertIn('path = "' + str(root / "junit/ui-serial.xml") + '"', contents)
            self.assertIn('report-skipped = "all"', contents)
            with self.assertRaises(ValueError):
                write_phase_config(root, "ui-serial", "run-123", template)


if __name__ == "__main__":
    unittest.main()
