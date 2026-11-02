"""Compare observed invocation diagnostics without granting test reuse."""

import copy
import hashlib
import importlib
import importlib.util
import unittest

from native_test_run_recording import cargo_command
from native_test_run_identity import INPUT_ENVIRONMENT


def listing(names: tuple[str, ...]) -> dict:
    raw = "".join(f"{name}: test\n" for name in names)
    raw += f"{len(names)} tests, 0 benchmarks\n"
    return {"argv": ["/fixture/test", "--list"], "raw": raw,
            "raw_sha256": hashlib.sha256(raw.encode()).hexdigest(),
            "cases": [{"name": name, "kind": "test"} for name in names],
            "footer": [len(names), 0]}


def manifest(kind: str = "test", options: tuple[str, ...] = ()) -> dict:
    artifact = {"package_id": "path+file:///fixture#fixture@0.1.0",
                "manifest_path": "/fixture/Cargo.toml", "features": [],
                "target": {"name": "fixture", "kind": ["lib"], "crate_types": ["lib"],
                           "src_path": "/fixture/src/lib.rs", "doctest": True},
                "executable": "/fixture/test", "fresh": False}
    source = {"head": "a" * 40, "tree": "b" * 40, "clean": True,
              "status_sha256": hashlib.sha256(b"").hexdigest(),
              "locks": {"Cargo.lock": "c" * 64, "scripts/screenshot/Cargo.lock": "f" * 64}}
    native = {"artifact": copy.deepcopy(artifact), "binary_sha256_at_listing": "d" * 64,
              "all": listing(("alpha", "beta", "ignored")), "ignored": listing(("ignored",))}
    native["ignored"]["argv"].append("--ignored")
    command = (["cargo", "test", "--workspace"] if kind == "test" else
               ["cargo", "llvm-cov", "--no-report", "--no-cfg-coverage", "--workspace", "--lib"])
    command += ["--", *options]
    return {"schema": 1, "run_id": kind, "kind": kind, "exit_code": 0,
            "source_before": source, "source_after": copy.deepcopy(source),
            "source_stable": True, "complete": False, "reuse_authorized": False,
            "inputs": {"os": "macOS", "machine": "arm64", "rustc": "rustc fixture",
                       "cargo": "cargo fixture", "environment": {
                           key: "" if key == "RUSTFLAGS" else None for key in INPUT_ENVIRONMENT},
                       "worker_sha256": "e" * 64},
            "requested_argv": command, "executed_argv": cargo_command(command, kind),
            "cargo": {"native_artifacts_complete": True, "reasons": [],
                      "artifacts": [artifact]}, "native_listings": [native],
            "listing_reasons": [], "limitations": ["doctest inventory not captured"]}


