"""Native remote deletion semantics, separate from publication admission."""

import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import tempfile
import unittest

from cleanup_remote_inventory import remote_tip
from release_cleanup_audit import CleanupRefused

REFERENCE = "refs/heads/release/cleanup-fixture"
GIT_TIMEOUT_SECONDS = 30


class RemoteDeletionProtocolTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.client = self.directory / "client"
        self.remote = self.directory / "remote.git"
        self.client.mkdir()
        self.environment = {key: value for key, value in os.environ.items()
                            if not key.startswith("GIT_")}
        self.git("init", "--bare", str(self.remote))
        self.git("init", "--initial-branch=master")
        for key, value in (("user.name", "Fixture"), ("user.email", "fixture@example.invalid"),
                           ("commit.gpgsign", "false")):
            self.git("config", key, value)
        self.hooks = self.client / "owned-hooks"
        self.hooks.mkdir()
        self.git("config", "core.hooksPath", str(self.hooks))
        self.first = self.commit("first")
        self.git("branch", REFERENCE.removeprefix("refs/heads/"), self.first)
        self.second = self.commit("second")
        self.git("remote", "add", "origin", str(self.remote))
        self.git("push", "origin", "master", REFERENCE)
        self.owner_input = self.directory / "owner-input"

    def git(self, *arguments, check=True):
        return subprocess.run(
            ["git", *arguments], cwd=self.client, env=self.environment,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=check,
            timeout=GIT_TIMEOUT_SECONDS,
        )

    def commit(self, message):
        self.git("commit", "--allow-empty", "-m", message)
        return self.git("rev-parse", "HEAD").stdout.decode().strip()

    def install_owner(self, *, advance=False):
        script = self.directory / "owner.py"
        command = [shutil.which("git"), "--git-dir", str(self.remote), "update-ref",
                   REFERENCE, self.second, self.first]
        script.write_text(
            "import os,pathlib,subprocess,sys\n"
            f"pathlib.Path({str(self.owner_input)!r}).write_bytes(sys.stdin.buffer.read())\n"
            + (f"subprocess.run({command!r}, check=True, "
               "env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')})\n"
               if advance else ""), encoding="utf-8",
        )
        hook = self.hooks / "pre-push"
        executable = shlex.quote(Path(sys.executable).as_posix())
        hook.write_text(f"#!/bin/sh\nexec {executable} {shlex.quote(script.as_posix())}\n",
                        encoding="utf-8")
        hook.chmod(0o700)

    def assert_advertised_first(self):
        self.assertEqual(self.owner_input.read_bytes().split(),
                         [b"(delete)", b"0" * len(self.first), REFERENCE.encode(),
                          self.first.encode()])

    def test_ordinary_deletion_runs_owner_then_removes_only_target(self):
        self.install_owner()
        self.git("push", "origin", "--delete", REFERENCE)
        self.assert_advertised_first()
        self.assertEqual(self.git("ls-remote", "origin", REFERENCE).stdout, b"")
        self.assertEqual(self.git("rev-parse", REFERENCE).stdout.decode().strip(), self.first)
        self.assertIn(self.second.encode(), self.git("ls-remote", "origin", "refs/heads/master").stdout)

    def test_remote_change_inside_pre_push_refuses_stale_deletion(self):
        self.install_owner(advance=True)
        result = self.git("push", "origin", "--delete", REFERENCE, check=False)
        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assert_advertised_first()
        self.assertEqual(self.git("ls-remote", "origin", REFERENCE).stdout.split(),
                         [self.second.encode(), REFERENCE.encode()])

    def test_remote_deletion_does_not_need_a_local_target_branch(self):
        self.git("branch", "-d", REFERENCE.removeprefix("refs/heads/"))
        self.install_owner()
        self.git("push", "origin", "--delete", REFERENCE)
        self.assert_advertised_first()
        self.assertEqual(self.git("ls-remote", "origin", REFERENCE).stdout, b"")
        self.assertNotEqual(self.git("show-ref", "--verify", REFERENCE, check=False).returncode, 0)

    def test_inventory_distinguishes_already_deleted_target(self):
        branch = REFERENCE.removeprefix("refs/heads/")
        self.assertEqual(remote_tip(self.client, str(self.remote), branch, self.first), self.first)
        self.git("push", "origin", "--delete", REFERENCE)
        self.assertIsNone(remote_tip(self.client, str(self.remote), branch, self.first))

    def test_inventory_does_not_treat_changed_tip_as_absence(self):
        self.git("--git-dir", str(self.remote), "update-ref", REFERENCE, self.second, self.first)
        with self.assertRaises(CleanupRefused):
            remote_tip(self.client, str(self.remote), REFERENCE.removeprefix("refs/heads/"), self.first)
        self.assertIn(self.second.encode(), self.git("ls-remote", "origin", REFERENCE).stdout)

    def test_transport_failure_is_not_reported_as_already_absent(self):
        with self.assertRaises(CleanupRefused):
            remote_tip(self.client, str(self.directory / "missing-remote"),
                       REFERENCE.removeprefix("refs/heads/"), self.first)


if __name__ == "__main__":
    unittest.main()
