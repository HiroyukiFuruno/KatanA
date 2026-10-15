"""Native installer interruption must not bypass the retained push owner."""

import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
INSTALLER = HERE / "install-pre-push-dispatcher.py"
GIT_TIMEOUT_SECONDS = 30
OWNER_MODE = 0o751


class PrePushInstallerInterruptTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)
        self.root = self.base / "client"
        self.remote = self.base / "remote.git"
        self.root.mkdir()
        self.git("init", "--initial-branch=master")
        self.git("init", "--bare", str(self.remote))
        for key, value in (("user.name", "Fixture"), ("user.email", "fixture@example.invalid"),
                           ("commit.gpgsign", "false")):
            self.git("config", key, value)
        self.git("commit", "--allow-empty", "-m", "fixture")
        self.git("remote", "add", "origin", str(self.remote))
        self._runtime()
        self.hook = self.root / ".git/hooks/pre-push"
        self.backup = self.hook.with_name("pre-push.katana-existing")
        self.owner_log = self.base / "owner-log"
        self.original = ("#!/bin/sh\ncat > " + shlex.quote(str(self.owner_log)) + "\nexit 23\n").encode()
        self.hook.write_bytes(self.original)
        self.hook.chmod(OWNER_MODE)

    def _runtime(self) -> None:
        runtime = self.root / "scripts/release"
        runtime.mkdir(parents=True)
        for source in HERE.glob("*.py"):
            if not source.name.startswith("test"):
                shutil.copy2(source, runtime / source.name)

    @staticmethod
    def clean_environment() -> dict[str, str]:
        return {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}

    def git(self, *arguments: str, check: bool = True) -> subprocess.CompletedProcess[bytes]:
        return subprocess.run(["git", *arguments], cwd=self.root, env=self.clean_environment(),
                              capture_output=True, check=check, timeout=GIT_TIMEOUT_SECONDS)

    def _interrupt_installation(self) -> subprocess.CompletedProcess[bytes]:
        program = (
            "import os,runpy,signal,sys\n"
            f"sys.path.insert(0, {str(HERE)!r})\n"
            "def interrupt(frame,event,argument):\n"
            "    if event == 'c_return' and argument is os.replace:\n"
            "        sys.setprofile(None)\n"
            "        os.kill(os.getpid(), signal.SIGINT)\n"
            "sys.setprofile(interrupt)\n"
            f"sys.argv = [{str(INSTALLER)!r}, '--repository', 'ACME/KatanA']\n"
            f"runpy.run_path({str(INSTALLER)!r}, run_name='__main__')\n"
        )
        return subprocess.run([sys.executable, "-B", "-c", program], cwd=self.root,
                              env=self.clean_environment(), capture_output=True, check=False,
                              timeout=GIT_TIMEOUT_SECONDS)

    def test_interrupt_after_native_replace_keeps_owner_and_rejects_normal_push(self) -> None:
        result = self._interrupt_installation()
        self.assertNotEqual(result.returncode, 0)
        self.assertTrue(self.hook.is_file(), result.stderr)
        self.assertEqual(self.backup.read_bytes(), self.original)
        self.assertEqual(self.backup.stat().st_mode & 0o777, OWNER_MODE)
        push = self.git("push", "origin", "HEAD:refs/heads/release/fixture", check=False)
        self.assertNotEqual(push.returncode, 0, push.stderr)
        self.assertTrue(self.owner_log.exists(), push.stderr)
        self.assertIn(b"refs/heads/release/fixture", self.owner_log.read_bytes())
        self.assertEqual(self.git("ls-remote", "origin", "refs/heads/release/fixture").stdout, b"")


if __name__ == "__main__":
    unittest.main()
