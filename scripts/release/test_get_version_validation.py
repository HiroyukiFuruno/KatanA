"""Untrusted release inputs must never reach workflow outputs unvalidated."""
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("get-version.sh").resolve()


class VersionValidationTest(unittest.TestCase):
    def run_version(self, event, input_version="", branch="", ref=""):
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory) / "output"
            out.write_text("existing=true\n")
            result = subprocess.run(["bash", str(SCRIPT), event, input_version, branch, ref, str(out)],
                                    capture_output=True, text=True, timeout=20)
            return result.returncode, out.read_text()

    def test_valid_event_versions_are_normalized(self):
        for args in (("workflow_dispatch", "0.22.47", "", ""),
                     ("workflow_dispatch", "v0.22.47", "", ""),
                     ("pull_request", "", "release/v0.22.47-batch", ""),
                     ("push", "", "", "refs/tags/v0.22.47")):
            code, output = self.run_version(*args)
            self.assertEqual(code, 0, args)
            self.assertEqual(output, "existing=true\nversion=v0.22.47\nversion_bare=0.22.47\n")

    def test_invalid_input_never_writes_outputs(self):
        for value in ("01.2.3", "1.02.3", "1.2.03", "1.2.3\nversion=malicious", "1.2.3;touch file", "v1.2.3-rc1", ""):
            for args in (("workflow_dispatch", value, "", ""),
                         ("pull_request", "", "release/v" + value, ""),
                         ("push", "", "", "refs/tags/v" + value)):
                with self.subTest(args=args):
                    code, output = self.run_version(*args)
                    self.assertNotEqual(code, 0)
                    self.assertEqual(output, "existing=true\n")


if __name__ == "__main__":
    unittest.main()