class RunComparisonTests(unittest.TestCase):
    def compare(self, normal: dict | None, coverage: list[dict]) -> dict:
        self.assertIsNotNone(importlib.util.find_spec("native_test_run_comparison"),
                             "invocation comparison is not implemented")
        return importlib.import_module("native_test_run_comparison").compare_runs(normal, coverage)

    def test_matching_candidates_never_authorize_execution_equivalence(self) -> None:
        result = self.compare(manifest(), [manifest("coverage")])
        self.assertTrue(result["selection_comparison_available"])
        self.assertEqual(len(result["common_candidates"]), 2)
        self.assertEqual(result["normal_only_candidates"], [])
        self.assertEqual(result["coverage_only_candidates"], [])
        self.assertFalse(result["execution_equivalent"])
        self.assertFalse(result["reuse_authorized"])
        self.assertFalse(result["inventory_complete"])
        self.assertEqual(len(result["inputs_sha256"]), 64)
        self.assertIn("doctest inventory not captured", result["reasons"])

    def test_skip_and_ignored_selection_are_not_counted_as_execution(self) -> None:
        normal = manifest(options=("--test-threads=2",))
        coverage = manifest("coverage", ("--skip", "beta"))
        result = self.compare(normal, [coverage])
        self.assertEqual(len(result["common_candidates"]), 1)
        self.assertEqual(len(result["normal_only_candidates"]), 1)
        ignored = self.compare(normal, [manifest("coverage", ("--ignored",))])
        self.assertEqual(len(ignored["common_candidates"]), 0)
        self.assertEqual(len(ignored["coverage_only_candidates"]), 1)

    def test_coverage_phase_union_preserves_missing_normal_cases(self) -> None:
        first = manifest("coverage", ("--skip", "beta"))
        second = manifest("coverage", ("--skip", "alpha"))
        second["run_id"] = "second-phase"
        result = self.compare(manifest(), [first, second])
        self.assertEqual(len(result["common_candidates"]), 2)
        self.assertEqual(result["normal_only_candidates"], [])
        self.assertEqual(len(result["coverage_runs"]), 2)
        self.assertEqual(result["coverage_phase_candidates"], [
            {"run_id": "coverage", "candidate_count": 1},
            {"run_id": "second-phase", "candidate_count": 1},
        ])
        self.assertEqual(result["coverage_phase_overlaps"], [{
            "left_run_id": "coverage", "right_run_id": "second-phase", "intersection_count": 0,
        }])
        self.assertIn("not execution count", result["coverage_phase_metrics_scope"])

    def test_three_phase_metrics_report_positive_overlap_and_disjoint_pair(self) -> None:
        phases = [manifest("coverage"), manifest("coverage", ("--skip", "beta")),
                  manifest("coverage", ("--skip", "alpha"))]
        for index, phase in enumerate(phases):
            phase["run_id"] = f"phase-{index}"
        result = self.compare(manifest(), phases)
        self.assertEqual([item["candidate_count"] for item in result["coverage_phase_candidates"]], [2, 1, 1])
        self.assertEqual([item["intersection_count"] for item in result["coverage_phase_overlaps"]], [1, 1, 0])
        self.assertEqual(len(result["common_candidates"]), 2)

    def test_unknown_selection_refuses_a_complete_candidate_comparison(self) -> None:
        for options in (("--future-selection",), ("--skip",), ("alpha", "--exact"),
                        ("--ignored", "--include-ignored")):
            with self.subTest(options=options):
                result = self.compare(manifest(), [manifest("coverage", options)])
                self.assertFalse(result["selection_comparison_available"])
                self.assertTrue(result["reasons"])
                self.assertFalse(result["reuse_authorized"])

    def test_raw_forgery_and_missing_or_duplicate_artifacts_are_rejected(self) -> None:
        for mutation in ("hash", "cases", "missing", "duplicate"):
            with self.subTest(mutation=mutation):
                damaged = manifest("coverage")
                native = damaged["native_listings"][0]
                if mutation == "hash":
                    native["all"]["raw_sha256"] = "f" * 64
                elif mutation == "cases":
                    native["all"]["cases"] = []
                elif mutation == "missing":
                    damaged["native_listings"] = []
                else:
                    damaged["native_listings"].append(copy.deepcopy(native))
                result = self.compare(manifest(), [damaged])
                self.assertFalse(result["selection_comparison_available"])
                self.assertFalse(result["reuse_authorized"])

    def test_source_environment_features_and_failure_remain_visible(self) -> None:
        changed = manifest("coverage")
        changed["inputs"]["environment"]["RUSTFLAGS"] = "--cfg coverage"
        changed["native_listings"][0]["artifact"]["features"] = ["additional"]
        changed["cargo"]["artifacts"][0]["features"] = ["additional"]
        changed["exit_code"] = 101
        result = self.compare(manifest(), [changed])
        self.assertTrue(result["condition_differences"])
        self.assertTrue(result["artifact_differences"])
        self.assertTrue(any("failed" in reason for reason in result["reasons"]))
        self.assertFalse(result["execution_equivalent"])
        changed["source_after"]["head"] = "f" * 40
        unstable = self.compare(manifest(), [changed])
        self.assertFalse(unstable["selection_comparison_available"])

    def test_missing_inputs_do_not_look_like_empty_equal_test_sets(self) -> None:
        for normal, coverage in ((None, []), (manifest(), []), (None, [manifest("coverage")])):
            with self.subTest(normal=normal is not None, count=len(coverage)):
                result = self.compare(normal, coverage)
                self.assertFalse(result["selection_comparison_available"])
                self.assertFalse(result["execution_equivalent"])
                self.assertFalse(result["reuse_authorized"])

    def test_dimensions_report_unknown_and_observed_differences_without_reuse(self):
        normal = manifest()
        coverage = manifest("coverage", ("--skip", "fixture"))
        coverage["inputs"]["environment"]["RUST_TEST_THREADS"] = "1"
        result = self.compare(normal, [coverage])
        dimensions = result["dimensions"]
        self.assertEqual(set(dimensions), {"doctest", "ignored", "fixture", "serial", "environment"})
        self.assertEqual(dimensions["doctest"]["status"], "unknown")
        self.assertEqual(dimensions["ignored"]["status"], "unknown")
        for key in ("fixture", "serial", "environment"):
            self.assertEqual(dimensions[key]["status"], "different")
        self.assertTrue(all(value["evidence"] for value in dimensions.values()))
        self.assertFalse(result["inventory_complete"])
        self.assertFalse(result["reuse_authorized"])


if __name__ == "__main__":
    unittest.main()
