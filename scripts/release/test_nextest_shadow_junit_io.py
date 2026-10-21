import os
from pathlib import Path
import tempfile
import unittest

from nextest_shadow_junit_io import MAX_REPORT_BYTES, read_report, write_phase_config


class NextestShadowJunitIOTests(unittest.TestCase):
    def test_rejects_nonregular_alias_and_oversized_reports(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            report = root / "report.xml"
            report.write_bytes(b"<testsuites />")
            alias = root / "alias.xml"
            alias.symlink_to(report)
            parent_alias = root / "alias"
            parent_alias.symlink_to(root, target_is_directory=True)
            fifo = root / "pipe.xml"
            os.mkfifo(fifo)
            for path in (root, alias, parent_alias / "report.xml", fifo, Path("relative.xml")):
                with self.subTest(path=path), self.assertRaises(ValueError):
                    read_report(path)
            with report.open("r+b") as stream:
                stream.truncate(MAX_REPORT_BYTES + 1)
            with self.assertRaises(ValueError):
                read_report(report)

    def test_config_preserves_repository_settings_and_rejects_alias_directory(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            template = root / "base.toml"
            original = "[profile.default]\nretries = 0\nfail-fast = false\n"
            template.write_text(original, encoding="utf-8")
            result = write_phase_config(root, "parallel", "namespace", template)
            self.assertTrue(Path(result["config_path"]).read_text().startswith(original))
            self.assertEqual(template.read_text(), original)
            other = root / "other"
            other.mkdir()
            (other / "junit").symlink_to(root / "junit", target_is_directory=True)
            with self.assertRaises(ValueError):
                write_phase_config(other, "serial", "namespace", template)


if __name__ == "__main__":
    unittest.main()
