#!/usr/bin/env python3
"""Verify that local and release gates have one owner each."""

from pathlib import Path
import unittest

ROOT = Path(__file__).parents[2]


def run_blocks(workflow: str) -> list[str]:
    lines = workflow.splitlines()
    blocks: list[str] = []
    for index, line in enumerate(lines):
        if line.lstrip().startswith("run:"):
            indent = len(line) - len(line.lstrip())
            body = [line]
            for following in lines[index + 1 :]:
                if not following.strip():
                    continue
                if len(following) - len(following.lstrip()) <= indent:
                    break
                body.append(following)
            blocks.append("\n".join(body))
    return blocks


def verify_workflow(workflow: str) -> None:
    for block in run_blocks(workflow):
        commands = "\n".join(
            line.split("#", 1)[0] for line in block.splitlines()
        )
        if "release-preflight" in commands or "scripts/release/preflight.sh" in commands:
            raise ValueError("check-pr-ready owns release preflight; do not run it twice")


class WorkflowContractTests(unittest.TestCase):
    def test_single_readiness_entrypoint_is_valid(self) -> None:
        verify_workflow("  - run: ./scripts/release/check-pr-ready.sh 0.22.42\n")

    def test_direct_recipe_is_rejected(self) -> None:
        with self.assertRaises(ValueError):
            verify_workflow("  - name: Duplicate\n    run: just release-preflight\n")

    def test_multiline_direct_script_is_rejected(self) -> None:
        with self.assertRaises(ValueError):
            verify_workflow("    run: |\n      ./scripts/release/check-pr-ready.sh\n      bash scripts/release/preflight.sh\n")

    def test_comments_and_other_step_fields_are_not_commands(self) -> None:
        verify_workflow("    run: echo ok # never use release-preflight\n  - name: next\n    env:\n      NOTE: release-preflight\n")


def main() -> None:
    lefthook = (ROOT / "lefthook.yml").read_text(encoding="utf-8")
    tests = (ROOT / "just/tests.just").read_text(encoding="utf-8")
    maintenance = (ROOT / "just/maintenance.just").read_text(encoding="utf-8")
    check_pr_ready = (ROOT / "scripts/release/check-pr-ready.sh").read_text(encoding="utf-8")
    build_workflow = (ROOT / ".github/workflows/build-and-release.yml").read_text(encoding="utf-8")
    readiness_workflow = (ROOT / ".github/workflows/release-readiness.yml").read_text(encoding="utf-8")
    impl_release = (ROOT / ".agents/workflows/impl-release.md").read_text(encoding="utf-8")

    assert "run: just lint-impacted" in lefthook
    assert "run: just lint\n" not in lefthook
    assert "pre-push: check\n" in tests
    assert "pre-push: check-full" not in tests
    assert "run: just check" in lefthook
    assert "preflight.sh" in check_pr_ready
    verify_workflow(build_workflow)
    verify_workflow(readiness_workflow)
    assert "check-platforms" in tests
    assert "coverage" in tests
    assert "check-full:" in maintenance
    assert "coverage" in maintenance
    assert "check-platforms" in maintenance
    assert "release/vX.Y.Z" in impl_release
    assert "feature/vX.Y.Z-taskN" not in impl_release
    assert "git reset --soft" not in impl_release
    assert "Draft PR" in impl_release
    assert "P0/P1" in impl_release
    assert "Ready for review" in impl_release
    assert "gh pr merge --merge" in impl_release
    assert "--delete-branch" not in impl_release
    print("OK: release flow ownership contract passed")


if __name__ == "__main__":
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(WorkflowContractTests)
    if not unittest.TextTestRunner().run(suite).wasSuccessful():
        raise SystemExit(1)
    main()
