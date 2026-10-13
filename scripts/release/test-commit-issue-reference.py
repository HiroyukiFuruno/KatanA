#!/usr/bin/env python3
"""Tests for commit message Issue-reference validation."""

from __future__ import annotations

import importlib.util
import os
import shlex
import subprocess
import tempfile
import unittest
from pathlib import Path


SCRIPT_PATH = Path(__file__).with_name("check-commit-issue-reference.py")
SPEC = importlib.util.spec_from_file_location("commit_issue_reference", SCRIPT_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"Could not load {SCRIPT_PATH}")
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class CommitIssueReferenceTest(unittest.TestCase):
    def test_extracts_same_repository_issue_numbers_in_order_without_duplicates(self) -> None:
        message = "fix (#335) https://github.com/HiroyukiFuruno/KatanA/issues/312 (#335)"
        self.assertEqual(MODULE.issue_references(message, "HiroyukiFuruno/KatanA"), [335, 312])

    def test_accepts_local_issue_number(self) -> None:
        self.assertTrue(MODULE.has_issue_reference("fix: resolve race (#343)", "HiroyukiFuruno/KatanA"))

    def test_accepts_same_repository_issue_url_case_insensitively(self) -> None:
        message = "Fix documented behavior https://github.com/hiroyukifuruno/katana/issues/335."
        self.assertTrue(MODULE.has_issue_reference(message, "HiroyukiFuruno/KatanA"))

    def test_rejects_message_without_reference(self) -> None:
        self.assertFalse(MODULE.has_issue_reference("fix: resolve race", "HiroyukiFuruno/KatanA"))

    def test_rejects_foreign_repository_url(self) -> None:
        message = "See https://github.com/other/project/issues/335"
        self.assertFalse(MODULE.has_issue_reference(message, "HiroyukiFuruno/KatanA"))

    def test_rejects_pull_request_url(self) -> None:
        message = "See https://github.com/HiroyukiFuruno/KatanA/pull/335"
        self.assertFalse(MODULE.has_issue_reference(message, "HiroyukiFuruno/KatanA"))

    def test_rejects_invalid_repository_identity(self) -> None:
        with self.assertRaises(ValueError):
            MODULE.has_issue_reference("fix (#335)", "KatanA")

    def test_accepts_punctuation_around_local_reference(self) -> None:
        self.assertTrue(MODULE.has_issue_reference("fix: (#335), closes regression", "HiroyukiFuruno/KatanA"))

    def test_rejects_zero_issue_number(self) -> None:
        self.assertFalse(MODULE.has_issue_reference("fix (#0)", "HiroyukiFuruno/KatanA"))

    def test_rejects_reference_embedded_in_word(self) -> None:
        self.assertFalse(MODULE.has_issue_reference("fix: ticket#335x", "HiroyukiFuruno/KatanA"))

    def test_rejects_suffix_attached_to_issue_number(self) -> None:
        self.assertFalse(MODULE.has_issue_reference("fix: #335x", "HiroyukiFuruno/KatanA"))

    def test_does_not_treat_url_fragment_as_local_reference(self) -> None:
        message = 'See "https://example.com/search?q=bug#335"'
        self.assertFalse(MODULE.has_issue_reference(message, "HiroyukiFuruno/KatanA"))

    def test_cli_reads_commit_message_file(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            message_path = Path(directory) / "COMMIT_EDITMSG"
            message_path.write_text("ci: split cache checks (#343)\n", encoding="utf-8")
            self.assertEqual(
                MODULE.main(["--repository", "HiroyukiFuruno/KatanA", str(message_path)]),
                0,
            )


class RealGitBranchContractTests(unittest.TestCase):
    def git(self, arguments: list[str], directory: Path, environment: dict[str, str]) -> str:
        result = subprocess.run(
            ["git", *arguments], cwd=directory, env=environment,
            capture_output=True, text=True, check=True,
        )
        return result.stdout.strip()

    def check_branch(self, branch: str, message: str) -> tuple[subprocess.CompletedProcess, str, str]:
        environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            hooks = root / "hooks"
            hooks.mkdir()
            hook = hooks / "commit-msg"
            hook.write_text(f"#!/bin/sh\nexec bash {shlex.quote(str(SCRIPT_PATH.with_suffix('.sh')))} \"$1\"\n")
            hook.chmod(0o755)
            self.git(["init", "-q", "-b", "master", str(root)], root, environment)
            for key, value in (("user.name", "KatanA Hook Test"), ("user.email", "hook-test@example.invalid"), ("commit.gpgsign", "false"), ("core.hooksPath", str(hooks))):
                self.git(["config", "--local", key, value], root, environment)
            (root / "source.txt").write_text("fixture\n", encoding="utf-8")
            self.git(["add", "source.txt"], root, environment)
            self.git(["commit", "-m", "bootstrap: fixture"], root, environment)
            if branch != "master":
                self.git(["switch", "-c", branch], root, environment)
            before = self.git(["rev-parse", "HEAD"], root, environment)
            (root / "source.txt").write_text("fixture\nchange\n", encoding="utf-8")
            self.git(["add", "source.txt"], root, environment)
            result = subprocess.run(
                ["git", "commit", "-m", message], cwd=root, env=environment,
                capture_output=True, text=True, check=False,
            )
            after = self.git(["rev-parse", "HEAD"], root, environment)
            return result, before, after

    def test_release_branch_rejects_missing_issue(self) -> None:
        result, before, after = self.check_branch("release/v1.0.0", "fix: no reference")
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("No genuine GitHub Issue was verified", result.stderr)
        self.assertEqual(after, before)

    def test_release_branch_accepts_local_issue(self) -> None:
        result, before, after = self.check_branch("release/v1.0.0", "fix: actual issue (#335)")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Verified a genuine GitHub Issue", result.stdout + result.stderr)
        self.assertNotEqual(after, before)

    def test_release_branch_rejects_pull_request_reference(self) -> None:
        result, before, after = self.check_branch("release/v1.0.0", "fix: pull request reference (#346)")
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("No genuine GitHub Issue was verified", result.stderr)
        self.assertEqual(after, before)

    def test_default_branch_is_unchanged(self) -> None:
        result, before, after = self.check_branch("master", "release: generated publication commit")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotEqual(after, before)


if __name__ == "__main__":
    unittest.main()
