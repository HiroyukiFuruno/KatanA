#!/usr/bin/env python3
"""Verify that local and release gates have one owner each."""

from pathlib import Path
import base64
import json
import os
import re
import shutil
import shlex
import subprocess
import sys
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
    if "--automated-release" not in preflight or " packaged-host " in preflight:
        raise ValueError("build preflight must use approved automatic quality without manual host evidence")
    automatic_gate = publish.find('check-document-fidelity-release-gate.py "${{ needs.preflight.outputs.version_bare }}" automated-release')
    asset_gate = publish.find("check-published-artifact-identity.py")
    public_commands = [publish.find(command) for command in
                       ("gh release edit", "gh release upload", "gh release create")]
    if automatic_gate < 0 or asset_gate < automatic_gate or any(position < 0 or asset_gate > position for position in public_commands):
        raise ValueError("automatic task and all-asset identity checks must succeed before public release")
    if "--scope automated-release --phase prepublish --build-manifests ." not in publish:
        raise ValueError("publication must verify all generated current-source payload identities")
    if " packaged-host " in publish or "--scope katana-host" in publish:
        raise ValueError("manual document QA cannot be an implicit publication prerequisite")
    if "needs: [preflight, build_macos, smoke_macos, build_linux, build_windows]" not in publish:
        raise ValueError("publication must follow every build and native macOS smoke")


def verify_post_publication_artifact_route(workflow: str) -> None:
    step = re.search(
        r"(?ms)^      - name: Verify actual published release artifacts\n(.*?)(?=^      - |\Z)",
        workflow,
    )
    if step is None:
        raise ValueError("post-release readiness must execute actual published asset checks")
    body = step.group(1)
    required = (
        "if: github.head_ref == 'release/v0.22.42-public-release-evidence'",
        "repos/HiroyukiFuruno/KatanA/commits/v0.22.42",
        'scripts/release/postpublication_run_admission.py --repository HiroyukiFuruno/KatanA --source "$tag_commit" --output "$work/run-admission.json"',
        "python3 -B scripts/release/test-published-git-source.py",
        "python3 -B scripts/release/test-postpublication-controller-scope.py",
        "python3 -B scripts/release/test-postpublication-run-admission.py",
        'gh run download "$build_run"',
        "--name artifacts-identity-macos --name artifacts-identity-linux --name artifacts-identity-windows",
        'record.get("github_run_id"), record.get("github_run_attempt"), record.get("provenance", {}).get("source_commit")',
        "(build_run, build_attempt, tag_commit)",
        "--scope automated-release --phase postpublication",
        '--assets "$work/assets"',
        '--build-manifests "$work/manifests"',
    )
    if any(marker not in body for marker in required):
        raise ValueError("post-release artifacts must bind the actual tag, build run, and all five payloads")
    if "  actions: read\n" not in workflow:
        raise ValueError("cross-run artifact downloads need read-only Actions permission")
    if ('name: published-run-admission' not in workflow or
            'steps.actual_published_artifacts.outputs.admission_path' not in workflow):
        raise ValueError("post-publication job admission must retain its full run and attempt evidence")
    if "check-document-host" in body or "--scope katana-host" in body:
        raise ValueError("published artifact checks must not require manual document QA")


