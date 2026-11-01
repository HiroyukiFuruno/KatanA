"""Keep Windows cache observations separate from quality/reuse decisions."""

from pathlib import Path
import tempfile
import unittest

from test_ci_resource_contract import current_test_job, step_block
from action_pin_fixture import PIN_CASES, action_pin_variants, prepare_pin_fixture, run_pin_contract


class WindowsCiObservationContractTest(unittest.TestCase):
    def test_windows_inventory_has_supported_python_and_real_pipe_regressions(self):
        job = current_test_job()
        setup = step_block(job, "Install Python for Windows inventory")
        self.assertIn("runner.os == 'Windows'", setup)
        self.assertRegex(
            setup,
            r"(?m)^        uses:[ \t]+actions/setup-python@[0-9a-f]{40}(?:[ \t]+#.*)?[ \t]*$",
        )
        self.assertIn("python-version: '3.14'", setup)
        tests = step_block(job, "Verify native inventory pipes (Windows)")
        self.assertIn("runner.os == 'Windows'", tests)
        self.assertIn("test_native_test_pipe_reader.py", tests)
        self.assertIn("test_native_test_run_recording.py", tests)
        self.assertNotIn("continue-on-error", tests)
        self.assertLess(job.index("Install Python for Windows inventory"),
                        job.index("Verify native inventory pipes (Windows)"))

    def test_registry_cache_has_an_observable_identity_without_target_cache(self):
        cache = step_block(current_test_job(), "Cache Windows Cargo registry")
        self.assertIn("id: windows_registry_cache", cache)
        self.assertIn("runner.os == 'Windows'", cache)
        self.assertIn("cargo-registry-v1-", cache)
        self.assertNotIn("restore-keys:", cache)
        self.assertNotIn("            target", cache)

    def test_original_workspace_invocation_runs_once_on_each_platform(self):
        step = step_block(current_test_job(), "Run tests")
        self.assertIn("WINDOWS_REGISTRY_CACHE_HIT:", step)
        self.assertIn("steps.windows_registry_cache.outputs.cache-hit", step)
        self.assertIn('if [[ "${RUNNER_OS}" == "Windows" ]]; then', step)
        self.assertIn("python3 -B scripts/ci/observe-ci-command.py", step)
        self.assertIn('"${KATANA_TEST_INVENTORY_ROOT}/windows-timing.json"', step)
        original = "bash scripts/ci/with-office-test-worker.sh 'cargo test --workspace'"
        self.assertEqual(step.count(original), 1)
        self.assertIn('BASH_EXECUTABLE="$(cygpath -w "$BASH")"', step)
        self.assertIn(
            '-- "$BASH_EXECUTABLE" scripts/ci/with-office-test-worker.sh '
            "'cargo test --workspace'",
            step,
        )
        self.assertIn("else", step)
        self.assertNotIn("continue-on-error", step)
        self.assertNotIn("--skip", step)
        self.assertNotIn("--exclude", step)

    def test_observations_are_uploaded_and_regressions_remain_in_the_gate(self):
        job = current_test_job()
        upload = step_block(job, "Upload observed test inventory diagnostics")
        self.assertIn("always()", upload)
        self.assertIn("path: ${{ env.KATANA_TEST_INVENTORY_ROOT }}", upload)
        tests = step_block(job, "Verify CI resource contract")
        for filename in (
            "test_observe_ci_command.py",
            "test_windows_ci_observation_contract.py",
        ):
            self.assertIn(filename, tests)
        workflow = (Path(__file__).resolve().parents[2]
                    / ".github/workflows/test-and-build.yml").read_text(encoding="utf-8")
        self.assertIn("!github.event.pull_request.draft", workflow)

    def test_action_pins_allow_rotation_and_reject_mutable_or_spoofed_refs(self):
        root = Path(__file__).resolve().parents[2]
        workflow = (root / ".github/workflows/test-and-build.yml").read_text(encoding="utf-8")
        with tempfile.TemporaryDirectory(prefix="katana-action-pins-") as temporary:
            fixture = Path(temporary)
            target = prepare_pin_fixture(root, fixture)
            for case in PIN_CASES:
                self.assert_pin_case(fixture, target, workflow, case)

    def assert_pin_case(self, fixture, target, workflow, case):
        name, action, revisions, script, test, requirements = case
        original = step_block(current_test_job(), name)
        for label, step, accepted in action_pin_variants(original, action, revisions, requirements):
            with self.subTest(action=action, variant=label):
                target.write_text(workflow.replace(original, step, 1), encoding="utf-8")
                result = run_pin_contract(fixture, script, test)
                self.assertEqual(result.returncode == 0, accepted, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
