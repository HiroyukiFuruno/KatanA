"""Destination identity parsing, without pretending to perform live lookup."""

import unittest

from push_repository_context import remote_repository


class PushRepositoryIdentityTests(unittest.TestCase):
    def test_https_and_ssh_identify_same_repository(self):
        self.assertEqual(remote_repository("https://github.com/HiroyukiFuruno/KatanA.git"), "HiroyukiFuruno/KatanA")
        self.assertEqual(remote_repository("git@github.com:HiroyukiFuruno/KatanA.git"), "HiroyukiFuruno/KatanA")

    def test_untrusted_host_and_ambiguous_identity_are_rejected(self):
        for url in ("https://evil.example/HiroyukiFuruno/KatanA.git",
                    "https://github.com/../KatanA.git", "https://github.com/HiroyukiFuruno/KatanA.git?other=1",
                    "file:///tmp/repo.git", "https://github.com/HiroyukiFuruno/KatanA/other.git"):
            with self.subTest(url=url), self.assertRaises(ValueError):
                remote_repository(url)


if __name__ == "__main__":
    unittest.main()
