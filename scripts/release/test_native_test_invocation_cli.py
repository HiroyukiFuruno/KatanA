"""Exercise the comparison CLI with real and malformed diagnostic inputs."""

from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from test_native_test_comparison_fixture import create_fixture, record_fixture, ROOT


CLI = ROOT / "scripts/release/compare-test-invocations.py"


def _run(directory: Path, summary: Path | None = None) -> subprocess.CompletedProcess[str]:
    environment = dict(os.environ)
    environment.pop("GITHUB_STEP_SUMMARY", None)
    if summary is not None:
        environment["GITHUB_STEP_SUMMARY"] = str(summary)
    return subprocess.run([sys.executable, str(CLI), "--directory", str(directory)],
                          cwd=ROOT, env=environment, capture_output=True, text=True, timeout=30)


class InvocationCliTests(unittest.TestCase):
    def assert_fresh_scope_is_unavailable(self, populated: Path) -> None:
        fresh = populated / "fresh-run"
        result = _run(fresh)
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads((fresh / "comparison.json").read_text())
        self.assertFalse(report["coverage_phase_inputs_complete"])
        self.assertFalse(report["selection_comparison_available"])
        self.assertEqual(report["input_sha256"], {})
        self.assertFalse(report["reuse_authorized"])

    def test_real_four_phase_inputs_report_fixture_skip_and_no_authority(self) -> None:
        with tempfile.TemporaryDirectory(prefix="inventory-cli-") as name:
            root = create_fixture(Path(name).resolve())
            output = root / "inventory"
            record_fixture(root, output)
            summary = output / "summary.md"
            result = _run(output, summary)
            self.assertEqual(result.returncode, 0, result.stderr)
            report = json.loads((output / "comparison.json").read_text(encoding="utf-8"))
            self.assertTrue(report["coverage_phase_inputs_complete"])
            self.assertFalse(report["execution_equivalent"])
            self.assertFalse(report["reuse_authorized"])
            self.assertIn("Test skip/reuse authorization: false", summary.read_text())
            if os.name == "posix":
                self.assertTrue(report["selection_comparison_available"])
                self.assertEqual(len(report["normal_only_candidates"]), 1)
                self.assertEqual(report["normal_only_candidates"][0][1], "fixture")
            else:
                self.assertFalse(report["selection_comparison_available"])
                self.assertTrue(any("incomplete" in reason for reason in report["reasons"]))
            self.assert_fresh_scope_is_unavailable(output)

    def test_missing_phase_is_unavailable(self) -> None:
        with tempfile.TemporaryDirectory(prefix="inventory-missing-") as name:
            directory = Path(name)
            (directory / "normal.json").write_text("{}\n", encoding="utf-8")
            (directory / "coverage").mkdir()
            result = _run(directory)
            self.assertEqual(result.returncode, 0, result.stderr)
            report = json.loads((directory / "comparison.json").read_text(encoding="utf-8"))
            self.assertFalse(report["coverage_phase_inputs_complete"])
            self.assertFalse(report["selection_comparison_available"])
            self.assertFalse(report["reuse_authorized"])

    def test_duplicate_json_keys_and_oversize_manifest_fail_closed(self) -> None:
        for payload in ('{"schema":1,"schema":1}\n', '{"x":NaN}\n',
                        '{"x":"' + "a" * (16 * 1024 * 1024) + '"}\n'):
            with self.subTest(size=len(payload)), tempfile.TemporaryDirectory(prefix="inventory-invalid-") as name:
                directory = Path(name)
                (directory / "normal.json").write_text(payload, encoding="utf-8")
                result = _run(directory)
                self.assertEqual(result.returncode, 2)
                self.assertFalse((directory / "comparison.json").exists())


if __name__ == "__main__":
    unittest.main()
