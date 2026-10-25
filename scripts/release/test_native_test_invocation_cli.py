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
from native_test_run_recording import cargo_command


CLI = ROOT / "scripts/release/compare-test-invocations.py"


def _run(directory: Path, summary: Path | None = None) -> subprocess.CompletedProcess[str]:
    environment = dict(os.environ)
    environment.pop("GITHUB_STEP_SUMMARY", None)
    if summary is not None:
        environment["GITHUB_STEP_SUMMARY"] = str(summary)
    return subprocess.run([sys.executable, str(CLI), "--directory", str(directory)],
                          cwd=ROOT, env=environment, capture_output=True, text=True, timeout=30)


class InvocationCliTests(unittest.TestCase):
    def assert_real_timing(self, report: dict, summary: Path) -> None:
        timing = report["invocation_timing"]
        self.assertGreater(timing["normal"]["command_seconds"], 0)
        self.assertEqual(len(timing["coverage"]), 4)
        for phase in timing["coverage"]:
            self.assertGreater(phase["command_seconds"], 0)
            self.assertGreaterEqual(phase["diagnostics_seconds"], 0)
        self.assertFalse(timing["reuse_authorized"])
        phases = report["coverage_phase_candidates"]
        self.assertEqual(len(phases), 4)
        self.assertTrue(any(item["candidate_count"] > 0 for item in phases))
        self.assertEqual(len(report["coverage_phase_overlaps"]), 6)
        self.assertIn("Coverage phase candidates:", summary.read_text())
        self.assertIn("Pairwise intersections:", summary.read_text())
        self.assertIn("not execution count", report["coverage_phase_metrics_scope"])
        self.assertIn("Command seconds", summary.read_text())
        self.assertIn("not savings", summary.read_text())

    def assert_fresh_scope_is_unavailable(self, populated: Path) -> None:
        fresh = populated / "fresh-run"
        result = _run(fresh)
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads((fresh / "comparison.json").read_text())
        self.assertFalse(report["coverage_phase_inputs_complete"])
        self.assertFalse(report["selection_comparison_available"])
        self.assertEqual(report["input_sha256"], {})
        self.assertFalse(report["reuse_authorized"])

    def assert_phase_metrics_unavailable(self, report: dict, summary: Path) -> None:
        self.assertFalse(report["selection_comparison_available"])
        self.assertIsNone(report["coverage_phase_candidates"])
        self.assertIsNone(report["coverage_phase_overlaps"])
        self.assertIn("Coverage phase candidates/overlaps: unavailable.",
                      summary.read_text())

    def assert_invalid_selector(self, output: Path, phase_path: Path,
                                original_phase: str) -> None:
        mutated = json.loads(original_phase)
        mutated["requested_argv"] = [
            "cargo", "llvm-cov", "--no-report", "--no-cfg-coverage",
            "--workspace", "--lib", "--wrong"
        ]
        mutated["executed_argv"] = cargo_command(mutated["requested_argv"], "coverage")
        phase_path.write_text(json.dumps(mutated) + "\n", encoding="utf-8")
        summary = output / "invalid-summary.md"
        result = _run(output, summary)
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads((output / "comparison.json").read_text())
        self.assert_phase_metrics_unavailable(report, summary)

    def assert_missing_phase(self, output: Path, phase_path: Path,
                             original_phase: str) -> None:
        phase_path.write_text(original_phase, encoding="utf-8")
        phase_path.unlink()
        summary = output / "missing-summary.md"
        result = _run(output, summary)
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads((output / "comparison.json").read_text())
        self.assertFalse(report["coverage_phase_inputs_complete"])
        self.assert_phase_metrics_unavailable(report, summary)

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
            self.assert_real_timing(report, summary)
            self.assertTrue(report["selection_comparison_available"])
            self.assertEqual(len(report["normal_only_candidates"]), 1)
            self.assertEqual(report["normal_only_candidates"][0][1], "fixture")

            phase_path = output / "coverage/workspace-lib-bin.json"
            original_phase = phase_path.read_text(encoding="utf-8")
            self.assert_invalid_selector(output, phase_path, original_phase)
            self.assert_missing_phase(output, phase_path, original_phase)
            self.assert_fresh_scope_is_unavailable(output)

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
