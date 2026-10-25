"""Pure format and selection tests, not native test execution evidence."""

import unittest
from contextlib import chdir
import hashlib
import os
from pathlib import Path
import sys
import shutil
import tempfile
from dataclasses import replace

from native_test_inventory import (
    InventoryFormatError, parse_inventory, selected_cases, validate_ignored_subset,
)
from native_test_inventory_collection import (
    MAX_STREAM_BYTES, InventoryExecutionError, _resolve_command, collect_inventory,
)
from native_test_inventory_record import InventoryConditions, InventoryRecord, compare_inventory


class InventoryTests(unittest.TestCase):
    def measured_conditions(self) -> InventoryConditions:
        return InventoryConditions(source_commit="a" * 40, os_identity="macOS",
                                   target="aarch64-apple-darwin", toolchain="rustc 1.90",
                                   features=(), rustflags="", jobs=2, test_threads=2,
                                   worker_identity="worker-a", binary_sha256="b" * 64,
                                   measured=True)

    def record(self, raw: str, conditions: InventoryConditions | None = None) -> InventoryRecord:
        inventory = parse_inventory(raw)
        raw_bytes = raw.encode()
        return InventoryRecord("test", raw_bytes, hashlib.sha256(raw_bytes).hexdigest(),
                               ("native", "--list"), inventory,
                               conditions or InventoryConditions())

    def test_preserves_names_and_kinds_with_consistent_footer(self) -> None:
        result = parse_inventory("alpha::one: test\nbeta: benchmark\n\n1 test, 1 benchmark\n")
        self.assertEqual([(case.name, case.kind) for case in result.cases],
                         [("alpha::one", "test"), ("beta", "benchmark")])
        self.assertEqual(result.footer_counts, (1, 1))

    def test_accepts_zero_cases_and_empty_ignored_listing(self) -> None:
        complete = parse_inventory("alpha: test\n1 test, 0 benchmarks\n")
        ignored = parse_inventory("\n0 tests, 0 benchmarks\n")
        validate_ignored_subset(complete, ignored)
        self.assertEqual(ignored.cases, ())

    def test_rejects_unknown_duplicate_and_conflicting_entries(self) -> None:
        for raw in ("unknown\n", "alpha: test\nalpha: test\n",
                    "alpha: test\nalpha: benchmark\n"):
            with self.subTest(raw=raw), self.assertRaises(InventoryFormatError):
                parse_inventory(raw, require_footer=False)

    def test_rejects_wrong_duplicate_or_nonterminal_footer(self) -> None:
        for raw in ("alpha: test\n0 tests, 0 benchmarks\n",
                    "0 tests, 0 benchmarks\n0 tests, 0 benchmarks\n",
                    "1 test, 0 benchmarks\nalpha: test\n"):
            with self.subTest(raw=raw), self.assertRaises(InventoryFormatError):
                parse_inventory(raw)

    def test_footerless_sample_is_explicitly_unverified(self) -> None:
        with self.assertRaises(InventoryFormatError):
            parse_inventory("alpha: test\n")
        result = parse_inventory("alpha: test\n", require_footer=False)
        self.assertIsNone(result.footer_counts)
        self.assertEqual(len(result.cases), 1)

    def test_rejects_unknown_ignored_case_and_changed_kind(self) -> None:
        complete = parse_inventory("alpha: test\n1 test, 0 benchmarks\n")
        for raw in ("beta: test\n1 test, 0 benchmarks\n",
                    "alpha: benchmark\n0 tests, 1 benchmark\n"):
            with self.subTest(raw=raw), self.assertRaises(InventoryFormatError):
                validate_ignored_subset(complete, parse_inventory(raw))

    def test_skip_selection_preserves_raw_cases_and_conditions(self) -> None:
        result = parse_inventory("alpha: test\nbeta_fixture: test\n2 tests, 0 benchmarks\n")
        skips = ("fixture",)
        self.assertEqual([case.name for case in selected_cases(result, skips)], ["alpha"])
        self.assertEqual(len(result.cases), 2)
        self.assertEqual(skips, ("fixture",))

    def test_accepts_crlf_and_blank_lines(self) -> None:
        result = parse_inventory("\r\nalpha: test\r\n\r\n1 test, 0 benchmarks\r\n")
        self.assertEqual(result.footer_counts, (1, 0))

    def test_comparison_separates_case_and_unmeasured_conditions(self) -> None:
        left = self.record("alpha: test\n1 test, 0 benchmarks\n")
        right = self.record("alpha: test\nbeta: test\n2 tests, 0 benchmarks\n")
        result = compare_inventory(left, right)
        self.assertEqual([case.name for case in result.added], ["beta"])
        self.assertTrue(result.footer_changed)
        self.assertFalse(result.conditions_equivalent)
        self.assertFalse(result.equivalent)

    def test_comparison_reports_kind_and_measured_condition_changes(self) -> None:
        conditions = self.measured_conditions()
        left = self.record("alpha: test\n1 test, 0 benchmarks\n", conditions)
        right = self.record("alpha: benchmark\n0 tests, 1 benchmark\n", conditions)
        result = compare_inventory(left, right)
        self.assertEqual(result.kind_changed, (("alpha", "test", "benchmark"),))
        self.assertTrue(result.conditions_equivalent)

    def test_equivalent_requires_measured_conditions(self) -> None:
        conditions = self.measured_conditions()
        left = self.record("alpha: test\n1 test, 0 benchmarks\n", conditions)
        right = self.record("alpha: test\n1 test, 0 benchmarks\n", conditions)
        result = compare_inventory(left, right)
        self.assertTrue(result.conditions_equivalent)
        self.assertTrue(result.equivalent)

    def test_measured_flag_does_not_certify_missing_conditions(self) -> None:
        conditions = InventoryConditions(measured=True)
        left = self.record("alpha: test\n1 test, 0 benchmarks\n", conditions)
        right = self.record("alpha: test\n1 test, 0 benchmarks\n", conditions)
        result = compare_inventory(left, right)
        self.assertFalse(result.conditions_equivalent)
        self.assertFalse(result.equivalent)

    def test_conditions_reject_invalid_identity_or_numeric_fields(self) -> None:
        invalid = self.measured_conditions()
        for replacement in (
            {"source_commit": "not-an-oid"},
            {"binary_sha256": ""},
            {"jobs": True},
            {"features": ("",)},
        ):
            candidate = invalid.__class__(**{**invalid.__dict__, **replacement})
            with self.subTest(replacement=replacement):
                self.assertFalse(candidate.complete())

        self.assertFalse(self.measured_conditions().__class__(
            **{**self.measured_conditions().__dict__, "binary_sha256": 1}).complete())
        self.assertFalse(self.measured_conditions().__class__(
            **{**self.measured_conditions().__dict__, "measured": "yes"}).equivalent(self.measured_conditions()))

    def test_footerless_and_tampered_raw_records_are_not_equivalent(self) -> None:
        raw = b"alpha: test\n"
        inventory = parse_inventory(raw.decode(), require_footer=False)
        record = InventoryRecord("test", raw, hashlib.sha256(raw).hexdigest(),
                                 ("native", "--list"), inventory, self.measured_conditions())
        self.assertFalse(compare_inventory(record, record).equivalent)
        raw = b"alpha: test\n1 test, 0 benchmarks\n"
        inventory = parse_inventory(raw.decode())
        record = replace(record, raw=raw, raw_sha256=hashlib.sha256(raw).hexdigest(), inventory=inventory)
        tampered = InventoryRecord("test", raw, "0" * 64, record.argv, inventory, record.conditions)
        self.assertFalse(compare_inventory(record, tampered).equivalent)
        altered = replace(record, inventory=parse_inventory("beta: test\n1 test, 0 benchmarks\n"))
        self.assertFalse(compare_inventory(record, altered).equivalent)
        self.assertFalse(compare_inventory(altered, altered).equivalent)

    def test_collector_requires_native_list_argv_and_known_kind(self) -> None:
        conditions = InventoryConditions()
        with self.assertRaises(InventoryExecutionError):
            collect_inventory(("echo", "alpha"), Path("."), "test", conditions)
        with self.assertRaises(InventoryExecutionError):
            collect_inventory(("echo", "--list"), Path("."), "other", conditions)

    def test_collector_resolves_relative_binary_from_requested_cwd(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            candidate = root / "native-list"
            if os.name == "posix":
                candidate.symlink_to(sys.executable)
                relative = "native-list"
            else:
                relative = os.path.relpath(sys.executable, root)
            command = _resolve_command((relative, "--list"), root, "test")
            self.assertEqual(command[0], str(Path(sys.executable).resolve()))

    def test_collector_kills_real_child_after_stream_limit(self) -> None:
        command = (sys.executable, "-c", f"import sys; sys.stdout.write('x' * {MAX_STREAM_BYTES + 1})", "--list")
        with self.assertRaises(InventoryExecutionError):
            collect_inventory(command, Path("."), "test", InventoryConditions())

    def test_missing_dot_relative_binary_does_not_use_calling_cwd(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            caller, requested = root / "caller", root / "requested"
            caller.mkdir()
            requested.mkdir()
            candidate = caller / "native-list"
            shutil.copyfile(sys.executable, candidate)
            candidate.chmod(candidate.stat().st_mode | 0o111)
            with chdir(caller), self.assertRaises(InventoryExecutionError):
                _resolve_command(("./native-list", "--list"), requested, "test")

    def test_comparison_rejects_different_run_kind_or_argv(self) -> None:
        conditions = self.measured_conditions()
        left = self.record("alpha: test\n1 test, 0 benchmarks\n", conditions)
        right = InventoryRecord("coverage", left.raw, left.raw_sha256,
                                ("coverage", "--list"), left.inventory, conditions)
        result = compare_inventory(left, right)
        self.assertTrue(result.argv_changed)
        self.assertTrue(result.run_kind_changed)
        self.assertFalse(result.equivalent)


if __name__ == "__main__":
    unittest.main()
