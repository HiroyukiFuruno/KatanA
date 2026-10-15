"""Real temporary Git repositories; not full public-release cleanup acceptance."""

from pathlib import Path
import tempfile
import unittest

from release_cleanup_audit import CleanupRefused, audit_branch, git, require_controller, synchronize_default


class CleanupNativeGitTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name) / "source"
        self.remote = Path(self.temporary.name) / "remote.git"
        self.root.mkdir()
        git(self.root, "init", "--initial-branch=master")
        git(self.root, "config", "user.email", "cleanup@example.invalid")
        git(self.root, "config", "user.name", "Cleanup fixture")
        git(self.root, "config", "commit.gpgsign", "false")
        git(self.root, "commit", "--allow-empty", "-m", "initial")
        git(self.root, "init", "--bare", str(self.remote))
        git(self.root, "remote", "add", "origin", str(self.remote))
        git(self.root, "push", "origin", "master")
        self.oid = git(self.root, "rev-parse", "HEAD").decode().strip()
        git(self.root, "branch", "release/completed")

    def test_clean_existing_default_sync_and_merged_unused_candidate(self):
        self.assertEqual(synchronize_default(self.root, self.root, "master"), self.oid)
        record = audit_branch(self.root, "master", "release/completed", self.oid)
        self.assertEqual(record["state"], "eligible")
        self.assertEqual(git(self.root, "rev-parse", "refs/heads/release/completed").decode().strip(), self.oid)

    def test_dirty_untracked_controller_is_preserved(self):
        (self.root / "untracked.txt").write_text("keep")
        with self.assertRaises(CleanupRefused):
            synchronize_default(self.root, self.root, "master")
        self.assertEqual((self.root / "untracked.txt").read_text(), "keep")

    def test_tracked_change_and_stash_are_not_hidden(self):
        (self.root / "tracked.txt").write_text("initial")
        git(self.root, "add", "tracked.txt")
        git(self.root, "commit", "-m", "tracked")
        (self.root / "tracked.txt").write_text("changed")
        with self.assertRaises(CleanupRefused):
            require_controller(self.root, self.root, "master")
        git(self.root, "stash", "push", "-m", "fixture existing stash")
        with self.assertRaises(CleanupRefused):
            require_controller(self.root, self.root, "master")
        self.assertTrue(git(self.root, "stash", "list"))

    def test_checked_out_target_is_retained_without_switching_it(self):
        git(self.root, "switch", "release/completed")
        record = audit_branch(self.root, "master", "release/completed", self.oid)
        self.assertEqual(record["state"], "retain")
        with self.assertRaises(CleanupRefused):
            synchronize_default(self.root, self.root, "master")
        self.assertEqual(git(self.root, "symbolic-ref", "HEAD").strip(), b"refs/heads/release/completed")

    def test_unmerged_target_and_changed_oid_are_retained(self):
        git(self.root, "switch", "release/completed")
        git(self.root, "commit", "--allow-empty", "-m", "unmerged")
        new = git(self.root, "rev-parse", "HEAD").decode().strip()
        git(self.root, "switch", "master")
        with self.assertRaises(CleanupRefused):
            audit_branch(self.root, "master", "release/completed", self.oid)
        self.assertEqual(audit_branch(self.root, "master", "release/completed", new)["state"], "retain")

    def test_default_nonrelease_and_option_targets_are_rejected(self):
        for target in ("master", "main", "feature/other", "--delete"):
            with self.subTest(target=target), self.assertRaises(CleanupRefused):
                audit_branch(self.root, "master", target, self.oid)

    def test_unrelated_repository_and_nested_root_are_rejected(self):
        other = Path(self.temporary.name) / "other"
        other.mkdir()
        git(other, "init", "--initial-branch=master")
        with self.assertRaises(CleanupRefused):
            require_controller(self.root, other, "master")
        nested = self.root / "nested"
        nested.mkdir()
        with self.assertRaises(CleanupRefused):
            require_controller(nested, self.root, "master")

    def test_remote_fast_forward_and_divergence_use_native_git(self):
        clone = Path(self.temporary.name) / "clone"
        git(self.root, "clone", str(self.remote), str(clone))
        git(clone, "config", "user.email", "cleanup@example.invalid")
        git(clone, "config", "user.name", "Cleanup fixture")
        git(clone, "config", "commit.gpgsign", "false")
        git(clone, "commit", "--allow-empty", "-m", "remote advance")
        git(clone, "push", "origin", "master")
        advanced = git(clone, "rev-parse", "HEAD").decode().strip()
        self.assertEqual(synchronize_default(self.root, self.root, "master"), advanced)
        git(self.root, "commit", "--allow-empty", "-m", "local advance")
        local = git(self.root, "rev-parse", "HEAD")
        git(clone, "commit", "--allow-empty", "-m", "divergent advance")
        git(clone, "push", "origin", "master")
        with self.assertRaises(CleanupRefused):
            synchronize_default(self.root, self.root, "master")
        self.assertEqual(git(self.root, "rev-parse", "HEAD"), local)


if __name__ == "__main__":
    unittest.main()
