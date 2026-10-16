"""Real Git configuration regression for publication-to-fetch identity binding."""

from pathlib import Path
import tempfile
import unittest

from release_cleanup_audit import CleanupRefused, git
from release_cleanup_publication import cleanup_destination, verify_publication_oid


class PublishedSourceIdentityTests(unittest.TestCase):
    def test_exact_publication_commit_matches_without_substitution(self):
        self.assertIsNone(verify_publication_oid("a" * 40, "a" * 40))
        self.assertIsNone(verify_publication_oid("b" * 64, "b" * 64))

    def test_short_or_other_publication_commit_is_refused(self):
        for expected in ("a" * 12, "b" * 40, "a" * 64):
            with self.subTest(expected=expected), self.assertRaises(CleanupRefused):
                verify_publication_oid("a" * 40, expected)


class CleanupDestinationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        git(self.root, "init", "--initial-branch=master")
        self.url = "https://github.com/HiroyukiFuruno/KatanA.git"
        git(self.root, "remote", "add", "origin", self.url)

    def test_exact_single_fetch_and_push_destination(self):
        self.assertEqual(cleanup_destination(self.root, self.root, "HiroyukiFuruno/KatanA"), self.url)

    def test_published_push_identity_cannot_mask_other_fetch_destination(self):
        git(self.root, "remote", "set-url", "--push", "origin", self.url)
        git(self.root, "remote", "set-url", "origin", "https://github.com/other/Repository.git")
        with self.assertRaises(CleanupRefused):
            cleanup_destination(self.root, self.root, "HiroyukiFuruno/KatanA")

    def test_multiple_destinations_are_refused(self):
        git(self.root, "remote", "set-url", "--add", "origin", self.url)
        with self.assertRaises(CleanupRefused):
            cleanup_destination(self.root, self.root, "HiroyukiFuruno/KatanA")

    def test_other_controller_configuration_is_checked_independently(self):
        controller = self.root / "controller"
        controller.mkdir()
        git(controller, "init", "--initial-branch=master")
        git(controller, "remote", "add", "origin", "https://github.com/other/Repository.git")
        with self.assertRaises(CleanupRefused):
            cleanup_destination(self.root, controller, "HiroyukiFuruno/KatanA")

    def test_two_stage_url_rewrite_is_refused_before_transport(self):
        other = "https://github.com/other/Repository.git"
        git(self.root, "config", "url." + self.url + ".insteadOf", "katana:")
        git(self.root, "config", "url." + other + ".insteadOf", self.url)
        git(self.root, "remote", "set-url", "origin", "katana:")
        self.assertEqual(git(self.root, "remote", "get-url", "origin").decode().strip(), self.url)
        git(self.root, "remote", "add", "probe", self.url)
        self.assertEqual(git(self.root, "remote", "get-url", "probe").decode().strip(), other)
        with self.assertRaises(CleanupRefused):
            cleanup_destination(self.root, self.root, "HiroyukiFuruno/KatanA")

    def test_push_only_rewrite_is_also_refused(self):
        git(self.root, "config", "url." + self.url + ".pushInsteadOf", "https://unused.example/")
        with self.assertRaises(CleanupRefused):
            cleanup_destination(self.root, self.root, "HiroyukiFuruno/KatanA")


if __name__ == "__main__":
    unittest.main()
