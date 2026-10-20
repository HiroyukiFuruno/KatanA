"""Keep Cargo compilation cache separate from fresh test runs and profiles."""

import importlib.util
import os
from pathlib import Path
import platform
import subprocess
import sys
import tempfile
import unittest
import uuid

from nextest_shadow_execution import command_context, phase_environment
from nextest_shadow_plan import PHASES, phase_plan

SPEC = importlib.util.spec_from_file_location(
    "shadow_cache_runner", Path(__file__).with_name("run-nextest-shadow.py"))
RUNNER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RUNNER)


class ShadowCacheFlowTests(unittest.TestCase):
    @unittest.skipUnless(platform.system() == "Darwin", "shadow execution is macOS-only")
    def test_invalid_cache_prevents_tool_or_build_child_start(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            marker = root / "child-started"
            child = root / "cargo.py"
            child.write_text(f"from pathlib import Path\nPath({str(marker)!r}).touch()\n",
                             encoding="utf-8")
            environment = {**os.environ, "CARGO": f"{sys.executable} {child}"}
            output = root / "tmp/nextest-shadow/fresh"
            result = subprocess.run(
                [sys.executable, "-B", str(Path(__file__).with_name("run-nextest-shadow.py")),
                 "--repo", str(root), "--output", str(output),
                 "--build-cache-target", str(root / "outside")],
                env=environment, capture_output=True, text=True, check=False)
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertIn("outside nextest shadow runs", result.stderr)
            self.assertFalse(marker.exists())
            self.assertFalse(output.exists())

    def test_default_and_explicit_targets_keep_all_phase_selections(self):
        output, cache = Path("/repo/tmp/fresh"), Path("/repo/tmp/old/coverage-target")
        _, fresh = command_context(output)
        _, reused = command_context(output, cache)
        self.assertEqual(fresh["CARGO_TARGET_DIR"], str(output / "coverage-target"))
        self.assertEqual(reused["CARGO_TARGET_DIR"], str(cache))
        self.assertEqual(reused["CARGO_LLVM_COV_TARGET_DIR"], str(cache))
        plans = phase_plan(Path("/repo"), output, cache)
        self.assertEqual({phase["target_dir"] for phase in plans}, {str(cache)})
        self.assertEqual([phase["phase"] for phase in plans], list(PHASES))
        self.assertEqual([phase["argv"] for phase in plans],
                         [phase["argv"] for phase in phase_plan(Path("/repo"), output)])

    def test_profile_namespace_is_fresh_even_with_same_output_and_cache(self):
        output, cache = Path("/repo/tmp/fresh"), Path("/repo/tmp/old/coverage-target")
        _, first = command_context(output, cache)
        _, second = command_context(output, cache)
        self.assertNotEqual(first["KATANA_SHADOW_PROFILE_ID"], second["KATANA_SHADOW_PROFILE_ID"])
        self.assertEqual(uuid.UUID(first["KATANA_SHADOW_PROFILE_ID"]).hex,
                         first["KATANA_SHADOW_PROFILE_ID"])
        names = {phase_environment(base, output / name)["LLVM_PROFILE_FILE_NAME"]
                 for base in (first, second) for name in PHASES}
        self.assertEqual(len(names), 8)
        for name in PHASES:
            phase = phase_environment(first, output / name)
            self.assertIn(first["KATANA_SHADOW_PROFILE_ID"], phase["LLVM_PROFILE_FILE_NAME"])
            self.assertEqual(phase["NEXTEST_STATE_DIR"], str(output / name / "nextest-state"))

    def test_receipt_names_cache_without_granting_test_reuse(self):
        repo = Path(__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "new"
            cache = Path(temporary) / "old/coverage-target"
            _, environment = command_context(output, cache)
            receipt = RUNNER.initial_receipt(repo, output, ["cargo"], environment, {}, {})
            self.assertEqual(receipt["build_cache_target"], str(cache))
            self.assertEqual(receipt["profile_namespace"], environment["KATANA_SHADOW_PROFILE_ID"])
            for key in ("execution_equivalent", "reuse_authorized", "report_checked",
                        "producer_completed"):
                self.assertFalse(receipt[key])


if __name__ == "__main__":
    unittest.main()
