"""Real squash graphs and guarded local CAS; no public branch is removed."""

from pathlib import Path
import subprocess
import unittest

from cleanup_execution import CleanupExecutor
from cleanup_merge_binding import require_matching_merge
from cleanup_session import CleanupSession
from release_cleanup_audit import CleanupRefused, audit_branch, common_directory, git
from release_cleanup_tag import require_published_commit
import test_cleanup_execution as fixtures


class SquashBindingTests(unittest.TestCase):
    def setUp(self):
        self.fixture = fixtures.CleanupExecutionTests()
        self.fixture.setUp()
        self.addCleanup(self.fixture.temporary.cleanup)
        self.root, self.branch = self.fixture.root, self.fixture.branch
        git(self.root, "config", "tag.gpgsign", "false")
        git(self.root, "switch", self.branch)
        (self.root / "change").write_text("release content", encoding="utf-8")
        git(self.root, "add", "change")
        git(self.root, "commit", "-m", "branch content")
        self.tip = git(self.root, "rev-parse", "HEAD").decode().strip()
        git(self.root, "switch", "master")
        git(self.root, "merge", "--squash", self.branch)
        git(self.root, "commit", "-m", "squash integration")
        self.merge = git(self.root, "rev-parse", "HEAD").decode().strip()
        self.executor = CleanupExecutor(self.root, "HiroyukiFuruno/KatanA", "master",
                                        self.branch, self.tip, merge_commit=self.merge, pull_request=1)

    def test_actual_squash_requires_merge_for_publication_and_tip_for_deletion(self):
        self.assertNotEqual(self.tip, self.merge)
        self.assertEqual(git(self.root, "rev-parse", self.tip + "^{tree}"),
                         git(self.root, "rev-parse", self.merge + "^{tree}"))
        self.assertEqual(audit_branch(self.root, "master", self.branch, self.tip)["state"], "retain")
        self.assertEqual(audit_branch(self.root, "master", self.branch, self.tip,
                                     integrated_oid=self.merge)["state"], "eligible")
        remote = self.root / "published.git"
        git(self.root, "init", "--bare", str(remote))
        git(self.root, "tag", "v1.0.0")
        git(self.root, "push", str(remote), "refs/tags/v1.0.0")
        with self.assertRaisesRegex(CleanupRefused, "not included"):
            require_published_commit(self.root, str(remote), "v1.0.0", self.tip)
        self.assertEqual(require_published_commit(self.root, str(remote), "v1.0.0", self.merge), self.merge)

    def test_actual_squashed_local_tip_deletes_through_existing_reference_owner(self):
        hook = common_directory(self.root) / "hooks/reference-transaction"
        hook.write_text('#!/bin/sh\nprintf "%s\\n" "$1" >> "$(git rev-parse --git-common-dir)/owner-log"\n')
        hook.chmod(0o700)
        self.executor.prepare_hooks(include_remote=False)
        with CleanupSession(self.root) as session:
            self.assertEqual(self.executor.delete_local(session), "deleted")
        self.assertEqual(git(self.root, "for-each-ref", "--format=%(refname)",
                             "refs/heads/" + self.branch), b"")
        self.assertEqual(git(self.root, "rev-parse", "HEAD").decode().strip(), self.merge)
        self.assertIn("prepared", (common_directory(self.root) / "owner-log").read_text().splitlines())

    def test_existing_owner_failure_keeps_squashed_tip(self):
        hook = common_directory(self.root) / "hooks/reference-transaction"
        hook.write_text("#!/bin/sh\nexit 23\n")
        hook.chmod(0o700)
        self.executor.prepare_hooks(include_remote=False)
        with CleanupSession(self.root) as session, self.assertRaises(subprocess.CalledProcessError):
            self.executor.delete_local(session)
        self.assertEqual(git(self.root, "rev-parse", "refs/heads/" + self.branch).decode().strip(), self.tip)

    def test_changed_tip_and_in_use_branch_are_still_retained(self):
        git(self.root, "update-ref", "refs/heads/" + self.branch, self.merge, self.tip)
        with self.assertRaises(CleanupRefused):
            audit_branch(self.root, "master", self.branch, self.tip, integrated_oid=self.merge)
        git(self.root, "update-ref", "refs/heads/" + self.branch, self.tip, self.merge)
        git(self.root, "switch", self.branch)
        self.assertEqual(audit_branch(self.root, "master", self.branch, self.tip,
                                     integrated_oid=self.merge)["state"], "retain")

    def test_fresh_default_consumer_does_not_require_deleted_branch_object(self):
        remote = self.root / "published.git"
        git(self.root, "init", "--bare", str(remote))
        git(self.root, "push", str(remote), "master")
        consumer = self.root / "consumer"
        git(self.root, "clone", "--single-branch", "--branch", "master", str(remote), str(consumer))
        with self.assertRaises(CleanupRefused):
            git(consumer, "cat-file", "-t", self.tip)
        self.assertEqual(audit_branch(consumer, "master", self.branch, self.tip,
                                     allow_absent=True, integrated_oid=self.merge)["state"], "eligible")

    def test_server_binding_rejects_wrong_pr_head_branch_or_merge(self):
        target = {"branch": self.branch, "expected": self.tip,
                  "merge_commit": self.merge, "pull_request": 1}
        require_matching_merge(target, self.branch, self.tip, self.merge, 1)
        for field, value in (("branch", "release/other"), ("expected", "a" * 40),
                             ("merge_commit", "b" * 40), ("pull_request", 2)):
            with self.subTest(field=field), self.assertRaises(CleanupRefused):
                require_matching_merge({**target, field: value}, self.branch, self.tip, self.merge, 1)


if __name__ == "__main__":
    unittest.main()