class WorkflowContractTests(unittest.TestCase):
    def test_homebrew_cask_generator_emits_current_declarative_dsl(self) -> None:
        updater_source = ROOT / "scripts/release/update-homebrew.sh"
        with tempfile.TemporaryDirectory(prefix="katana-homebrew-generator-") as temporary:
            root = Path(temporary)
            fake_bin = root / "bin"
            fake_bin.mkdir()
            captures = root / "requests.jsonl"
            curl = fake_bin / "curl"
            curl.write_text(
                "#!/usr/bin/env python3\n"
                "import json, os, sys\n"
                "args = sys.argv[1:]\n"
                "if '-X' in args and args[args.index('-X') + 1] == 'PUT':\n"
                " payload = json.loads(args[args.index('-d') + 1])\n"
                " with open(os.environ['CASK_REQUESTS'], 'a', encoding='utf-8') as out:\n"
                "  out.write(json.dumps(payload) + '\\n')\n"
                " print(json.dumps({'content': {'html_url': 'https://example.invalid/cask'}}) + '\\n200')\n"
                "else:\n"
                " print(json.dumps({'sha': 'fixture-current-sha'}))\n",
                encoding="utf-8",
            )
            curl.chmod(0o700)
            python = fake_bin / "python3"
            python.write_text("#!/bin/sh\nexec " + shlex.quote(sys.executable) + ' "$@"\n')
            python.chmod(0o700)
            environment = {
                "PATH": os.pathsep.join((str(fake_bin), "/usr/bin", "/bin")),
                "HOME": str(root),
                "ZDOTDIR": str(root),
                "EXPECTED_CURL": str(curl),
                "HOMEBREW_KATANA_GIT_TOKEN": "fixture-token",
                "CASK_REQUESTS": str(captures),
            }
            probe = subprocess.run(
                ["/bin/zsh", "-df", "-c",
                 '[[ "$(command -v curl)" = "$EXPECTED_CURL" && "$HOMEBREW_KATANA_GIT_TOKEN" = "fixture-token" ]]'],
                cwd=root, env=environment, capture_output=True, check=False, timeout=10,
            )
            self.assertEqual(probe.returncode, 0, "Generator fixture environment is not isolated")
            result = subprocess.run(
                ["/bin/zsh", "-df", str(updater_source), "v0.22.43", "ab" * 32, "KatanA-Desktop-0.22.43.dmg"],
                cwd=root,
                env=environment,
                capture_output=True,
                text=True,
                check=False,
                timeout=15,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertTrue(captures.is_file(), result.stdout + result.stderr)
            payloads = [json.loads(line) for line in captures.read_text(encoding="utf-8").splitlines()]
            self.assertEqual(len(payloads), 2)
            rendered = [base64.b64decode(item["content"]).decode("utf-8") for item in payloads]
            expected_step = (
                'postflight_steps do\n'
                '    run "/usr/bin/xattr", args: ["-cr", "{{appdir}}/KatanA Desktop.app"]\n'
                '  end'
            )
            for cask in rendered:
                self.assertIn("depends_on macos: :ventura", cask)
                self.assertIn(expected_step, cask)
                self.assertNotIn('depends_on macos: ">= :ventura"', cask)
                self.assertNotIn("postflight do", cask)
                self.assertNotIn("system_command", cask)
                description = re.search(r'^  desc "([^"]+)"', cask, re.MULTILINE).group(1)
                self.assertLessEqual(len(description), 80)
                zap_paths = re.findall(r'^    "(~/Library/[^\"]+)",', cask, re.MULTILINE)
                self.assertEqual(zap_paths, sorted(zap_paths))
                self.assertIn("https://github.com/HiroyukiFuruno/KatanA/releases/download/v0.22.43/KatanA-Desktop-0.22.43.dmg", cask)

    def test_homebrew_generator_rejects_placeholder_digest_before_network_or_auth(self) -> None:
        with tempfile.TemporaryDirectory(prefix="katana-homebrew-invalid-digest-") as temporary:
            root = Path(temporary)
            environment = {"PATH": "/usr/bin:/bin", "HOME": str(root), "ZDOTDIR": str(root)}
            for digest in ("0" * 64, "invalid", "a" * 63):
                with self.subTest(digest=digest):
                    result = subprocess.run(
                        ["/bin/zsh", "-df", str(ROOT / "scripts/release/update-homebrew.sh"),
                         "v0.22.43", digest, "KatanA-Desktop-0.22.43.dmg"],
                        cwd=root, env=environment, capture_output=True, text=True, timeout=15,
                    )
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn("SHA256", result.stderr)
                    self.assertNotIn("Fetching", result.stdout)

    def test_external_registry_failures_are_detected_without_blocking_product_release(self) -> None:
        result = subprocess.run(
            [sys.executable, "-B", str(ROOT / "scripts/release/test-sync-external.py")],
            cwd=ROOT, capture_output=True, text=True, timeout=60,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def run_check_pr_ready_fixture(self, branch: str, cargo_version: str) -> subprocess.CompletedProcess[str]:
        # hook由来のGit環境や個人設定を隔離し、fixtureの操作が実作業repoへ向かないようにする。
        environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        environment["GIT_CONFIG_NOSYSTEM"] = "1"
        environment["GIT_CONFIG_GLOBAL"] = os.devnull
        environment["GITHUB_ACTIONS"] = "false"
        temporary_root = ROOT / "tmp"
        temporary_root.mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="katana-readiness-selector-", dir=temporary_root) as temporary:
            root = Path(temporary)
            subprocess.run(
                ["git", "init", "--quiet", "--initial-branch", branch, str(root)],
                check=True,
                capture_output=True,
                text=True,
                env=environment,
            )
            release = root / "scripts/release"
            release.mkdir(parents=True)
            for name in ("check-pr-ready.sh", "extract-release-branch-version.sh", "select-pr-readiness-mode.sh"):
                shutil.copy2(ROOT / "scripts/release" / name, release / name)
                (release / name).chmod(0o755)
            (root / "Cargo.toml").write_text(f'[package]\nname = "fixture"\nversion = "{cargo_version}"\n', encoding="utf-8")
            return subprocess.run(
                ["bash", "scripts/release/check-pr-ready.sh", "--pr-bootstrap"],
                cwd=root,
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                timeout=10,
            )

    def test_private_readiness_mode_selector_matrix(self) -> None:
        selector = "scripts/release/select-pr-readiness-mode.sh"
        matrix = (
            ("release/v0.22.42-public-release-evidence", "0.22.42", "", "post-release-automated"),
            ("release/v0.22.42-public-release-evidence", "0.22.42", "pr-bootstrap", "post-release-automated"),
            ("release/v0.22.42-public-release-evidence", "0.22.42", "strict", "strict"),
            ("release/v0.22.42-public-release-evidence", "0.22.42", "automated-release", "automated-release"),
            ("release/v0.22.42-public-release-evidence", "0.22.42", "release-artifact-pending", "release-artifact-pending"),
            ("release/v0.22.42-public-release-evidence", "0.22.42", "post-release-evidence", "post-release-evidence"),
            ("release/v0.22.42-public-release-evidence", "0.22.42", "post-release-host", "post-release-host"),
            ("release/v0.22.42", "0.22.42", "pr-bootstrap", "pr-bootstrap"),
            ("release/v0.22.42-public-release-evidence-extra", "0.22.42", "pr-bootstrap", "pr-bootstrap"),
            ("docs/v0.22.42-public-release-evidence", "0.22.42", "pr-bootstrap", "pr-bootstrap"),
            ("release/v0.22.43-public-release-evidence", "0.22.43", "", ""),
            ("release/v0.22.42-public-release-evidence", "0.22.43", "", ""),
        )
        for branch, target, requested, expected in matrix:
            with self.subTest(branch=branch, target=target, requested=requested):
                result = subprocess.run(
                    ["bash", selector, branch, target, requested],
                    cwd=ROOT,
                    capture_output=True,
                    text=True,
                    check=False,
                    timeout=5,
                )
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(result.stdout, expected + "\n")

    def test_readiness_selector_is_before_existing_version_guard(self) -> None:
        source = (ROOT / "scripts/release/check-pr-ready.sh").read_text(encoding="utf-8")
        assignment = 'TASK_GATE_MODE=$(bash "$READINESS_MODE_SELECTOR" "$CURRENT_BRANCH" "$TARGET_VERSION" "$TASK_GATE_MODE")'
        matches = list(re.finditer(r"(?m)^" + re.escape(assignment) + r"$", source))
        self.assertEqual(len(matches), 1)
        self.assertGreater(matches[0].start(), source.index("# 3. Determine Target Version"))
        self.assertLess(matches[0].start(), source.index("# 4. Check for Consistency"))
        self.assertLess(
            matches[0].start(),
            source.index("# 5. Version increment or published-release guard"),
        )
        self.assertIn("gh release view", source)
        self.assertIn("draft=false prerelease=false", source)

    def test_selector_preserves_version_rejection_for_exact_and_ordinary_branches(self) -> None:
        exact = self.run_check_pr_ready_fixture(
            "release/v0.22.42-public-release-evidence",
            "0.22.43",
        )
        ordinary = self.run_check_pr_ready_fixture("release/v0.22.43", "0.22.42")
        self.assertEqual(exact.returncode, 1, exact.stdout + exact.stderr)
        self.assertIn("does not match target version", exact.stderr)
        self.assertEqual(ordinary.returncode, 1, ordinary.stdout + ordinary.stderr)
        self.assertIn("does not match target version", ordinary.stderr)

    def test_post_release_manifest_checks_execute_actual_tag_run_attempt_binding(self) -> None:
        workflow = (ROOT / ".github/workflows/release-readiness.yml").read_text(encoding="utf-8")
        match = re.search(r"<<'PY'\n(.*?)\n          PY", workflow, re.S)
        self.assertIsNotNone(match)
        program = textwrap.dedent(match.group(1))
        for platform in ("macos", "linux", "windows"):
            for field in (None, "run", "attempt", "source"):
                with self.subTest(platform=platform, changed=field), tempfile.TemporaryDirectory() as temporary:
                    root = Path(temporary)
                    for name in ("macos", "linux", "windows"):
                        record = {"github_run_id": "321", "github_run_attempt": "2",
                                  "provenance": {"source_commit": "a" * 40}}
                        if name == platform:
                            if field == "run":
                                record["github_run_id"] = "320"
                            elif field == "attempt":
                                record["github_run_attempt"] = "1"
                            elif field == "source":
                                record["provenance"]["source_commit"] = "b" * 40
                        path = root / f"artifacts-identity-{name}/identity.json"
                        path.parent.mkdir()
                        path.write_text(json.dumps(record), encoding="utf-8")
                    result = subprocess.run([sys.executable, "-B", "-c", program, str(root), "321", "2", "a" * 40],
                                            capture_output=True, text=True, timeout=10)
                    if field is None:
                        self.assertEqual(result.returncode, 0, result.stderr)
                    else:
                        self.assertNotEqual(result.returncode, 0)
                        self.assertIn("actual tag/run/attempt", result.stderr)

    def test_post_release_route_checks_real_assets_from_the_actual_tag_build(self) -> None:
        workflow = (ROOT / ".github/workflows/release-readiness.yml").read_text(encoding="utf-8")
        verify_post_publication_artifact_route(workflow)
        for before, after in (
            ("--phase postpublication", "--phase prepublish"),
            ('--assets "$work/assets"', '--assets "$work"'),
            ('--source "$tag_commit"', "--branch master"),
            ('gh run download "$build_run"', "gh run download"),
            ("--name artifacts-identity-windows", ""),
            ("(build_run, build_attempt, tag_commit)", "(build_run, build_attempt, 'unverified')"),
            ("      - name: Verify actual published release artifacts\n        if: github.head_ref == 'release/v0.22.42-public-release-evidence'",
             "      - name: Verify actual published release artifacts\n        if: false"),
            ("  actions: read\n", ""),
        ):
            with self.subTest(marker=before), self.assertRaises(ValueError):
                verify_post_publication_artifact_route(workflow.replace(before, after, 1))

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
        self.assertIn("python3 -B -m unittest discover -s scripts/ci -p test_zstd_advisory_policy.py", source)
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

    def test_post_release_automatic_route_retains_existing_published_release_guard(self) -> None:
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
            '"$TASK_GATE_MODE" == "post-release-host" || "$TASK_GATE_MODE" == "post-release-automated" ]]; then',
            check_pr_ready,
        )
        self.assertIn('gh release view "v${TARGET_VERSION}"', check_pr_ready)
        self.assertIn('"v${TARGET_VERSION} draft=false prerelease=false"', check_pr_ready)
        self.assertIn('KATANA_OPENSPEC_TASK_GATE="$TASK_GATE_MODE" ./scripts/release/preflight.sh', check_pr_ready)
        self.assertEqual(run_readiness_routing(readiness, "release/v0.22.42-public-release-evidence"),
                         ["0.22.42", "--post-release-automated"])

    def test_only_exact_fixed_release_evidence_branch_uses_post_automated(self) -> None:
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
                expected = "--automated-release" if version == "0.22.42" else "--pr-bootstrap"
                self.assertEqual(run_readiness_routing(readiness, branch), [version, expected])

    def test_legacy_docs_evidence_route_is_unchanged(self) -> None:
        readiness = (ROOT / ".github/workflows/release-readiness.yml").read_text(encoding="utf-8")
        for branch in ("docs/v0.22.42-public-release-evidence", "docs/v0.22.41-public-release-evidence"):
            with self.subTest(branch=branch):
                self.assertEqual(run_readiness_routing(readiness, branch), ["0.22.42", "--post-release-evidence"])

    def test_readiness_checkout_keeps_recorded_build_commit_available(self) -> None:
        readiness = (ROOT / ".github/workflows/release-readiness.yml").read_text(encoding="utf-8")
        checkout = readiness.split("uses: actions/checkout@", 1)[1].split("      - uses:", 1)[0]
        self.assertIn("with:\n          fetch-depth: 0\n", checkout)

    def test_real_workflow_requires_automatic_quality_and_all_assets_before_publication(self) -> None:
        workflow = (ROOT / ".github/workflows/build-and-release.yml").read_text(encoding="utf-8")
        verify_document_phase_order(workflow)
        for altered in (
            workflow.replace("--automated-release", "--release-artifact-pending"),
            workflow.replace(" automated-release --root .", " source --root ."),
            workflow.replace("--scope automated-release", "--scope katana-host"),
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
