#!/usr/bin/env python3
"""Verify Draft PRs are quiet and only Ready PRs run release gates."""

from pathlib import Path
import re
import unittest

ROOT = Path(__file__).parents[2]
EVENT_TYPES = (
    "opened",
    "synchronize",
    "reopened",
    "ready_for_review",
    "converted_to_draft",
)
TEST_GUARD = "if: github.event_name == 'push' || (github.event_name == 'pull_request' && !github.event.pull_request.draft)"
PR_GUARD = "if: github.event_name == 'pull_request' && !github.event.pull_request.draft"


def job_block(workflow: str, name: str) -> str:
    match = re.search(
        rf"^  {re.escape(name)}:\n(.*?)(?=^  [A-Za-z0-9_-]+:|\Z)",
        workflow,
        re.MULTILINE | re.DOTALL,
    )
    if not match:
        raise AssertionError(f"missing root job: {name}")
    return match.group(0)


def verify_workflow(workflow: str, readiness: str) -> None:
    for current in (workflow, readiness):
        pull_request = re.search(r"^  pull_request:\n(.*?)(?=^\S|\Z)", current, re.MULTILINE | re.DOTALL)
        if pull_request is None:
            raise AssertionError("missing pull_request trigger")
        event_types = pull_request.group(1)
        for event_type in EVENT_TYPES:
            if event_type not in event_types:
                raise AssertionError(f"missing pull_request event type: {event_type}")
    if TEST_GUARD not in job_block(workflow, "test"):
        raise AssertionError("test job must allow master pushes and non-Draft PRs")
    for name in ("lint", "supply-chain", "codeql"):
        if PR_GUARD not in job_block(workflow, name):
            raise AssertionError(f"{name} must skip Draft PRs")
    if PR_GUARD not in job_block(readiness, "release-check"):
        raise AssertionError("release readiness must skip Draft PRs")
    for step in ("Run tests", "Run linter", "Dependency Supply Chain", "CodeQL Security Scan"):
        if step not in workflow:
            raise AssertionError(f"existing CI step disappeared: {step}")
    if "check-pr-ready.sh" not in readiness:
        raise AssertionError("release readiness command disappeared")


class DraftCiContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = (ROOT / ".github/workflows/test-and-build.yml").read_text(encoding="utf-8")
        cls.readiness = (ROOT / ".github/workflows/release-readiness.yml").read_text(encoding="utf-8")

    def test_current_workflows_follow_draft_policy(self) -> None:
        verify_workflow(self.workflow, self.readiness)

    def test_missing_ready_event_is_rejected(self) -> None:
        altered = self.workflow.replace("ready_for_review, ", "")
        with self.assertRaises(AssertionError):
            verify_workflow(altered, self.readiness)

    def test_missing_root_guard_is_rejected(self) -> None:
        altered = self.workflow.replace(f"    {PR_GUARD}\n", "    if: github.event_name == 'pull_request'\n")
        with self.assertRaises(AssertionError):
            verify_workflow(altered, self.readiness)

    def test_missing_readiness_ready_event_is_rejected(self) -> None:
        altered = self.readiness.replace("ready_for_review, ", "")
        with self.assertRaises(AssertionError):
            verify_workflow(self.workflow, altered)

    def test_missing_readiness_guard_is_rejected(self) -> None:
        altered = self.readiness.replace(PR_GUARD, "if: github.event_name == 'pull_request'")
        with self.assertRaises(AssertionError):
            verify_workflow(self.workflow, altered)


if __name__ == "__main__":
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(DraftCiContractTests)
    if not unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful():
        raise SystemExit(1)
