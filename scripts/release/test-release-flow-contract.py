#!/usr/bin/env python3
"""Verify that local and release gates have one owner each."""

from pathlib import Path
import os
import subprocess
import tempfile
import textwrap
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


def run_readiness_routing(workflow: str, branch: str) -> list[str]:
    commands = [block for block in run_blocks(workflow) if "./scripts/release/check-pr-ready.sh" in block]
    if len(commands) != 1:
        raise ValueError("readiness must have exactly one executable routing block")
    script = textwrap.dedent("\n".join(commands[0].splitlines()[1:]))
    with tempfile.TemporaryDirectory(prefix="katana-readiness-routing-") as temporary:
        root = Path(temporary)
        checker = root / "scripts/release/check-pr-ready.sh"
        checker.parent.mkdir(parents=True)
        checker.write_text('#!/usr/bin/env bash\nprintf \'%s\\0\' "$@" > "$ROUTING_CAPTURE"\n', encoding="utf-8")
        checker.chmod(0o755)
        (root / "Cargo.toml").write_text('version = "0.22.42"\n', encoding="utf-8")
        capture = root / "recorded-arguments"
        environment = {"PATH": os.environ["PATH"], "GITHUB_HEAD_REF": branch, "ROUTING_CAPTURE": str(capture)}
        result = subprocess.run(["bash", "-euo", "pipefail", "-c", script], cwd=root, env=environment,
                                check=False, capture_output=True, text=True, timeout=10)
        if result.returncode != 0:
            raise AssertionError(f"actual readiness routing failed: {result.stderr}")
        if not capture.is_file():
            raise AssertionError("actual readiness routing did not call its checker")
        return [value.decode() for value in capture.read_bytes().split(b"\0") if value]


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
    def test_native_dependency_contract_is_built_once_before_release_unit_tests(self) -> None:
        preflight = (ROOT / "scripts/release/preflight.sh").read_text(encoding="utf-8")
        build = "cargo build --locked --offline --quiet -p katana-linter --bin dependency-contract"
        unit_tests = "python3 -m unittest discover -s scripts/release -p 'test_*.py'"
        self.assertEqual(preflight.count(build), 1)
        self.assertLess(preflight.index(build), preflight.index(unit_tests))

    def test_supply_chain_covers_host_and_tool_locked_graphs_at_all_entrypoints(self) -> None:
        entry = "bash scripts/ci/check-supply-chain.sh"
        checker = ROOT / "scripts/ci/check-supply-chain.sh"
        self.assertTrue(checker.is_file(), "Host and tool locks need one shared supply-chain gate")
        source = checker.read_text(encoding="utf-8")
        self.assertIn("set -euo pipefail", source)
        self.assertIn("python3 -B -m unittest discover -s scripts/ci -p test_zip_advisory_overlay.py", source)
        self.assertIn("python3 scripts/ci/zip_advisory_overlay.py --deny-file deny.toml", source)
        for manifest in ("Cargo.toml", "scripts/screenshot/Cargo.toml", "scripts/paint-metrics/Cargo.toml"):
            self.assertIn(manifest, source)
        overlay = (ROOT / "scripts/ci/zip_advisory_overlay.py").read_text(encoding="utf-8")
        self.assertIn('"cargo", "metadata", "--locked", "--all-features"', overlay)
        self.assertIn('"check", "--hide-inclusion-graph"', overlay)
        self.assertIn('"--locked", "--manifest-path", str(manifest)', overlay)
        for path in (
            "just/quality.just", "scripts/release/preflight.sh",
            ".github/workflows/test-and-build.yml",
        ):
            with self.subTest(path=path):
                self.assertIn(entry, (ROOT / path).read_text(encoding="utf-8"))
        recipe = (ROOT / "just/quality.just").read_text(encoding="utf-8")
        self.assertNotIn("{{RTK_CMD}}" + entry, recipe)

    def test_coverage_installer_is_pinned_before_execution(self) -> None:
        workflow = (ROOT / ".github/workflows/test-and-build.yml").read_text(encoding="utf-8")
        self.assertNotIn("/releases/latest/download/cargo-llvm-cov", workflow)
        self.assertLess(workflow.index("- name: Install coverage tool\n"), workflow.index("- name: Run coverage\n"))
        install = workflow.split("- name: Install coverage tool\n", 1)[1].split("- name: Run coverage\n", 1)[0]
        self.assertIn("uses: taiki-e/install-action@1e29709c0ec63babbe8f266397a465f29a825f91", install)
        self.assertIn("tool: cargo-llvm-cov@0.9.1", install)
        self.assertIn("fallback: none", install)
        condition = "if: runner.os == 'macOS' && github.event_name == 'pull_request'"
        self.assertIn(condition, install)
        coverage_step = workflow.split("- name: Run coverage\n", 1)[1].split("\n      - name:", 1)[0]
        self.assertIn(condition, coverage_step)
        coverage = next(block for block in run_blocks(workflow) if "JOBS=1 just coverage" in block)
        self.assertNotIn("curl", coverage)
        self.assertIn("rustup component add llvm-tools-preview", coverage)

    def test_post_release_host_route_retains_existing_published_release_guard(self) -> None:
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
        self.assertIn('gh release view "v${TARGET_VERSION}"', check_pr_ready)
        self.assertIn('"v${TARGET_VERSION} draft=false prerelease=false"', check_pr_ready)
        self.assertIn('KATANA_OPENSPEC_TASK_GATE="$TASK_GATE_MODE" ./scripts/release/preflight.sh', check_pr_ready)
        self.assertEqual(run_readiness_routing(readiness, "release/v0.22.42-public-release-evidence"),
                         ["0.22.42", "--post-release-host"])

    def test_only_exact_fixed_release_evidence_branch_uses_post_host(self) -> None:
        readiness = (ROOT / ".github/workflows/release-readiness.yml").read_text(encoding="utf-8")
        for branch, version in (
            ("release/v0.22.42", "0.22.42"),
            ("release/v0.22.43-public-release-evidence", "0.22.43"),
            ("release/v0.22.420-public-release-evidence", "0.22.420"),
            ("release/v0.22.42-public-release-evidence-extra", "0.22.42"),
            ("release/v0.22.42-public-release-evidence/task", "0.22.42"),
            ("release/v0.22.42-PUBLIC-release-evidence", "0.22.42"),
            ("release/v0.22.42-public-release-evidenc", "0.22.42"),
            ("feature/v0.22.42-public-release-evidence", "0.22.42"),
            ("docs/v0.22.42-public-release-evidence-extra", "0.22.42"),
            ("master", "0.22.42"),
        ):
            with self.subTest(branch=branch):
                self.assertEqual(run_readiness_routing(readiness, branch), [version, "--pr-bootstrap"])

    def test_legacy_docs_evidence_route_is_unchanged(self) -> None:
        readiness = (ROOT / ".github/workflows/release-readiness.yml").read_text(encoding="utf-8")
        for branch in ("docs/v0.22.42-public-release-evidence", "docs/v0.22.41-public-release-evidence"):
            with self.subTest(branch=branch):
                self.assertEqual(run_readiness_routing(readiness, branch), ["0.22.42", "--post-release-evidence"])

    def test_readiness_checkout_keeps_recorded_build_commit_available(self) -> None:
        readiness = (ROOT / ".github/workflows/release-readiness.yml").read_text(encoding="utf-8")
        checkout = readiness.split("uses: actions/checkout@", 1)[1].split("      - uses:", 1)[0]
        self.assertIn("with:\n          fetch-depth: 0\n", checkout)

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
    assert "run: python3 scripts/release/run-owned-push-check.py" in lefthook
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
