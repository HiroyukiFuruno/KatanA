"""Native empty-scope prune and retention of unknown administrative state."""

from pathlib import Path
import tempfile
import unittest

from cleanup_prune import prune_empty_registry
from release_cleanup_audit import CleanupRefused, common_directory, git


class CleanupPruneTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        git(self.root, "init", "--initial-branch=master")
        git(self.root, "-c", "commit.gpgsign=false", "-c", "user.name=Fixture",
            "-c", "user.email=fixture@example.invalid", "commit", "--allow-empty", "-m", "seed")

    def test_single_checkout_prune_preserves_default_and_commit(self):
        before = git(self.root, "rev-parse", "HEAD")
        prune_empty_registry(self.root)
        self.assertEqual(git(self.root, "rev-parse", "HEAD"), before)
        self.assertEqual(git(self.root, "symbolic-ref", "HEAD").strip(), b"refs/heads/master")

    def test_unknown_registry_entry_is_retained_without_interpreting_it(self):
        registry = common_directory(self.root) / "worktrees"
        registry.mkdir()
        unknown = registry / "unknown"
        unknown.mkdir()
        (unknown / "evidence").write_text("keep")
        with self.assertRaises(CleanupRefused):
            prune_empty_registry(self.root)
        self.assertEqual((unknown / "evidence").read_text(), "keep")

    def test_registry_symlink_is_refused_and_target_is_preserved(self):
        outside = self.root / "outside"
        outside.mkdir()
        (outside / "keep").write_text("evidence")
        registry = common_directory(self.root) / "worktrees"
        registry.symlink_to(outside, target_is_directory=True)
        with self.assertRaises(CleanupRefused):
            prune_empty_registry(self.root)
        self.assertEqual((outside / "keep").read_text(), "evidence")


if __name__ == "__main__":
    unittest.main()
