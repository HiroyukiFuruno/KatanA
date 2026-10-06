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


def verify_release_guidance(workflow: str, skill: str) -> None:
    if "`./scripts/release/check-pr-ready.sh X.Y.Z` を実行し" in workflow:
        raise ValueError("normal push owns readiness; do not require a prior manual run")
    if "通常pushのpre-push hook" not in workflow:
        raise ValueError("release guidance must identify the normal push owner")
    if "1コミットに圧縮" in skill:
        raise ValueError("release skill must preserve ordinary reviewed commits")


def verify_document_phase_order(workflow: str) -> None:
    preflight = workflow.split("  build_macos:", 1)[0]
    publish = workflow.split("  publish:", 1)[1]
    if "--document-source" not in preflight or " packaged-host " in preflight:
        raise ValueError("build preflight must verify source, not unbuilt packaged evidence")
    host_gate = publish.find('check-document-fidelity-release-gate.py "${{ needs.preflight.outputs.version_bare }}" packaged-host')
    public_commands = [publish.find(command) for command in
                       ("gh release edit", "gh release upload", "gh release create")]
    if host_gate < 0 or any(position < 0 or host_gate > position for position in public_commands):
        raise ValueError("packaged host acceptance must succeed before any public release")
    if "needs: [preflight, build_macos, smoke_macos, build_linux, build_windows]" not in publish:
        raise ValueError("host acceptance must follow every build and native macOS smoke")


class WorkflowContractTests(unittest.TestCase):
    def test_post_release_host_mode_is_opt_in(self) -> None:
        check_pr_ready = (ROOT / "scripts/release/check-pr-ready.sh").read_text(
            encoding="utf-8"
        )
        readiness = (ROOT / ".github/workflows/release-readiness.yml").read_text(
            encoding="utf-8"
        )
        self.assertIn(
            "--post-release-host) TASK_GATE_MODE=\"post-release-host\" ;;",
            check_pr_ready,
        )
        self.assertIn(
            'if [[ "$TASK_GATE_MODE" == "post-release-evidence" || '
            '"$TASK_GATE_MODE" == "post-release-host" ]]; then',
            check_pr_ready,
        )
        self.assertIn("--post-release-evidence", readiness)
        self.assertNotIn("--post-release-host", readiness)

    def test_real_workflow_requires_packaged_host_before_publication(self) -> None:
        workflow = (ROOT / ".github/workflows/build-and-release.yml").read_text(encoding="utf-8")
        verify_document_phase_order(workflow)
        for altered in (
            workflow.replace("--document-source", "--release-artifact-pending"),
            workflow.replace(" packaged-host --root .", " source --root ."),
            workflow.replace("needs: [preflight, build_macos, smoke_macos, build_linux, build_windows]", "needs: preflight"),
            workflow.replace("      - name: Verify downloaded publication executable identities", "      - name: Early unsafe publication\n        run: gh release upload v0.22.42 bad.zip\n      - name: Verify downloaded publication executable identities"),
        ):
            with self.assertRaises(ValueError):
                verify_document_phase_order(altered)

    def test_manual_readiness_before_push_is_rejected(self) -> None:
        with self.assertRaises(ValueError):
            verify_release_guidance("`./scripts/release/check-pr-ready.sh X.Y.Z` を実行し", "")

    def test_old_squash_skill_is_rejected(self) -> None:
        with self.assertRaises(ValueError):
            verify_release_guidance("通常pushのpre-push hook", "1コミットに圧縮")

    def test_hook_owned_guidance_is_valid(self) -> None:
        verify_release_guidance("通常pushのpre-push hook", "検証済み通常コミット")

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
    release_skill = (ROOT / ".agents/skills/impl-release/SKILL.md").read_text(encoding="utf-8")

    assert "run: just lint-impacted" in lefthook
    assert "run: just lint\n" not in lefthook
    assert "pre-push: check\n" in tests
    assert "pre-push: check-full" not in tests
    assert "run: just check" in lefthook
    assert "preflight.sh" in check_pr_ready
    verify_workflow(build_workflow)
    verify_document_phase_order(build_workflow)
    verify_workflow(readiness_workflow)
    verify_release_guidance(impl_release, release_skill)
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
