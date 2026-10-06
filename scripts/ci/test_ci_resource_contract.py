"""Guard the test job's cache-space mitigation without parsing all YAML."""

from pathlib import Path
import re
import unittest


def test_job_block(workflow):
    match = re.search(r"(?ms)^  test:\n(.*?)(?=^  [A-Za-z][\w-]*:\n|\Z)", workflow)
    if not match:
        raise AssertionError("could not locate jobs.test block")
    return match.group(1)


def step_block(job, name):
    pattern = rf"(?ms)^      - name: {re.escape(name)}\n(.*?)(?=^      - (?:name|uses):|\Z)"
    match = re.search(pattern, job)
    if not match:
        raise AssertionError(f"could not locate test job step: {name}")
    return match.group(1)


class CiResourceContractTest(unittest.TestCase):
    def test_original_failure_sample_is_opt_in_and_cannot_replace_failure(self):
        job = current_test_job()
        tests = step_block(job, "Run tests")
        self.assertIn("KATANA_HTML_FAILURE_SAMPLE: ${{ runner.os == 'macOS' && '1' || '0' }}", tests)
        self.assertNotIn("DEBUG", tests)
        self.assertNotIn("continue-on-error", tests)
        upload = step_block(job, "Upload original HTML failure sample (macOS)")
        self.assertIn("failure()", upload)
        self.assertIn("steps.workspace_tests.outcome == 'failure'", upload)
        self.assertIn("runner.os == 'macOS'", upload)
        self.assertIn("path: target/html-startup-failures", upload)
        self.assertNotIn("cargo test", upload)

    def test_html_failure_trace_preserves_the_original_gate(self):
        job = current_test_job()
        tests = step_block(job, "Run tests")
        self.assertIn("id: workspace_tests", tests)
        self.assertNotIn("continue-on-error", tests)
        self.assertNotIn("DEBUG", tests)
        diagnostic = step_block(job, "Trace failed HTML startup (macOS)")
        self.assertIn("failure()", diagnostic)
        self.assertIn("steps.workspace_tests.outcome == 'failure'", diagnostic)
        self.assertIn("runner.os == 'macOS'", diagnostic)
        self.assertIn("DEBUG: 'true'", diagnostic)
        self.assertIn("scripts/ci/with-office-test-worker.sh", diagnostic)
        self.assertIn("cargo test --locked -p katana-ui --lib", diagnostic)
        self.assertIn(
            "app::action::html_navigation::tests::"
            "file_navigation_to_an_open_dirty_target_preserves_target_state",
            diagnostic,
        )
        self.assertIn("-- --exact --nocapture", diagnostic)
        self.assertNotIn("continue-on-error", diagnostic)
        self.assertLess(job.index("- name: Run tests"), job.index("- name: Trace failed HTML startup (macOS)"))

    def test_test_concurrency_is_bounded_without_skipping_tests(self):
        root = Path(__file__).resolve().parents[2]
        justfile = (root / "Justfile").read_text(encoding="utf-8")
        self.assertIn(
            'export RUST_TEST_THREADS := env_var_or_default("RUST_TEST_THREADS", JOBS)',
            justfile,
        )
        recipes = (root / "just/tests.just").read_text(encoding="utf-8")
        linux = recipes.split("check-linux: check-linux-office-worker-contract\n", 1)[1].split("\n\n", 1)[0]
        self.assertIn('-e RUST_TEST_THREADS="{{RUST_TEST_THREADS}}"', linux)
        self.assertIn("cargo test --locked -q --workspace", linux)
        self.assertRegex(current_test_job(), r"(?m)^      RUST_TEST_THREADS: 2$")

    def test_local_gates_keep_ci_symbol_only_profile_defaults(self):
        justfile = (Path(__file__).resolve().parents[2] / "Justfile").read_text(
            encoding="utf-8"
        )
        for name, value in (
            ("CARGO_PROFILE_DEV_DEBUG", "0"),
            ("CARGO_PROFILE_TEST_DEBUG", "0"),
            ("CARGO_PROFILE_DEV_STRIP", "none"),
            ("CARGO_PROFILE_TEST_STRIP", "none"),
        ):
            self.assertIn(
                f'export {name} := env_var_or_default("{name}", "{value}")',
                justfile,
            )
            for platform in ("linux", "windows"):
                compose_path = (
                    Path(__file__).resolve().parents[2]
                    / f"platforms/{platform}/ci/compose.yml"
                )
                compose = compose_path.read_text(encoding="utf-8")
                self.assertIn(f"      - {name}\n", compose)
        self.assertNotRegex(
            justfile,
            r"export CARGO_PROFILE_(DEV|TEST)_(OPT_LEVEL|DEBUG_ASSERTIONS|OVERFLOW_CHECKS)",
        )

    def test_separate_screenshot_workspace_runs_before_acceptance(self):
        job = current_test_job()
        name = "Test screenshot harness contracts"
        contracts = step_block(job, name)
        self.assertIn("shell: bash", contracts)
        self.assertIn("CARGO_TARGET_DIR: target/screenshot-harness", contracts)
        self.assertIn(
            "cargo test --locked --release --manifest-path scripts/screenshot/Cargo.toml",
            contracts,
        )
        acceptance = step_block(job, "Run multi-format headless acceptance")
        condition = "if: github.event_name == 'pull_request' && startsWith(github.head_ref, 'release/v')"
        self.assertIn(condition, contracts)
        self.assertIn(condition, acceptance)
        self.assertLess(job.index(name), job.index("- name: Run multi-format headless acceptance"))

    def test_cold_font_diagnostic_keeps_full_suite_and_isolates_debug(self):
        job = current_test_job()
        diagnostic_name = "Measure cold real-font lookup (Windows)"
        diagnostic = step_block(job, diagnostic_name)
        self.assertIn("if: runner.os == 'Windows'", diagnostic)
        self.assertIn("DEBUG: 'true'", diagnostic)
        self.assertIn("cargo test --locked -p katana-ui --lib", diagnostic)
        self.assertIn("-- --exact --nocapture", diagnostic)
        self.assertIn(
            "preview_pane::document_surface::font_lookup::tests::"
            "real_background_worker_returns_result_with_identity_while_ui_paints",
            diagnostic,
        )
        self.assertLess(job.index(diagnostic_name), job.index("- name: Run tests"))
        self.assertNotIn("DEBUG", step_block(job, "Run tests"))
        source = Path(__file__).resolve().parents[2] / (
            "crates/katana-ui/src/preview_pane/document_surface/font_lookup_tests.rs"
        )
        self.assertIn(
            "fn real_background_worker_returns_result_with_identity_while_ui_paints()",
            source.read_text(encoding="utf-8"),
        )

    def test_space_mitigation_and_cache_namespace(self):
        job = current_test_job()
        self.assertIn("os: [macos-latest, windows-latest, ubuntu-latest]", job)

        for name, value in (
            ("CARGO_PROFILE_DEV_DEBUG", "0"),
            ("CARGO_PROFILE_TEST_DEBUG", "0"),
            ("CARGO_PROFILE_DEV_STRIP", "none"),
            ("CARGO_PROFILE_TEST_STRIP", "none"),
        ):
            self.assertRegex(job, rf"(?m)^      {name}: {value}$")

        cache = step_block(job, "Cache Cargo registry & target")
        self.assertRegex(cache, r"(?m)^          key: \$\{\{ runner.os \}\}-cargo-test-no-debug-info-v1-\$\{\{ hashFiles\('\*\*/Cargo.lock'\) \}\}$")
        self.assertRegex(cache, r"(?m)^          restore-keys: \|\n            \$\{\{ runner.os \}\}-cargo-test-no-debug-info-v1-\n\s*\Z")
        self.assertNotRegex(job, r"CARGO_PROFILE_(DEV|TEST)_(OPT_LEVEL|DEBUG_ASSERTIONS|OVERFLOW_CHECKS)")

    def test_existing_test_and_acceptance_commands_remain(self):
        job = current_test_job()
        sweep_install = step_block(job, "Install cargo-sweep")
        self.assertIn("cargo install cargo-sweep --locked", sweep_install)
        self.assertIn("cargo sweep --version", sweep_install)
        sweep_contract = step_block(job, "Verify Cargo sweep safety contract")
        self.assertIn("test_cargo_sweep_guard.py", sweep_contract)
        check_types = step_block(job, "Check types")
        self.assertIn("run: cargo check --workspace", check_types)
        tests = step_block(job, "Run tests")
        self.assertIn("bash scripts/ci/with-office-test-worker.sh 'cargo test --workspace'", tests)

        acceptance = step_block(job, "Run multi-format headless acceptance")
        self.assertIn("scripts/screenshot/run.sh", acceptance)
        self.assertIn("v0-22-38-multi-format-documents.json", acceptance)
        upload = step_block(job, "Upload multi-format headless evidence")
        self.assertIn("multi-format-headless-${{ runner.os }}", upload)

        coverage = step_block(job, "Run coverage")
        self.assertIn("runner.os == 'macOS'", coverage)
        self.assertIn("JOBS=1 just coverage", coverage)


def current_test_job():
    path = Path(__file__).resolve().parents[2] / ".github/workflows/test-and-build.yml"
    return test_job_block(path.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
