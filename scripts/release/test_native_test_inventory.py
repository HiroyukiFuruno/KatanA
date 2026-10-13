"""Pure format and selection tests, not native test execution evidence."""

import unittest

from native_test_inventory import (
    InventoryFormatError, parse_inventory, selected_cases, validate_ignored_subset,
)


class InventoryTests(unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main()
