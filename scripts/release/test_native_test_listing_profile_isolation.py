"""Keep real instrumented inventory processes out of the measured test profiles."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from native_test_artifact_listing import list_artifacts
from test_native_fixture_build_cache import prepared_fixture
import test_native_test_coverage_recording as coverage_fixtures
import test_native_test_run_recording as run_fixtures


def profile_snapshot(root: Path) -> dict[str, str]:
    return {str(path.relative_to(root)): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in root.rglob("*.profraw")}


class ListingProfileIsolationTests(unittest.TestCase):
    def test_unsafe_temporary_root_cannot_mask_real_test_failure(self) -> None:
        with tempfile.TemporaryDirectory(prefix="katana listing unsafe ") as directory:
            root = Path(directory).resolve()
            run_fixtures.RunRecordingTests().fixture(root, True)
            environment = coverage_fixtures.CoverageRecordingTests().environment(root)
            temporary = root / "target/coverage/temporary"
            temporary.mkdir(parents=True)
            environment.update({"TMPDIR": str(temporary), "TMP": str(temporary),
                                "TEMP": str(temporary)})
            output = root / "target/inventory/unsafe.json"
            result = subprocess.run(
                [sys.executable, "-B", str(coverage_fixtures.SCRIPT), "--output", str(output),
                 "--kind", "coverage", "--", "cargo", "llvm-cov", "--no-report",
                 "--no-cfg-coverage", "--offline", "--lib"], cwd=root, env=environment,
                capture_output=True, timeout=60)
            self.assertEqual(result.returncode, 101, result.stderr.decode(errors="replace"))
            manifest = json.loads(output.read_text())
            self.assertEqual(manifest["exit_code"], 101)
            self.assertEqual(manifest["native_listings"], [])
            self.assertTrue(manifest["listing_reasons"])
            self.assertFalse(manifest["reuse_authorized"])
            profile_root = Path(manifest["coverage"]["environment"]["LLVM_PROFILE_FILE"]).parent
            self.assertEqual(len(profile_snapshot(profile_root)), 1)

    def test_recording_creates_only_the_real_test_process_profile(self) -> None:
        with prepared_fixture("pass-coverage-standard",
                              coverage_fixtures.prepare_standard_fixture) as root:
            result, manifest = coverage_fixtures.CoverageRecordingTests().execute(
                root, False, prepared=True)
            self.assertEqual(result.returncode, 0, result.stderr.decode(errors="replace"))
            profile_root = Path(manifest["coverage"]["environment"]["LLVM_PROFILE_FILE"]).parent
            profiles = profile_snapshot(profile_root)
            self.assertEqual(len(profiles), 1, profiles)
            self.assertTrue(manifest["native_listings"])

    def test_real_listing_leaves_existing_profiles_unchanged_for_both_kinds(self) -> None:
        with prepared_fixture("pass-coverage-standard",
                              coverage_fixtures.prepare_standard_fixture) as root:
            fixture = coverage_fixtures.CoverageRecordingTests()
            result, manifest = fixture.execute(root, False, prepared=True)
            self.assertEqual(result.returncode, 0, result.stderr.decode(errors="replace"))
            environment = fixture.environment(root) | manifest["coverage"]["environment"]
            profile_root = Path(environment["LLVM_PROFILE_FILE"]).parent
            before = profile_snapshot(profile_root)
            self.assertTrue(before)
            for kind in ("test", "coverage"):
                records, reasons = list_artifacts(manifest["cargo"]["artifacts"], root,
                                                  kind, environment)
                self.assertEqual(reasons, [])
                self.assertEqual(len(records), 1)
                self.assertEqual(profile_snapshot(profile_root), before)
            fixture.report(root, manifest)


if __name__ == "__main__":
    unittest.main()
