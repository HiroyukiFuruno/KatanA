#!/usr/bin/env python3
"""Keep Windows dependency caching separate from build/test evidence."""

from pathlib import Path
import re
import unittest

ROOT = Path(__file__).parents[2]
CACHE_ACTION = "actions/cache@55cc8345863c7cc4c66a329aec7e433d2d1c52a9"


def verify_cache(workflow: str, expected: int) -> None:
    blocks = re.findall(
        r"      - name: Cache Windows Cargo registry\n(.*?)(?=      - |\Z)",
        workflow, re.DOTALL,
    )
    if len(blocks) != expected:
        raise AssertionError("missing Windows registry cache")
    for block in blocks:
        if "if: runner.os == 'Windows'" not in block or CACHE_ACTION not in block:
            raise AssertionError("cache must use the pinned action on Windows")
        paths = block.split("path: |\n", 1)[1].split("          key:", 1)[0]
        if paths.split() != ["~/.cargo/registry/index", "~/.cargo/registry/cache", "~/.cargo/git/db"]:
            raise AssertionError("compiled artifacts must not enter registry cache")
        for identity in ("runner.os", "runner.arch", "hashFiles('**/Cargo.lock')"):
            if identity not in block:
                raise AssertionError("missing cache identity")
        if "restore-keys:" in block:
            raise AssertionError("registry cache requires an exact lock key")
    if re.search(r"if:.*(?:cache-hit|registry_cache)", workflow):
        raise AssertionError("cache hit must not skip verification")


class WindowsRegistryCacheTests(unittest.TestCase):
    def setUp(self) -> None:
        self.ci = (ROOT / ".github/workflows/test-and-build.yml").read_text()
        self.release = (ROOT / ".github/workflows/build-and-release.yml").read_text()

    def test_ci_cache_preserves_existing_release_validation(self) -> None:
        verify_cache(self.ci, 2)
        for step in ("Build release (Windows)", "Verify Windows asset contract", "Smoke launch packaged Windows app"):
            self.assertIn(step, self.release)

    def test_compiled_artifacts_are_rejected(self) -> None:
        altered = self.ci.replace("~/.cargo/git/db", "~/.cargo/git/db\n            target")
        with self.assertRaisesRegex(AssertionError, "compiled artifacts"):
            verify_cache(altered, 2)

    def test_missing_lock_identity_is_rejected(self) -> None:
        altered = self.ci.replace("hashFiles('**/Cargo.lock')", "'static'")
        with self.assertRaisesRegex(AssertionError, "cache identity"):
            verify_cache(altered, 2)

    def test_cache_hit_cannot_skip_tests(self) -> None:
        altered = self.ci + "\n      - name: Run tests\n        if: steps.registry_cache.outputs.cache-hit != 'true'\n"
        with self.assertRaisesRegex(AssertionError, "skip verification"):
            verify_cache(altered, 2)


if __name__ == "__main__":
    unittest.main()
