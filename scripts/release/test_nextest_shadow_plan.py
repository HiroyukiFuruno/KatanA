from pathlib import Path
import unittest
import tomllib

from nextest_shadow_plan import PHASES, VERSION, phase_plan, shadow_contract


ROOT = Path(__file__).resolve().parents[2]


class NextestShadowPlanTests(unittest.TestCase):
    def test_phase_plan_preserves_selection_and_safety_flags(self):
        for item, selection in zip(phase_plan(ROOT, Path("/tmp/shadow")), PHASES.values()):
            with self.subTest(phase=item["phase"]):
                self.assertEqual(item["argv"][-len(selection):], list(selection))
                self.assertIn("--no-cfg-coverage", item["argv"])
                self.assertIn("--locked", item["argv"])
                self.assertIn("--no-fail-fast", item["argv"])
                self.assertIn("--retries", item["argv"])
                self.assertNotIn("--all-features", item["argv"])

    def test_phase_plan_uses_one_isolated_target(self):
        plans = phase_plan(ROOT, Path("/tmp/shadow"))
        self.assertEqual({item["target_dir"] for item in plans}, {"/tmp/shadow/coverage-target"})
        self.assertEqual(VERSION, "0.9.148")

    def test_integration_skip_is_command_level_and_serial_has_one_thread(self):
        plans = {item["phase"]: item["argv"] for item in phase_plan(ROOT, Path("/tmp/shadow"))}
        integration = plans["workspace-integration"]
        serial = plans["ui-serial"]
        self.assertEqual(integration[-5:], ["--exclude", "katana-ui", "--", "--skip", "fixture"])
        self.assertEqual(serial[-2:], ["--test-threads", "1"])
        self.assertNotIn("--skip", plans["ui-parallel"])

    def test_shadow_configs_bind_every_phase_to_one_process_group(self):
        config = (ROOT / "scripts/release/nextest-shadow.toml").read_text(encoding="utf-8")
        user = (ROOT / "scripts/release/nextest-shadow-user.toml").read_text(encoding="utf-8")
        self.assertIn("[test-groups.global-shadow]", config)
        self.assertIn("max-threads = 1", config)
        self.assertIn('filter = "all()"', config)
        self.assertIn('test-group = "global-shadow"', config)
        self.assertIn("fail-fast = false", config)
        self.assertIn("retries = 0", config)
        self.assertIn("[experimental]", user)
        self.assertIn("record = true", user)
        self.assertIn("[record]", user)
        self.assertIn("enabled = true", user)
        self.assertEqual(set(tomllib.loads(user)), {"experimental", "record"})

    def test_invalid_phase_is_rejected(self):
        with self.assertRaises(TypeError):
            phase_plan("repo", Path("/tmp/shadow"))

    def test_phase_plan_matches_runner_contract(self):
        plans = phase_plan(ROOT, Path("/tmp/shadow"))
        self.assertEqual([item["phase"] for item in plans], list(PHASES))
        for item in plans:
            self.assertEqual(item["argv"][:2], ["llvm-cov", "nextest"])
            self.assertEqual(item["target_dir"], "/tmp/shadow/coverage-target")
            self.assertNotIn("cargo", item["argv"])

    def test_phase_plan_assigns_one_new_junit_config_and_report_per_phase(self):
        output = Path("/tmp/shadow")
        for item in phase_plan(ROOT, output):
            with self.subTest(phase=item["phase"]):
                config = output / "junit" / f"{item['phase']}.toml"
                report = output / "junit" / f"{item['phase']}.xml"
                self.assertEqual(item["junit_config"], str(config))
                self.assertEqual(item["junit_report"], str(report))
                self.assertEqual(item["config_template"],
                                 str(ROOT / "scripts/release/nextest-shadow.toml"))
                self.assertEqual(item["argv"][item["argv"].index("--config-file") + 1],
                                 str(config))
                self.assertEqual(item["argv"][item["argv"].index("--user-config-file") + 1],
                                 str(ROOT / "scripts/release/nextest-shadow-user.toml"))

    def test_shadow_contract_is_non_authoritative(self):
        contract = shadow_contract(ROOT)
        self.assertFalse(contract["execution_equivalent"])
        self.assertFalse(contract["reuse_authorized"])
        self.assertTrue(contract["normal_remaining"])
        self.assertTrue(contract["doctest_remaining"])


if __name__ == "__main__":
    unittest.main()
