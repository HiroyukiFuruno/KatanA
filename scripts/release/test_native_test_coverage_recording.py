"""Exercise official show-env instrumentation with real Cargo tests and profiles."""

import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from test_native_fixture_build_cache import prepared_fixture
import test_native_test_run_recording as fixtures

SCRIPT = Path(__file__).with_name("record-test-invocation.py").resolve()


def prepare_standard_fixture(root: Path) -> None:
    fixtures.RunRecordingTests().fixture(root, False)


class CoverageRecordingTests(unittest.TestCase):
    def environment(self, root: Path) -> dict[str, str]:
        environment = fixtures.RunRecordingTests().environment(root)
        environment.update({"CARGO_LLVM_COV_TARGET_DIR": str(root / "target/coverage"),
                            "CARGO_LLVM_COV_BUILD_DIR": str(root / "target/coverage")})
        environment["RUSTFLAGS"] = environment.get("RUSTFLAGS", "") + " --cfg coverage"
        return environment

    def execute(self, root: Path, failing: bool, custom_target: bool = True,
                separate_build: bool = False, configured_build: bool = False,
                prepared: bool = False) -> tuple:
        if not prepared:
            fixtures.RunRecordingTests().fixture(root, failing)
        output = root / "target/inventory/coverage.json"
        requested = ["cargo", "llvm-cov", "--no-cfg-coverage", "--no-report", "--offline",
                     "--lib", "--", "--test-threads=1", "--skip", "not_present"]
        environment = self.environment(root)
        if separate_build:
            environment["CARGO_LLVM_COV_BUILD_DIR"] = str(root / "target/build space")
        if configured_build:
            environment["CARGO_BUILD_BUILD_DIR"] = str(root / "target/configured build")
        if not custom_target:
            environment.pop("CARGO_LLVM_COV_TARGET_DIR")
            environment.pop("CARGO_LLVM_COV_BUILD_DIR")
        result = subprocess.run(
            [sys.executable, "-B", str(SCRIPT), "--output", str(output), "--kind", "coverage",
             "--", *requested], cwd=root, env=environment,
            capture_output=True, timeout=60)
        self.assertTrue(output.is_file(), result.stderr.decode(errors="replace"))
        manifest = json.loads(output.read_text())
        self.assertEqual(manifest["requested_argv"], requested)
        return result, manifest

    def test_separate_build_directory_preserves_original_report_contract(self) -> None:
        with prepared_fixture("pass-coverage-separate", prepare_standard_fixture) as root:
            result, manifest = self.execute(root, False, separate_build=True, prepared=True)
            self.assertEqual(result.returncode, 0, result.stderr.decode(errors="replace"))
            self.assertTrue(manifest["cargo"]["native_artifacts_complete"])
            self.assertTrue(manifest["native_listings"])
            self.report(root, manifest, separate_build=True)

    def test_configured_separate_build_directory_produces_original_report(self) -> None:
        with prepared_fixture("pass-coverage-configured", prepare_standard_fixture) as root:
            result, manifest = self.execute(root, False, separate_build=True,
                                            configured_build=True, prepared=True)
            self.assertEqual(result.returncode, 0, result.stderr.decode(errors="replace"))
            self.assertTrue(manifest["cargo"]["native_artifacts_complete"])
            self.assertEqual(manifest["coverage"]["environment"]["CARGO_BUILD_BUILD_DIR"],
                             str(root / "target/build space"))
            self.report(root, manifest, separate_build=True, configured_build=True)

    def test_default_target_warm_repeat_keeps_profiles_and_report_together(self) -> None:
        with prepared_fixture("pass-coverage-default", prepare_standard_fixture) as root:
            result, first = self.execute(root, False, custom_target=False, prepared=True)
            self.assertEqual(result.returncode, 0, result.stderr.decode(errors="replace"))
            self.assertEqual(first["coverage"]["environment"]["CARGO_TARGET_DIR"],
                             str(root / "target/llvm-cov-target"))
            self.report(root, first)
            environment = self.environment(root)
            environment.pop("CARGO_LLVM_COV_TARGET_DIR")
            environment.pop("CARGO_LLVM_COV_BUILD_DIR")
            output = root / "target/inventory/repeat.json"
            result = subprocess.run(
                [sys.executable, "-B", str(SCRIPT), "--output", str(output),
                 "--kind", "coverage", "--", *first["requested_argv"]],
                cwd=root, env=environment, capture_output=True, timeout=60)
            self.assertEqual(result.returncode, 0, result.stderr.decode(errors="replace"))
            repeated = json.loads(output.read_text())
            self.assertTrue(repeated["cargo"]["native_artifacts_complete"])
            previous, = first["cargo"]["artifacts"]
            current, = repeated["cargo"]["artifacts"]
            self.assertFalse(previous["fresh"])
            self.assertTrue(current["fresh"])
            self.assertEqual(current["executable"], previous["executable"])
            self.report(root, repeated)

    def test_real_instrumented_test_records_artifacts_and_produces_report(self) -> None:
        with prepared_fixture("pass-coverage-standard", prepare_standard_fixture) as root:
            result, manifest = self.execute(root, False, prepared=True)
            self.assertEqual(result.returncode, 0, result.stderr.decode(errors="replace"))
            self.assertTrue(manifest["cargo"]["native_artifacts_complete"])
            self.assertEqual(manifest["executed_argv"][:2], ["cargo", "test"])
            self.assertNotIn("--target-dir", manifest["executed_argv"])
            listing, = manifest["native_listings"]
            self.assertEqual([case["name"] for case in listing["all"]["cases"]],
                             ["tests::alpha", "tests::ignored"])
            self.assertEqual([case["name"] for case in listing["ignored"]["cases"]], ["tests::ignored"])
            self.assertTrue(manifest["source_stable"])
            self.assertFalse(manifest["complete"])
            self.assertFalse(manifest["reuse_authorized"])
            self.report(root, manifest)

    def report(self, root: Path, manifest: dict, separate_build: bool = False,
               configured_build: bool = False) -> None:
        environment = self.environment(root)
        if separate_build:
            environment["CARGO_LLVM_COV_BUILD_DIR"] = str(root / "target/build space")
        if configured_build:
            environment["CARGO_BUILD_BUILD_DIR"] = str(root / "target/configured build")
        recorded = manifest["coverage"]["environment"]
        if recorded["CARGO_TARGET_DIR"] == str(root / "target/llvm-cov-target"):
            environment.pop("CARGO_LLVM_COV_TARGET_DIR")
            environment.pop("CARGO_LLVM_COV_BUILD_DIR")
        profile_root = Path(recorded["LLVM_PROFILE_FILE"]).parent
        self.assertTrue(list(profile_root.glob("*.profraw")))
        self.assertEqual(list(root.glob("*.profraw")), [])
        output = root / "target/report.json"
        result = subprocess.run(
            ["cargo", "llvm-cov", "report", "--json", "--output-path", str(output)],
            cwd=root, env=environment, capture_output=True, timeout=60)
        self.assertEqual(result.returncode, 0, result.stderr.decode(errors="replace"))
        self.assertGreater(json.loads(output.read_text())["data"][0]["totals"]["lines"]["covered"], 0)

    def test_real_test_failure_preserves_status_despite_successful_inventory(self) -> None:
        with tempfile.TemporaryDirectory(prefix="katana coverage failure ") as directory:
            result, manifest = self.execute(Path(directory), True)
            self.assertEqual(result.returncode, 101, result.stderr.decode(errors="replace"))
            self.assertEqual(manifest["exit_code"], 101)
            self.assertTrue(manifest["cargo"]["native_artifacts_complete"])
            self.assertTrue(manifest["native_listings"])
            self.assertFalse(manifest["complete"])
            self.assertFalse(manifest["reuse_authorized"])


if __name__ == "__main__":
    unittest.main()
