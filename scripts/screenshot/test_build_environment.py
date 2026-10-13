"""Verify the real shell environment defaults without invoking Cargo."""

import os
from pathlib import Path
import subprocess
import unittest


SCRIPT = Path(__file__).with_name("build-environment.sh")


class BuildEnvironmentTests(unittest.TestCase):
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
