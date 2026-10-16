"""Native local deletion through the production reference hook and executor."""

from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

from cleanup_execution import CleanupExecutor
from cleanup_session import CleanupSession
from release_cleanup_audit import CleanupRefused, common_directory, git


class CleanupExecutionTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        git(self.root, "init", "--initial-branch=master")
        for key, value in (("user.name", "Fixture"), ("user.email", "fixture@example.invalid"),
                           ("commit.gpgsign", "false")):
            git(self.root, "config", key, value)
        runtime = self.root / "scripts" / "release"
        runtime.mkdir(parents=True)
        for source in Path(__file__).parent.glob("*.py"):
            if not source.name.startswith("test"):
                shutil.copyfile(source, runtime / source.name)
        git(self.root, "add", "scripts")
        git(self.root, "commit", "-m", "runtime fixture")
        self.oid = git(self.root, "rev-parse", "HEAD").decode().strip()
        self.branch = "release/fixture"
        git(self.root, "branch", self.branch)
        self.executor = CleanupExecutor(self.root, "HiroyukiFuruno/KatanA", "master",
                                        self.branch, self.oid)

    def test_local_delete_installs_and_uses_real_existing_reference_owner(self):
        hooks = common_directory(self.root) / "hooks"
        owner_log = common_directory(self.root) / "owner-log"
        hook = hooks / "reference-transaction"
        hook.write_text('#!/bin/sh\nprintf "%s\\n" "$1" >> "$(git rev-parse --git-common-dir)/owner-log"\n')
        hook.chmod(0o700)
        self.executor.prepare_hooks(include_remote=False)
        with CleanupSession(self.root) as session:
            self.assertEqual(self.executor.delete_local(session), "deleted")
        phases = owner_log.read_text().splitlines()
        self.assertIn(phases, (["prepared", "committed"],
                               ["preparing", "aborted", "prepared", "committed"]))
        self.assertEqual(git(self.root, "for-each-ref", "--format=%(refname)", "refs/heads/" + self.branch), b"")
        self.assertEqual(git(self.root, "rev-parse", "HEAD").decode().strip(), self.oid)

    def test_missing_local_target_is_reported_absent_without_creating_it(self):
        git(self.root, "branch", "-d", self.branch)
        self.executor.prepare_hooks(include_remote=False)
        with CleanupSession(self.root) as session:
            self.assertEqual(self.executor.delete_local(session), "absent")

    def test_changed_target_is_retained(self):
        git(self.root, "commit", "--allow-empty", "-m", "advance")
        changed = git(self.root, "rev-parse", "HEAD").decode().strip()
        git(self.root, "update-ref", "refs/heads/" + self.branch, changed, self.oid)
        with CleanupSession(self.root) as session, self.assertRaises(CleanupRefused):
            self.executor.delete_local(session)
        self.assertEqual(git(self.root, "rev-parse", "refs/heads/" + self.branch).decode().strip(), changed)

    def test_in_use_target_is_retained(self):
        git(self.root, "switch", self.branch)
        with CleanupSession(self.root) as session, self.assertRaises(CleanupRefused):
            self.executor.delete_local(session)
        self.assertEqual(git(self.root, "symbolic-ref", "HEAD").strip(), ("refs/heads/" + self.branch).encode())

    def test_prepared_remote_hook_owner_cannot_be_silently_replaced(self):
        hooks = common_directory(self.root) / "hooks"
        hook = hooks / "pre-push"
        hook.write_text("#!/bin/sh\nexit 0\n")
        hook.chmod(0o700)
        self.executor.prepare_hooks(include_remote=True)
        self.executor.verify_remote_hook()
        (hooks / "pre-push.katana-existing").write_text("#!/bin/sh\nexit 23\n")
        with self.assertRaises(CleanupRefused):
            self.executor.verify_remote_hook()

    def test_existing_reference_owner_failure_retains_local_branch(self):
        hooks = common_directory(self.root) / "hooks"
        hook = hooks / "reference-transaction"
        hook.write_text("#!/bin/sh\nexit 23\n")
        hook.chmod(0o700)
        self.executor.prepare_hooks(include_remote=False)
        with CleanupSession(self.root) as session, self.assertRaises(subprocess.CalledProcessError) as caught:
            self.executor.delete_local(session)
        self.assertEqual(caught.exception.returncode, 128)
        self.assertEqual(git(self.root, "rev-parse", "refs/heads/" + self.branch).decode().strip(), self.oid)

    def test_prepared_reference_owner_replacement_cannot_remove_original_rejection(self):
        hooks = common_directory(self.root) / "hooks"
        hook = hooks / "reference-transaction"
        hook.write_text("#!/bin/sh\nexit 23\n")
        hook.chmod(0o700)
        self.executor.prepare_hooks(include_remote=False)
        (hooks / "reference-transaction.katana-existing").write_text("#!/bin/sh\nexit 0\n")
        with CleanupSession(self.root) as session, self.assertRaises(CleanupRefused):
            self.executor.delete_local(session)
        self.assertEqual(git(self.root, "rev-parse", "refs/heads/" + self.branch).decode().strip(), self.oid)


if __name__ == "__main__":
    unittest.main()
