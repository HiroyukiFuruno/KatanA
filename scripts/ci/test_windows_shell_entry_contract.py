"""Keep native Windows test commands bound to the executing Git Bash."""

import unittest

from test_ci_resource_contract import current_test_job, step_block


class WindowsShellEntryContractTests(unittest.TestCase):
    def test_python_child_receives_the_running_bash_as_a_native_path(self):
        step = step_block(current_test_job(), "Run tests")
        windows = step.split('if [[ "${RUNNER_OS}" == "Windows" ]]; then', 1)[1]
        windows = windows.split("          else", 1)[0]
        self.assertIn('BASH_EXECUTABLE="$(cygpath -w "$BASH")"', windows)
        self.assertIn('-- "$BASH_EXECUTABLE" scripts/ci/with-office-test-worker.sh', windows)
        self.assertLess(windows.index("cygpath -w"), windows.index('-- "$BASH_EXECUTABLE"'))
        self.assertNotIn("-- bash ", windows)

    def test_multi_command_python_gates_use_fail_fast_bash(self):
        job = current_test_job()
        for name in ("Verify CI resource contract", "Verify native inventory pipes (Windows)",
                     "Verify Cargo sweep safety contract"):
            with self.subTest(step=name):
                step = step_block(job, name)
                self.assertIn("shell: bash", step)
                self.assertNotIn("continue-on-error", step)
                self.assertNotIn("set +e", step)

    def test_cargo_guard_receives_the_running_git_bash_and_keeps_all_tests(self):
        step = step_block(current_test_job(), "Verify Cargo sweep safety contract")
        binding = 'export KATANA_GUARD_BASH_EXECUTABLE="$(cygpath -w "$BASH")"'
        command = "python3 -B -m unittest discover -s scripts/ci -p 'test_cargo_*guard.py' -v"
        self.assertIn('if [[ "${RUNNER_OS}" == "Windows" ]]; then', step)
        self.assertIn(binding, step)
        self.assertEqual(step.count(command), 1)
        self.assertLess(step.index(binding), step.index(command))
        self.assertNotIn("--skip", step)
        self.assertNotIn("--exclude", step)

    def test_both_platform_paths_keep_the_original_workspace_test_command(self):
        step = step_block(current_test_job(), "Run tests")
        original = "scripts/ci/with-office-test-worker.sh 'cargo test --workspace'"
        self.assertEqual(step.count(original), 2)
        self.assertNotIn("--skip", step)
        self.assertNotIn("--exclude", step)

    def test_shell_regression_remains_in_the_original_resource_gate(self):
        step = step_block(current_test_job(), "Verify CI resource contract")
        self.assertIn("test_windows_shell_entry_contract.py", step)


if __name__ == "__main__":
    unittest.main()
