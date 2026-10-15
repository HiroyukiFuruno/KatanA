"""Malformed observations must never become equal empty candidate collections."""

import copy
import unittest

from native_test_run_comparison import compare_runs
from test_native_test_run_comparison import listing, manifest


class ComparisonBoundaryTests(unittest.TestCase):
    def test_other_source_phase_is_not_current_candidate_set_evidence(self) -> None:
        changed = manifest("coverage")
        changed["source_before"]["head"] = "f" * 40
        changed["source_after"] = copy.deepcopy(changed["source_before"])
        report = compare_runs(manifest(), [changed])
        self.assertFalse(report["selection_comparison_available"])
        self.assertTrue(report["condition_differences"])
        self.assertFalse(report["reuse_authorized"])

    def test_unknown_environment_capture_version_is_unavailable(self) -> None:
        changed = manifest("coverage")
        changed["inputs"]["environment"]["UNKNOWN"] = "x"
        self.unavailable(changed)

    def test_valid_raw_order_changes_are_retained_as_artifact_differences(self) -> None:
        changed = manifest("coverage")
        changed["native_listings"][0]["all"] = listing(("beta", "alpha", "ignored"))
        report = compare_runs(manifest(), [changed])
        self.assertTrue(report["selection_comparison_available"])
        self.assertEqual(len(report["common_candidates"]), 2)
        paths = [path for phase in report["artifact_differences"]
                 for change in phase["changes"] for path in change["paths"]]
        self.assertIn("all_raw_sha256", paths)

    def unavailable(self, damaged: dict) -> None:
        result = compare_runs(manifest(), [damaged])
        self.assertFalse(result["selection_comparison_available"])
        self.assertFalse(result["execution_equivalent"])
        self.assertFalse(result["reuse_authorized"])
        self.assertEqual(result["common_candidates"], [])

    def test_requested_and_executed_argv_are_both_required(self) -> None:
        for value in ([], ["cargo", "test"], ["cargo", "test", "--", "--list"], "cargo test"):
            damaged = manifest("coverage")
            damaged["executed_argv"] = value
            self.unavailable(damaged)

    def test_listing_argv_cannot_swap_all_and_ignored_or_change_executable(self) -> None:
        for key in ("all", "ignored"):
            damaged = manifest("coverage")
            damaged["native_listings"][0][key]["argv"] = ["/different/test", "--list"]
            self.unavailable(damaged)

    def test_missing_identity_and_dirty_or_boolean_status_are_unavailable(self) -> None:
        for mutation in ("missing-lock", "dirty", "bool-exit", "status", "claimed-reuse"):
            damaged = manifest("coverage")
            if mutation == "missing-lock":
                damaged["source_before"]["locks"].pop("Cargo.lock")
            elif mutation == "dirty":
                damaged["source_before"]["clean"] = False
            elif mutation == "bool-exit":
                damaged["exit_code"] = True
            elif mutation == "status":
                damaged["source_before"]["status_sha256"] = "f" * 64
            else:
                damaged["reuse_authorized"] = True
            damaged["source_after"] = copy.deepcopy(damaged["source_before"])
            self.unavailable(damaged)

    def test_malformed_normal_run_id_is_reported_instead_of_raising(self) -> None:
        normal = manifest()
        normal["run_id"] = []
        result = compare_runs(normal, [manifest("coverage")])
        self.assertFalse(result["selection_comparison_available"])
        self.assertFalse(result["reuse_authorized"])

    def test_absent_native_artifacts_are_not_an_equal_empty_test_set(self) -> None:
        damaged = manifest("coverage")
        damaged["cargo"]["artifacts"] = []
        damaged["native_listings"] = []
        self.unavailable(damaged)

    def test_duplicate_run_and_target_with_changed_source_are_unavailable(self) -> None:
        duplicated = manifest("coverage")
        self.assertFalse(compare_runs(manifest(), [duplicated, duplicated])[
            "selection_comparison_available"])
        changed = copy.deepcopy(duplicated["cargo"]["artifacts"][0])
        changed["target"]["src_path"] = "/fixture/src/different.rs"
        duplicated["cargo"]["artifacts"].append(changed)
        self.unavailable(duplicated)

    def test_unknown_conditions_and_bool_footer_do_not_match(self) -> None:
        damaged = manifest("coverage")
        damaged["inputs"]["environment"]["RUSTFLAGS"] = {"unexpected": "nested"}
        self.unavailable(damaged)
        damaged = manifest("coverage")
        damaged["native_listings"][0]["ignored"]["footer"] = [True, 0]
        self.unavailable(damaged)

    def test_include_ignored_adds_candidates_but_never_execution_authority(self) -> None:
        result = compare_runs(manifest(), [manifest("coverage", ("--include-ignored",))])
        self.assertTrue(result["selection_comparison_available"])
        self.assertEqual(len(result["coverage_only_candidates"]), 1)
        self.assertFalse(result["execution_equivalent"])
        self.assertFalse(result["reuse_authorized"])


if __name__ == "__main__":
    unittest.main()
