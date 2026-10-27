"""Keep optional native profiling separate from required product checks."""

import hashlib
from pathlib import Path
import re
import tomllib
import unittest
from unittest import mock

from test_ci_resource_contract import current_test_job, step_block


ROOT = Path(__file__).resolve().parents[2]
FEATURE = "native-profiler-diagnostics"
NATIVE_TEST = (
    "app::action::html_navigation::diagnostics::tests::"
    "sample_captures_owned_child_pid_to_workspace_failure_directory"
)


class NativeProfilerJobContractTest(unittest.TestCase):
    def test_contracts_read_utf8_under_a_legacy_windows_locale(self):
        names = [name for name in dir(type(self)) if name.startswith("test_")
                 and name != "test_contracts_read_utf8_under_a_legacy_windows_locale"]
        self.assertGreaterEqual(len(names), 6)
        suite = unittest.TestSuite(type(self)(name) for name in names)
        result = unittest.TestResult()
        with mock.patch("io.text_encoding", side_effect=lambda encoding, *args: encoding or "cp1252"):
            suite.run(result)
        self.assertEqual(result.testsRun, len(names))
        self.assertTrue(result.wasSuccessful(), result.errors + result.failures)

    def test_native_feature_is_explicit_and_disabled_by_default(self):
        manifest = tomllib.loads((ROOT / "crates/katana-ui/Cargo.toml").read_text(encoding="utf-8"))
        self.assertEqual(manifest["features"][FEATURE], [])
        self.assertNotIn(FEATURE, manifest["features"].get("default", []))
        source = (ROOT / "crates/katana-ui/src/app/action/html_navigation_diagnostics.rs").read_text(encoding="utf-8")
        self.assertRegex(source, rf'cfg\(all\(target_os = "macos", feature = "{FEATURE}"\)\)')
        self.assertNotIn("#[ignore", source)

    def test_original_real_capture_assertions_and_deadline_are_unchanged(self):
        source = (ROOT / "crates/katana-ui/src/app/action/html_navigation_diagnostics.rs").read_text(encoding="utf-8")
        body = source[source.index("    fn sample_captures_owned_child_pid_to_workspace_failure_directory()"):
                      source.rfind("\n}")]
        self.assertEqual(hashlib.sha256(body.encode()).hexdigest(),
                         "62e6065f8f08e475b7988777b040994fc645ceba3a18c0d73f11689d1f01ca12")
        self.assertIn("const SAMPLE_TIMEOUT: Duration = Duration::from_secs(5);", source)

    def test_product_workspace_invocation_is_not_relaxed(self):
        step = step_block(current_test_job(), "Run tests")
        self.assertEqual(hashlib.sha256(step.encode()).hexdigest(),
                         "dc411561cd372a39f277318252e592ec6df946d717dbdff71345fcfd07b0c51b")
        self.assertNotIn(FEATURE, step)
        contracts = step_block(current_test_job(), "Verify CI resource contract")
        self.assertIn("test_native_profiler_job_contract.py", contracts)

    def test_explicit_target_runs_the_same_native_test(self):
        recipes = (ROOT / "just/tests.just").read_text(encoding="utf-8")
        target = re.search(r"(?ms)^test-native-html-diagnostics:\n(.*?)(?=\n\S|\Z)", recipes)
        self.assertIsNotNone(target)
        self.assertIn("Darwin", target.group(1))
        self.assertIn(f"--features {FEATURE}", target.group(1))
        self.assertIn(NATIVE_TEST, target.group(1))
        self.assertIn("-- --exact --nocapture", target.group(1))
        self.assertNotIn("--skip", target.group(1))

    def test_diagnostic_job_reports_its_own_failures_and_artifacts(self):
        workflow = (ROOT / ".github/workflows/html-failure-diagnostics.yml").read_text(encoding="utf-8")
        self.assertIn("name: HTML Failure Diagnostics", workflow)
        self.assertIn("workflow_dispatch:", workflow)
        self.assertIn("!github.event.pull_request.draft", workflow)
        self.assertIn("runs-on: macos-latest", workflow)
        self.assertIn("run: just test-native-html-diagnostics", workflow)
        self.assertIn("if: always()", workflow)
        self.assertIn("path: target/html-startup-failures", workflow)
        self.assertNotIn("continue-on-error", workflow)
        self.assertNotIn("DevToolsSecurity", workflow)
        self.assertNotIn("Test and Build", workflow)

    def test_cached_builds_do_not_restore_old_sample_reports(self):
        workflow = (ROOT / ".github/workflows/html-failure-diagnostics.yml").read_text(encoding="utf-8")
        cache = step_block(workflow, "Cache diagnostic build inputs")
        self.assertIn("            target/debug\n", cache)
        self.assertNotIn("            target\n", cache)
        self.assertNotIn("html-startup-failures", cache)


if __name__ == "__main__":
    unittest.main()
