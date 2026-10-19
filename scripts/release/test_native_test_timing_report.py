"""Timing diagnostics must not manufacture speedup or test reuse evidence."""

import copy
import importlib.util
import math
from pathlib import Path
import unittest

from native_test_timing_report import timing_report


def observed(run_id: str = "normal", status: int = 0) -> dict:
    return {"run_id": run_id, "exit_code": status, "invocation_timing": {
        "schema": 1, "clock": "perf_counter", "command": {
            "started_at": "2026-10-19T10:00:00+00:00",
            "finished_at": "2026-10-19T10:00:03+00:00", "elapsed_seconds": 2.5},
        "diagnostics": {"elapsed_seconds": 0.25}}}


class TimingReportTests(unittest.TestCase):
    def test_reports_measured_command_and_diagnostics_without_a_speedup_claim(self):
        result = timing_report(observed(), [observed("coverage")], True)
        self.assertEqual(result["normal"]["command_seconds"], 2.5)
        self.assertEqual(result["coverage"][0]["diagnostics_seconds"], 0.25)
        self.assertEqual(result["coverage_command_seconds_sum"], 2.5)
        self.assertFalse(result["reuse_authorized"])
        self.assertNotIn("speedup", result)

    def test_failed_invocation_retains_status_and_measured_time(self):
        result = timing_report(observed(status=101), [], False)
        self.assertEqual(result["normal"]["exit_code"], 101)
        self.assertEqual(result["normal"]["command_seconds"], 2.5)
        self.assertIsNone(result["coverage_command_seconds_sum"])

    def test_missing_or_malformed_measurement_is_not_zero(self):
        for value in (None, {}, {"schema": True}, {"schema": 2}):
            run = observed()
            run["invocation_timing"] = value
            result = timing_report(run, [], False)
            self.assertIsNone(result["normal"])
            self.assertTrue(result["unavailable_reasons"])

    def test_boolean_negative_and_nonfinite_durations_are_rejected(self):
        for key in ("command", "diagnostics"):
            for value in (True, -1, math.inf, math.nan, "2.5"):
                run = observed()
                run["invocation_timing"][key]["elapsed_seconds"] = value
                self.assertIsNone(timing_report(run, [], False)["normal"])

    def test_timestamps_must_be_utc_but_wall_clock_jump_is_not_duration(self):
        run = observed()
        run["invocation_timing"]["command"]["finished_at"] = "2026-10-19T09:59:00Z"
        self.assertEqual(timing_report(run, [], False)["normal"]["command_seconds"], 2.5)
        for value in ("yesterday", "2026-10-19T10:00:00", "2026-10-19T10:00:00+09:00"):
            run["invocation_timing"]["command"]["started_at"] = value
            self.assertIsNone(timing_report(run, [], False)["normal"])

    def test_incomplete_duplicate_or_overflowing_phases_have_no_total(self):
        phase = observed("coverage")
        self.assertIsNone(timing_report(observed(), [phase], False)[
            "coverage_command_seconds_sum"])
        self.assertIsNone(timing_report(observed(), [phase, phase], True)[
            "coverage_command_seconds_sum"])
        large = copy.deepcopy(phase)
        large["invocation_timing"]["command"]["elapsed_seconds"] = 1e308
        other = copy.deepcopy(large)
        other["run_id"] = "other"
        self.assertIsNone(timing_report(observed(), [large, other], True)[
            "coverage_command_seconds_sum"])

    def test_existing_cli_publishes_timing_without_replacing_candidate_decision(self):
        source = Path(__file__).with_name("compare-test-invocations.py")
        spec = importlib.util.spec_from_file_location("compare_timing_test", source)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        report = {"selection_comparison_available": False, "common_candidates": [],
                  "normal_only_candidates": [], "coverage_only_candidates": [], "reasons": [],
                  "invocation_timing": timing_report(observed(), [], False)}
        summary = module._summary(report)
        self.assertIn("Command seconds", summary)
        self.assertIn("2.500000", summary)
        self.assertIn("not measured", summary)
        self.assertIn("Test skip/reuse authorization: false", summary)


if __name__ == "__main__":
    unittest.main()
