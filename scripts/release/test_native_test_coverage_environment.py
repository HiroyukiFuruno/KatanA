"""Pure parser contracts for the installed cargo-llvm-cov show-env format."""

import unittest

from native_test_coverage_environment import parse_show_environment


def valid(prefix: str = "") -> bytes:
    separator = "\x1f"
    return (f"export LLVM_PROFILE_FILE='{prefix}/%m.profraw'\n"
            "export __CARGO_LLVM_COV_RUSTC_WRAPPER=1\n"
            f"export __CARGO_LLVM_COV_RUSTC_WRAPPER_RUSTFLAGS='-C{separator}instrument-coverage'\n"
            "export __CARGO_LLVM_COV_RUSTC_WRAPPER_CRATE_NAMES='katana'\n"
            "export RUSTC_WRAPPER='/tool/wrapper'\n"
            "export CARGO_LLVM_COV=1\n"
            "export CARGO_LLVM_COV_SHOW_ENV=1\n"
            "export CARGO_LLVM_COV_TARGET_DIR='target/llvm-cov-target'\n"
            "export CARGO_LLVM_COV_BUILD_DIR='target/llvm-cov-target'\n").encode()


class CoverageEnvironmentTests(unittest.TestCase):
    def test_real_091_shape_and_parent_are_preserved(self) -> None:
        parsed = parse_show_environment(valid("target dir"), {
            "CARGO_TARGET_DIR": "caller-target", "RUSTFLAGS": "caller-flags"})
        self.assertEqual(parsed["CARGO_TARGET_DIR"], "caller-target")
        self.assertEqual(parsed["RUSTFLAGS"], "caller-flags")
        self.assertEqual(parsed["CARGO_LLVM_COV"], "1")

    def test_quoted_values_and_unset_are_applied_without_mutating_parent(self) -> None:
        parent = {"CARGO_TARGET_DIR": "target", "RUSTFLAGS": "keep", "RUSTDOCFLAGS": "remove"}
        raw = valid("target dir").replace(b"target dir/%m.profraw", b"$(touch SHOULD_NOT_EXIST)\nprofile")
        parsed = parse_show_environment(raw + b"unset RUSTDOCFLAGS\n", parent)
        self.assertEqual(parent, {"CARGO_TARGET_DIR": "target", "RUSTFLAGS": "keep", "RUSTDOCFLAGS": "remove"})
        self.assertEqual(parsed["LLVM_PROFILE_FILE"], "$(touch SHOULD_NOT_EXIST)\nprofile")
        self.assertNotIn("RUSTDOCFLAGS", parsed)

    def test_rejects_unknown_malformed_duplicate_and_unsafe_input(self) -> None:
        for raw in (valid() + b"export PATH=/tmp\n", valid() + valid(),
                    valid() + b"echo unsafe\n", valid() + b"export A='unterminated\n",
                    valid() + b"unset PATH\n",
                    valid().replace(b"__CARGO_LLVM_COV_RUSTC_WRAPPER=1", b"__CARGO_LLVM_COV_RUSTC_WRAPPER=/tool/wrapper"),
                    valid().replace(b"instrument-coverage", b"instrument-coverage=off"),
                    valid().replace(b"instrument-coverage", b"false"),
                    valid().replace(b"instrument-coverage", b"instrument-coverage\x1f-C\x1finstrument-coverage=off"),
                    valid().replace(b"instrument-coverage", b"instrument-coverage\x1f-Cinstrument-coverage=off"),
                    valid().replace(b"__CARGO_LLVM_COV_RUSTC_WRAPPER=1", b"__CARGO_LLVM_COV_RUSTC_WRAPPER=0")):
            with self.subTest(raw=raw[-30:]), self.assertRaises(ValueError):
                parse_show_environment(raw, {})

    def test_inherited_contract_cannot_replace_missing_output(self) -> None:
        inherited = parse_show_environment(valid(), {})
        for raw in (b"", valid().replace(b"export RUSTC_WRAPPER='/tool/wrapper'\n", b"")):
            with self.subTest(raw=raw), self.assertRaises(ValueError):
                parse_show_environment(raw, inherited)

    def test_rejects_missing_instrumentation_contract_and_limits(self) -> None:
        for raw in (valid().replace(b"CARGO_LLVM_COV=1", b"CARGO_LLVM_COV=0"),
                    valid().replace(b"LLVM_PROFILE_FILE='/%m.profraw'", b"LLVM_PROFILE_FILE=''"),
                    valid().replace(b"-C\x1finstrument-coverage", b""),
                    valid() + b"\x00", b"x" * (64 * 1024 + 1), b"\xff"):
            with self.subTest(size=len(raw)), self.assertRaises(ValueError):
                parse_show_environment(raw, {})


if __name__ == "__main__":
    unittest.main()
