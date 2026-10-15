"""Keep original test gates and phase selections beside collect-only metadata."""

from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]


class InventoryWiringTests(unittest.TestCase):
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
        self.assertIn("path: target/test-inventory", upload)
        self.assertNotIn("continue-on-error", source)
        self.assertIn("!github.event.pull_request.draft", source)


if __name__ == "__main__":
    unittest.main()
