"""SemVer boundaries and independent ASCII identifier grammar checks."""

from itertools import product
import unittest

from downstream_evidence_fields import SEMVER_RE


def identifier_is_valid(identifier: str) -> bool:
    allowed = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz-"
    if not identifier or any(char not in allowed for char in identifier):
        return False
    return not identifier.isdigit() or identifier == "0" or not identifier.startswith("0")


class SemVerTests(unittest.TestCase):
    def test_boundary_versions(self) -> None:
        accepted = (
            "0.0.0", "1.2.3-0", "1.2.3-00a", "1.2.3-01-", "1.2.3--",
            "1.2.3-a0a", "1.2.3-a.0", "1.2.3+build.01",
        )
        rejected = (
            "01.2.3", "1.02.3", "1.2.03", "1.2.3-01", "1.2.3-",
            "1.2.3-a.01", "1.2.3-a..b", "1.2.3+",
        )
        for version in accepted:
            with self.subTest(version=version):
                self.assertIsNotNone(SEMVER_RE.fullmatch(version))
        for version in rejected:
            with self.subTest(version=version):
                self.assertIsNone(SEMVER_RE.fullmatch(version))

    def test_ascii_prerelease_oracle_1365_inputs(self) -> None:
        for length in range(6):
            for chars in product("01a-", repeat=length):
                identifier = "".join(chars)
                with self.subTest(identifier=identifier):
                    actual = SEMVER_RE.fullmatch(f"1.2.3-{identifier}") is not None
                    self.assertEqual(actual, identifier_is_valid(identifier))

    def test_long_invalid_tail_is_rejected(self) -> None:
        self.assertIsNone(SEMVER_RE.fullmatch("1.2.3-" + "a" * 4096 + "!"))


if __name__ == "__main__":
    unittest.main()
