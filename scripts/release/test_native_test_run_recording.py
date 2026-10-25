"""Execute real Cargo/libtest in isolated native fixtures, without mocks."""

import hashlib
import json
import os
from contextlib import nullcontext
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from native_test_run_recording import cargo_command
from test_native_fixture_build_cache import prepared_fixture
import test_native_test_run_identity as identity_fixtures

SCRIPT = Path(__file__).with_name("record-test-invocation.py").resolve()


class RunRecordingTests(unittest.TestCase):
    def assert_invocation_timing(self, manifest: dict) -> None:
        timing = manifest["invocation_timing"]
        self.assertEqual(timing["schema"], 1)
        self.assertEqual(timing["clock"], "perf_counter")
        for phase in ("command", "diagnostics"):
            self.assertGreaterEqual(timing[phase]["elapsed_seconds"], 0.0)
        self.assertIsInstance(timing["command"]["started_at"], str)
        self.assertIsInstance(timing["command"]["finished_at"], str)

    def environment(self, root: Path) -> dict:
        result = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        result.update({"CARGO_TARGET_DIR": str(root / "target"), "CARGO_BUILD_JOBS": "1"})
        return result

    def fixture(self, root: Path, failing: bool) -> None:
        (root / "src").mkdir()
        (root / "Cargo.toml").write_text(
            '[package]\nname="inventory-native-fixture"\nversion="0.1.0"\nedition="2021"\n',
            encoding="utf-8")
        assertion = "assert_eq!(1, 2);" if failing else "assert_eq!(answer(), 2);"
        (root / "src/lib.rs").write_text(
            '/// ```\n/// assert_eq!(2, 2);\n/// ```\npub fn answer() -> u32 { 2 }\n'
            '#[cfg(test)] mod tests { use super::*;\n'
            '#[test] fn alpha() { println!("{}", '
            'r#"{"reason":"compiler-artifact","profile":{"test":true}}"#); '
            f'{assertion} }}\n'
            '#[test] #[ignore] fn ignored() { assert_eq!(answer(), 2); }\n}\n', encoding="utf-8")
        identity_fixtures.RunIdentityTests().repository(root)
        (root / ".gitignore").write_text("/target\n", encoding="utf-8")
        environment = self.environment(root)
        subprocess.run(["cargo", "generate-lockfile", "--offline"], cwd=root,
                       env=environment, check=True, capture_output=True)
        subprocess.run(["git", "add", "."], cwd=root, env=environment, check=True)
        subprocess.run(["git", "-c", "commit.gpgsign=false", "commit", "-qm", "Cargo fixture"],
                       cwd=root, env=environment, check=True)

    def run_fixture(self, failing: bool, prepared: bool = False,
                    cache_key: str = "pass-normal") -> tuple:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            if prepared:
                roots = prepared_fixture(cache_key, lambda prepared_root: self.fixture(prepared_root, False))
            else:
                self.fixture(root, failing)
                roots = nullcontext(root)
            with roots as root:
                output = root / "target/inventory/normal.json"
                result = subprocess.run(
                    [sys.executable, "-B", str(SCRIPT), "--output", str(output), "--kind", "test",
                     "--", "cargo", "test", "--workspace", "--offline", "--", "--nocapture"],
                    cwd=root, env=self.environment(root), capture_output=True, timeout=60)
                self.assertTrue(output.is_file(), result.stderr.decode(errors="replace"))
                return result, json.loads(output.read_text(encoding="utf-8"))

    def test_actual_cargo_records_native_and_ignored_without_claiming_doctests(self) -> None:
        result, manifest = self.run_fixture(False, prepared=True)
        self.assertEqual(result.returncode, 0, result.stderr.decode(errors="replace"))
        self.assertEqual(manifest["exit_code"], 0)
        self.assertTrue(manifest["source_stable"])
        self.assertFalse(manifest["complete"])
        self.assertFalse(manifest["reuse_authorized"])
        self.assertIn("doctest inventory not captured", manifest["limitations"])
        self.assertIn("--message-format=json", manifest["executed_argv"])
        self.assertTrue(manifest["cargo"]["native_artifacts_complete"])
        self.assertEqual(len(manifest["cargo"]["artifacts"]), 1)
        listing, = manifest["native_listings"]
        self.assertEqual([case["name"] for case in listing["all"]["cases"]],
                         ["tests::alpha", "tests::ignored"])
        self.assertEqual([case["name"] for case in listing["ignored"]["cases"]], ["tests::ignored"])
        self.assertEqual(listing["all"]["raw_sha256"],
                         hashlib.sha256(listing["all"]["raw"].encode()).hexdigest())
        self.assertIn(b"test result: ok", result.stdout)
        self.assertEqual(manifest["listing_reasons"], [])
        self.assert_invocation_timing(manifest)

    def test_prepared_fixture_rebuilds_tests_while_reusing_only_cargo_artifacts(self) -> None:
        first, first_manifest = self.run_fixture(False, prepared=True, cache_key="pass-normal-probe")
        second, second_manifest = self.run_fixture(False, prepared=True, cache_key="pass-normal-probe")
        self.assertEqual(first.returncode, 0, first.stderr.decode(errors="replace"))
        self.assertEqual(second.returncode, 0, second.stderr.decode(errors="replace"))
        first_artifact, = first_manifest["cargo"]["artifacts"]
        second_artifact, = second_manifest["cargo"]["artifacts"]
        self.assertFalse(first_artifact["fresh"])
        self.assertTrue(second_artifact["fresh"])
        self.assertNotEqual(first_manifest["run_id"], second_manifest["run_id"])
        self.assertEqual(first_manifest["native_listings"][0]["all"]["cases"],
                         second_manifest["native_listings"][0]["all"]["cases"])

    def test_actual_cargo_failure_cannot_be_replaced_by_successful_listing(self) -> None:
        result, manifest = self.run_fixture(True)
        self.assertEqual(result.returncode, 101, result.stderr.decode(errors="replace"))
        self.assertEqual(manifest["exit_code"], 101)
        self.assertFalse(manifest["complete"])
        self.assertFalse(manifest["reuse_authorized"])
        self.assertIn(b"test result: FAILED", result.stdout)
        self.assert_invocation_timing(manifest)

    def test_missing_rustc_identity_keeps_real_cargo_failure(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.fixture(root, False)
            output = root / "target/inventory/missing-rustc.json"
            environment = self.environment(root)
            environment["RUSTC"] = str(root / "missing-rustc")
            result = subprocess.run(
                [sys.executable, "-B", str(SCRIPT), "--output", str(output), "--kind", "test",
                 "--", "cargo", "test", "--workspace", "--offline", "--", "--nocapture"],
                cwd=root, env=environment, capture_output=True, timeout=60)
            manifest = json.loads(output.read_text(encoding="utf-8"))
            self.assertEqual(result.returncode, 101)
            self.assertEqual(manifest["exit_code"], 101)
            self.assertIsNone(manifest["inputs"]["rustc"])
            self.assertFalse(manifest["complete"])
            self.assertFalse(manifest["reuse_authorized"])

    def test_command_preserves_selection_and_rejects_non_execution(self) -> None:
        arguments = ["cargo", "test", "--workspace", "--", "--test-threads=1", "--skip", "fixture"]
        observed = cargo_command(arguments, "test")
        self.assertEqual(observed, arguments[:3] + ["--message-format=json"] + arguments[3:])
        for invalid in (["cargo", "test", "--no-run"], ["cargo", "test", "--", "--list"],
                        ["cargo", "test", "--message-format=short"], ["bash", "-c", "cargo test"]):
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                cargo_command(invalid, "test")
        coverage = ["cargo", "llvm-cov", "--no-cfg-coverage", "--no-report", "--lib",
                    "--", "--test-threads=1"]
        self.assertEqual(cargo_command(coverage, "coverage"),
                         ["cargo", "test", "--lib", "--message-format=json",
                          "--", "--test-threads=1"])


if __name__ == "__main__":
    unittest.main()
