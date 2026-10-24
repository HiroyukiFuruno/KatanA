"""Keep original test gates and phase selections beside collect-only metadata."""

from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]


class InventoryWiringTests(unittest.TestCase):
    def test_tool_installers_share_the_pinned_coverage_compatible_revision(self) -> None:
        for filename, expected_count in (("release-readiness.yml", 1),
                                         ("build-and-release.yml", 2),
                                         ("test-and-build.yml", 3)):
            with self.subTest(workflow=filename):
                source = (ROOT / ".github/workflows" / filename).read_text()
                revisions = re.findall(r"uses: taiki-e/install-action@([^\s]+)", source)
                self.assertEqual(revisions, ["1e29709c0ec63babbe8f266397a465f29a825f91"] * expected_count)

    def test_windows_native_listing_has_a_bounded_standard_pipe_reader(self) -> None:
        source = (ROOT / "scripts/release/native_test_inventory_collection.py").read_text()
        self.assertIn("from native_test_pipe_reader import read_pipes", source)
        self.assertIn("read_pipes(process, MAX_STREAM_BYTES, TIMEOUT_SECONDS)", source)
        self.assertNotIn("collector requires POSIX selector pipes", source)
        self.assertIn("Python 3.12", source)
        self.assertIn('if os.name == "posix":', source)

    def test_ci_compares_observed_inputs_without_replacing_existing_test_gates(self) -> None:
        source = (ROOT / ".github/workflows/test-and-build.yml").read_text()
        start = source.index("- name: Compare observed test inventory diagnostics")
        end = source.index("- name: Upload observed test inventory diagnostics")
        compare = source[start:end]
        self.assertIn("if: always() && runner.os == 'macOS'", compare)
        self.assertIn("python3 -B scripts/release/compare-test-invocations.py", compare)
        self.assertIn('--directory "$KATANA_TEST_INVENTORY_ROOT"', compare)
        self.assertLess(source.index("- name: Run coverage"), start)
        self.assertNotIn("continue-on-error", compare)
        self.assertNotIn("cargo test", compare)
        self.assertNotIn("cargo llvm-cov", compare)

    def test_ci_inventory_uses_current_run_and_attempt_not_restored_cache_records(self) -> None:
        source = (ROOT / ".github/workflows/test-and-build.yml").read_text()
        self.assertIn("KATANA_TEST_INVENTORY_ROOT: target/test-inventory/${{ github.run_id }}-${{ github.run_attempt }}", source)
        self.assertIn("KATANA_TEST_INVENTORY_OUTPUT: ${{ env.KATANA_TEST_INVENTORY_ROOT }}/normal.json", source)
        self.assertIn("KATANA_TEST_INVENTORY_DIRECTORY: ${{ env.KATANA_TEST_INVENTORY_ROOT }}/coverage", source)
        self.assertIn("path: ${{ env.KATANA_TEST_INVENTORY_ROOT }}", source)
        self.assertNotIn("KATANA_TEST_INVENTORY_OUTPUT: target/test-inventory/normal.json", source)

    def test_preflight_jobs_install_real_coverage_contract_tools(self) -> None:
        for filename, name in (("release-readiness.yml", "release-check"),
                               ("build-and-release.yml", "preflight")):
            with self.subTest(workflow=filename):
                source = (ROOT / ".github/workflows" / filename).read_text()
                body = source.split(f"\n  {name}:\n", 1)[1]
                job = re.split(r"\n  [a-zA-Z_][\w-]*:\n", body, maxsplit=1)[0]
                self.assertIn("components: llvm-tools-preview", job)
                self.assertIn("tool: cargo-deny@0.20.2,cargo-llvm-cov@0.9.1", job)
                self.assertNotIn("~/.cargo/bin", job)
                self.assertLess(job.index("components: llvm-tools-preview"),
                                job.index("tool: cargo-deny@0.20.2,cargo-llvm-cov@0.9.1"))

    def test_normal_entry_preserves_worker_and_original_nonrecording_command(self) -> None:
        source = (ROOT / "scripts/ci/with-office-test-worker.sh").read_text()
        self.assertIn('export KATANA_KDV_OFFICE_WORKER="$OFFICE_WORKER"', source)
        self.assertIn('"$1" == \'cargo test --workspace\'', source)
        self.assertIn('--kind test -- cargo test --workspace', source)
        self.assertIn('exec bash -c "$1"', source)
        self.assertLess(source.index("export KATANA_KDV_OFFICE_WORKER"),
                        source.index("record-test-invocation.py"))

    def test_coverage_records_four_distinct_original_test_selections(self) -> None:
        source = (ROOT / "scripts/ci/coverage.sh").read_text()
        for phase, selection in (
            ("workspace-lib-bin", '--no-report --jobs "$JOBS" --workspace --lib --bins -q'),
            ("workspace-integration", '--no-report --jobs "$JOBS" --workspace --test \'*\' --exclude katana-ui -q'),
            ("ui-parallel", '--no-report --jobs "$JOBS" -p katana-ui --test ui_integration_parallel -q'),
            ("ui-serial", '--no-report --jobs 1 -p katana-ui --test ui_integration_serial -q'),
        ):
            self.assertIn(f"llvm_cov {phase} {selection}", source)
        self.assertIn('-- --test-threads="$JOBS" --skip fixture', source)
        self.assertIn('-- --test-threads=1', source)
        self.assertIn('RUSTFLAGS="${RUSTFLAGS:-} --cfg coverage" python3', source)
        self.assertIn('-- cargo llvm-cov --no-cfg-coverage "$@"', source)
        self.assertIn('cargo llvm-cov clean --workspace', source)
        self.assertIn('cargo llvm-cov report', source)

    def test_ci_keeps_original_gate_and_uploads_diagnostics_even_after_failure(self) -> None:
        source = (ROOT / ".github/workflows/test-and-build.yml").read_text()
        self.assertIn("bash scripts/ci/with-office-test-worker.sh 'cargo test --workspace'", source)
        self.assertIn("JOBS=1 just coverage", source)
        start = source.index("- name: Upload observed test inventory diagnostics")
        upload = source[start:source.index("\n  lint:", start)]
        self.assertIn("if: always()", upload)
        self.assertIn("path: ${{ env.KATANA_TEST_INVENTORY_ROOT }}", upload)
        self.assertNotIn("continue-on-error", source)
        self.assertIn("!github.event.pull_request.draft", source)


if __name__ == "__main__":
    unittest.main()
