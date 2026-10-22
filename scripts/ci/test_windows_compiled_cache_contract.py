"""Keep compiled Windows artifacts separate from test-result authority."""

from pathlib import Path
import unittest

from test_ci_resource_contract import current_test_job, step_block


class WindowsCompiledCacheContractTests(unittest.TestCase):
    def test_exact_identity_precedes_compiled_cache(self):
        job = current_test_job()
        identity = step_block(job, "Identify Windows compiled target")
        self.assertIn("runner.os == 'Windows'", identity)
        self.assertIn("id: windows_target_identity", identity)
        self.assertIn("windows_target_cache_identity.py", identity)
        self.assertIn('--github-output "$GITHUB_OUTPUT"', identity)
        self.assertIn("--profile test --selection ci-test", identity)
        cache = step_block(job, "Cache Windows compiled target")
        self.assertIn("id: windows_target_cache", cache)
        self.assertIn("runner.os == 'Windows'", cache)
        self.assertIn("steps.windows_target_identity.outputs.cache_eligible == 'true'", cache)
        self.assertIn("actions/cache@55cc8345863c7cc4c66a329aec7e433d2d1c52a9", cache)
        self.assertIn("key: ${{ steps.windows_target_identity.outputs.cache_key }}", cache)
        self.assertIn("            target/debug\n", cache)
        self.assertIn("            target/.rustc_info.json", cache)
        for unsafe in ("restore-keys:", "target/test-inventory", "continue-on-error"):
            self.assertNotIn(unsafe, cache)
        self.assertLess(job.index("Identify Windows compiled target"),
                        job.index("Cache Windows compiled target"))

    def test_hits_do_not_skip_original_tests_or_gain_reuse_authority(self):
        job = current_test_job()
        tests = step_block(job, "Run tests")
        self.assertNotIn("if:", tests)
        self.assertEqual(tests.count("bash scripts/ci/with-office-test-worker.sh 'cargo test --workspace'"), 2)
        for option in ("--compiled-cache-hit", "--compiled-cache-key", "--compiled-cache-eligible"):
            self.assertIn(option, tests)
        self.assertIn("steps.windows_target_cache.outputs.cache-hit", tests)
        self.assertIn("steps.windows_target_identity.outputs.cache_key", tests)
        gate = step_block(job, "Verify CI resource contract")
        self.assertIn("test_windows_compiled_cache_contract.py", gate)
        self.assertIn("test_windows_target_cache_identity.py", gate)
        workflow = (Path(__file__).resolve().parents[2] / ".github/workflows/test-and-build.yml").read_text()
        self.assertIn("!github.event.pull_request.draft", workflow)


if __name__ == "__main__":
    unittest.main()
