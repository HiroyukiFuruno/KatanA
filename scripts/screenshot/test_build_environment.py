"""Verify the real shell environment defaults without invoking Cargo."""

import os
from pathlib import Path
import subprocess
import unittest


SCRIPT = Path(__file__).with_name("build-environment.sh")
RUNNER = Path(__file__).with_name("run.sh")


class BuildEnvironmentTests(unittest.TestCase):
    def runner_target_assignments(self) -> str:
        return "\n".join(
            line for line in RUNNER.read_text().splitlines()
            if line.startswith((
                "BUILD_TARGET_DIR=",
                "SCREENSHOT_TARGET_DIR=",
                "EXTERNAL_HYPERLINK_FIXTURE=",
            ))
        )

    def evaluate_runner_targets(self, assignments: str, target: str | None) -> tuple[str, str, str]:
        environment = dict(os.environ)
        if target is None:
            environment.pop("CARGO_TARGET_DIR", None)
        else:
            environment["CARGO_TARGET_DIR"] = target
        script = (
            "set -euo pipefail\nREPO_ROOT=/fixed/katana-root\n"
            f"{assignments}\nprintf '%s\\0%s\\0%s\\0' "
            "\"$BUILD_TARGET_DIR\" \"$SCREENSHOT_TARGET_DIR\" "
            "\"$EXTERNAL_HYPERLINK_FIXTURE\"\n"
        )
        result = subprocess.run(["bash", "-c", script], env=environment,
                                capture_output=True, check=True, text=False)
        values = result.stdout.decode().split("\0")
        self.assertEqual(values[-1], "")
        return tuple(values[:3])

    def test_screenshot_runner_target_assignments_preserve_shell_path_semantics(self):
        assignments = self.runner_target_assignments()
        self.assertEqual(assignments.count("\n") + 1, 3)
        expected_default = (
            "/fixed/katana-root/target",
            "/fixed/katana-root/target/screenshot-harness",
            "/fixed/katana-root/target/screenshot-fixtures/representative-with-external-hyperlink.pptx",
        )
        cases = ((None, expected_default), ("", expected_default))
        cases += tuple((target, (target, f"{target}/screenshot-harness",
                                  f"{target}/screenshot-fixtures/representative-with-external-hyperlink.pptx"))
                       for target in ("/tmp/canonical output", "relative canonical output"))
        for target, expected in cases:
            with self.subTest(target=target):
                self.assertEqual(self.evaluate_runner_targets(assignments, target), expected)

    def test_screenshot_runner_respects_caller_target_and_separates_child_lock(self):
        source = RUNNER.read_text()
        self.assertIn('BUILD_TARGET_DIR="${CARGO_TARGET_DIR:-${REPO_ROOT}/target}"', source)
        self.assertIn('SCREENSHOT_TARGET_DIR="${BUILD_TARGET_DIR}/screenshot-harness"', source)
        self.assertIn('EXTERNAL_HYPERLINK_FIXTURE="${BUILD_TARGET_DIR}/screenshot-fixtures/', source)
        self.assertIn('CARGO_TARGET_DIR="${BUILD_TARGET_DIR}" cargo build --locked --release', source)
        self.assertIn('CARGO_TARGET_DIR="${SCREENSHOT_TARGET_DIR}" cargo build --locked --release', source)
        self.assertNotIn('BUILD_TARGET_DIR="${REPO_ROOT}/target"', source)

    def flags(self, supplied: str | None, encoded: str = "", platform: str = "darwin") -> tuple[str, str]:
        environment = dict(os.environ)
        environment.pop("RUSTFLAGS", None)
        if supplied is not None:
            environment["RUSTFLAGS"] = supplied
        environment["CARGO_ENCODED_RUSTFLAGS"] = encoded
        result = subprocess.run(
            ["bash", "-eu", "-c", 'OSTYPE="$2"; source "$1"; printf "%s\\0%s\\0" "${RUSTFLAGS-__unset__}" "$CARGO_ENCODED_RUSTFLAGS"',
             "build-environment-test", str(SCRIPT), platform],
            env=environment, capture_output=True, check=True,
        )
        flags, preserved, terminator = result.stdout.decode().split("\0")
        self.assertEqual(terminator, "")
        return flags, preserved

    def test_unset_flags_match_strict_just_default(self):
        self.assertEqual(self.flags(None), ("-D warnings", ""))

    def test_explicit_flags_and_empty_value_are_preserved(self):
        for flags in ("", "-D warnings -C debuginfo=0"):
            with self.subTest(flags=flags):
                self.assertEqual(self.flags(flags), (flags, ""))

    def test_encoded_flags_are_not_removed_or_rewritten(self):
        encoded = "-C\x1fdebuginfo=0"
        self.assertEqual(self.flags(None, encoded), ("-D warnings", encoded))

    def test_other_platforms_preserve_unset_target_configuration(self):
        for platform in ("linux-gnu", "msys", "cygwin"):
            with self.subTest(platform=platform):
                self.assertEqual(self.flags(None, platform=platform), ("__unset__", ""))
                self.assertEqual(self.flags("", platform=platform), ("", ""))
                self.assertEqual(self.flags("-C target-feature=+crt-static", platform=platform),
                                 ("-C target-feature=+crt-static", ""))


if __name__ == "__main__":
    unittest.main()
