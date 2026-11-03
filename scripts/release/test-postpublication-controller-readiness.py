#!/usr/bin/env python3
"""Guard the narrowly-scoped v0.22.47 post-publication controller readiness route."""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SELECTOR = ROOT / "scripts/release/select-pr-readiness-mode.sh"
GATE = ROOT / "scripts/release/check-pr-ready.sh"
BRANCH = "release/v0.22.47-postpublication-controller"
MODE = "postpublication-controller-repair"


def bash_executable(environment: dict[str, str]) -> str:
    if os.name == "nt":
        executable = environment.get("KATANA_GUARD_BASH_EXECUTABLE", "")
    else:
        executable = shutil.which("bash", path=environment.get("PATH")) or ""
    path = Path(executable)
    if not executable or not path.is_absolute() or not path.is_file():
        raise AssertionError(f"an absolute native bash executable is required: {executable!r}")
    environment["PATH"] = str(path.parent) + os.pathsep + environment.get("PATH", "")
    return str(path)


def select(branch: str, version: str, requested: str = "") -> subprocess.CompletedProcess[str]:
    environment = os.environ.copy()
    shell = bash_executable(environment)
    return subprocess.run(
        [shell, str(SELECTOR), branch, version, requested],
        cwd=ROOT,
        env=environment,
        capture_output=True,
        text=True,
        check=False,
        timeout=5,
    )


class PostpublicationControllerReadinessTest(unittest.TestCase):
    def test_only_exact_branch_and_version_select_repair_mode(self) -> None:
        for requested in ("", "pr-bootstrap"):
            with self.subTest(requested=requested):
                result = select(BRANCH, "0.22.47", requested)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(result.stdout, MODE + "\n")

        for branch, version in (
            (BRANCH + "-extra", "0.22.47"),
            ("release/v0.22.47", "0.22.47"),
            (BRANCH, "0.22.48"),
            ("release/v0.22.48-postpublication-controller", "0.22.48"),
        ):
            with self.subTest(branch=branch, version=version):
                result = select(branch, version, "pr-bootstrap")
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(result.stdout, "pr-bootstrap\n")

    def test_repair_mode_cannot_be_requested_as_a_bypass(self) -> None:
        for branch, version in ((BRANCH, "0.22.47"), ("release/v0.22.47", "0.22.47")):
            with self.subTest(branch=branch):
                result = select(branch, version, MODE)
                self.assertNotEqual(result.returncode, 0)

    def test_gate_inline_import_is_callable_from_repo_root(self) -> None:
        source = GATE.read_text(encoding="utf-8")
        match = re.search(r"(?m)^\s*python3 -B -c '([^']*)' \\?\|\| \{$", source)
        self.assertIsNotNone(match, "controller helper invocation is missing")
        prefix = match.group(1).split("; verify_host_controller(", 1)[0]
        self.assertIn('sys.path.insert(0, str(Path.cwd() / "scripts/release"))', prefix)
        command = subprocess.run(
            [sys.executable, "-B", "-c", prefix + "; assert callable(verify_host_controller)"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
            timeout=30,
        )
        self.assertEqual(command.returncode, 0, command.stdout + command.stderr)

    def test_gate_binds_published_source_scope_and_preserves_preflight(self) -> None:
        source = GATE.read_text(encoding="utf-8")
        self.assertIn("gh release view", source)
        self.assertIn("draft=false prerelease=false", source)
        self.assertIn('if [[ "$RELEASE_REPO" != "HiroyukiFuruno/KatanA" ]]; then', source)
        self.assertIn("gh api \"repos/${RELEASE_REPO}/commits/v0.22.47\" --jq '.sha'", source)
        self.assertIn('[[ "$PUBLISHED" != "6dabe8531e2b340630887967b63206b59ebc6de3" ]]', source)
        self.assertIn("verify_host_controller(Path.cwd(), \"6dabe8531e2b340630887967b63206b59ebc6de3\", controller=None)", source)
        self.assertIn('if ! KATANA_OPENSPEC_TASK_GATE="pr-bootstrap" ./scripts/release/preflight.sh "$TARGET_VERSION"', source)
        self.assertIn('if ! KATANA_OPENSPEC_TASK_GATE="$TASK_GATE_MODE" ./scripts/release/preflight.sh "$TARGET_VERSION"', source)
        self.assertIn('if ! ./scripts/release/check-version-increment.sh "$TARGET_VERSION"; then', source)
        branch = source.split('elif [[ "$CONTROLLER_REPAIR_MODE" == "true" ]]; then', 1)[1].split("\nelse\n", 1)[0]
        self.assertNotIn("check-version-increment.sh", branch)

    def test_foreign_repository_override_is_rejected_before_gh(self) -> None:
        with tempfile.TemporaryDirectory(prefix="controller-foreign-repo-") as directory:
            temporary = Path(directory)
            marker = temporary / "gh-was-called"
            fake_gh = temporary / "gh"
            fake_gh.write_text("#!/bin/sh\nprintf called > \"$GH_MARKER\"\n", encoding="utf-8")
            fake_gh.chmod(0o755)
            environment = os.environ.copy()
            environment["KATANA_RELEASE_REPO"] = "attacker/fork"
            environment["GH_MARKER"] = str(marker)
            environment["GITHUB_ACTIONS"] = "true"
            environment["GITHUB_HEAD_REF"] = BRANCH
            environment["PATH"] = str(temporary) + os.pathsep + environment["PATH"]
            shell = bash_executable(environment)
            result = subprocess.run(
                [shell, str(GATE), "0.22.47", "--pr-bootstrap"],
                cwd=ROOT,
                env=environment,
                capture_output=True,
                text=True,
                check=False,
                timeout=60,
            )
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("restricted to HiroyukiFuruno/KatanA", result.stderr)
            self.assertFalse(marker.exists(), "gh must not receive a foreign repository")


if __name__ == "__main__":
    unittest.main()
