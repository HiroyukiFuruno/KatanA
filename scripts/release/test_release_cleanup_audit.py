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

    def test_remote_only_audit_requires_explicit_absent_local_permission(self):
        git(self.root, "branch", "-d", "release/completed")
        with self.assertRaises(CleanupRefused):
            audit_branch(self.root, "master", "release/completed", self.oid)
        result = audit_branch(self.root, "master", "release/completed", self.oid,
                              allow_absent=True)
        self.assertEqual(result["state"], "eligible")
        self.assertEqual(result["reason"], "merged target without a local branch")

    def test_absent_parent_does_not_select_a_different_local_child_branch(self):
        git(self.root, "branch", "-d", "release/completed")
        git(self.root, "branch", "release/completed/keep")
        result = audit_branch(self.root, "master", "release/completed", self.oid,
                              allow_absent=True)
        self.assertEqual(result["reason"], "merged target without a local branch")
        self.assertEqual(git(self.root, "rev-parse", "refs/heads/release/completed/keep").decode().strip(), self.oid)

    def test_absent_local_branch_cannot_admit_an_unmerged_remote_commit(self):
        git(self.root, "switch", "release/completed")
        git(self.root, "commit", "--allow-empty", "-m", "remote-only unmerged")
        unmerged = git(self.root, "rev-parse", "HEAD").decode().strip()
        git(self.root, "switch", "master")
        git(self.root, "update-ref", "-d", "refs/heads/release/completed", unmerged)
        result = audit_branch(self.root, "master", "release/completed", unmerged,
                              allow_absent=True)
        self.assertEqual(result["state"], "retain")

    def test_absent_local_target_still_requires_a_full_commit_oid(self):
        git(self.root, "branch", "-d", "release/completed")
        for expected in (self.oid[:12], "--help", "f" * len(self.oid)):
            with self.subTest(expected=expected), self.assertRaises(CleanupRefused):
                audit_branch(self.root, "master", "release/completed", expected,
                             allow_absent=True)

    def test_sha256_absent_local_target_rejects_a_resolvable_40_character_prefix(self):
        root = Path(self.temporary.name) / "sha256"
        root.mkdir()
        git(root, "init", "--object-format=sha256", "--initial-branch=master")
        git(root, "-c", "commit.gpgsign=false", "-c", "user.name=Fixture",
            "-c", "user.email=fixture@example.invalid", "commit", "--allow-empty", "-m", "sha256")
        oid = git(root, "rev-parse", "HEAD").decode().strip()
        self.assertEqual(len(oid), 64)
        self.assertEqual(audit_branch(root, "master", "release/missing", oid,
                                      allow_absent=True)["state"], "eligible")
        with self.assertRaises(CleanupRefused):
            audit_branch(root, "master", "release/missing", oid[:40], allow_absent=True)

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
