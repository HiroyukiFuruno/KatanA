"""Filesystem and native-Git contracts for the reference hook installer."""

import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import tempfile
import unittest


HERE = Path(__file__).resolve().parent
INSTALLER = HERE / "install-reference-dispatcher.py"
MARKER = "KATANA_REFERENCE_DISPATCHER_V1"


class ReferenceHookInstallerTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.source = self.root / "scripts/release"
        self.source.mkdir(parents=True)
        for path in HERE.glob("*.py"):
            if not path.name.startswith("test"):
                shutil.copy2(path, self.source / path.name)
        subprocess.run(["git", "init", "-q", str(self.root)], check=True, env=self.clean_env())
        self.hooks = self.root / ".git/hooks"
        self.hook = self.hooks / "reference-transaction"
        self.backup = self.hooks / "reference-transaction.katana-existing"
        subprocess.run(["git", "config", "user.name", "Fixture"], cwd=self.root, check=True,
                       env=self.clean_env())
        subprocess.run(["git", "config", "user.email", "fixture@example.invalid"], cwd=self.root, check=True,
                       env=self.clean_env())
        subprocess.run(["git", "config", "commit.gpgsign", "false"], cwd=self.root, check=True,
                       env=self.clean_env())
        subprocess.run(["git", "commit", "--allow-empty", "-m", "init"], cwd=self.root, check=True,
                       env=self.clean_env(), capture_output=True)

    @staticmethod
    def clean_env() -> dict[str, str]:
        return {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}

    def install(self) -> subprocess.CompletedProcess[str]:
        return subprocess.run([sys.executable, str(INSTALLER)], cwd=self.root, text=True,
                              capture_output=True, env=self.clean_env())

    def _owner(self, exit_code: int = 0) -> tuple[bytes, Path]:
        log = self.root / "owner-input"
        content = (
            "#!/bin/sh\n"
            f"printf '%s\\n' \"$1\" > {shlex.quote(str(self.root / 'owner-phase'))}\n"
            f"cat > {shlex.quote(str(log))}\n"
            f"exit {exit_code}\n"
        ).encode()
        self.hook.write_bytes(content)
        os.chmod(self.hook, 0o751)
        return content, log

    def test_existing_owner_bytes_mode_and_install_are_idempotent(self) -> None:
        original, _ = self._owner()
        result = self.install()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.backup.read_bytes(), original)
        self.assertEqual(self.backup.stat().st_mode & 0o777, 0o751)
        wrapper = self.hook.read_bytes()
        self.assertIn(MARKER.encode(), wrapper)
        self.assertEqual(self.install().returncode, 0)
        self.assertEqual(self.hook.read_bytes(), wrapper)

    def test_existing_owner_can_be_absent_and_native_commit_passes(self) -> None:
        result = self.install()
        self.assertEqual(result.returncode, 0, result.stderr)
        commit = subprocess.run(["git", "commit", "--allow-empty", "-m", "hooked"], cwd=self.root,
                                capture_output=True, env=self.clean_env(), check=False)
        self.assertEqual(commit.returncode, 0, commit.stderr)

    def test_interrupt_after_native_replace_retains_existing_owner(self) -> None:
        original, _ = self._owner()
        program = (
            "import os,runpy,signal,sys\n"
            f"sys.path.insert(0, {str(HERE)!r})\n"
            "def interrupt(frame,event,argument):\n"
            "    if event == 'c_return' and argument is os.replace:\n"
            "        sys.setprofile(None)\n"
            "        os.kill(os.getpid(), signal.SIGINT)\n"
            "sys.setprofile(interrupt)\n"
            f"sys.argv = [{str(INSTALLER)!r}]\n"
            f"runpy.run_path({str(INSTALLER)!r}, run_name='__main__')\n"
        )
        result = subprocess.run([sys.executable, "-B", "-c", program], cwd=self.root,
                                env=self.clean_env(), capture_output=True, check=False)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(MARKER.encode(), self.hook.read_bytes())
        self.assertTrue(self.backup.exists(), result.stderr)
        self.assertEqual(self.backup.read_bytes(), original)
        self.assertEqual(self.backup.stat().st_mode & 0o777, 0o751)
        commit = subprocess.run(["git", "commit", "--allow-empty", "-m", "after-interrupt"],
                                cwd=self.root, env=self.clean_env(), capture_output=True, check=False)
        self.assertEqual(commit.returncode, 0, commit.stderr)

    def test_installed_owner_preserves_phase_stdin_and_exit(self) -> None:
        _, log = self._owner(23)
        self.assertEqual(self.install().returncode, 0)
        payload = b"old new refs/heads/release/completed\n"
        result = subprocess.run([str(self.hook), "prepared"], cwd=self.root, input=payload,
                                capture_output=True, env=self.clean_env(), check=False)
        self.assertEqual(result.returncode, 23)
        self.assertEqual((self.root / "owner-phase").read_text(), "prepared\n")
        self.assertEqual(log.read_bytes(), payload)

    def test_backup_collision_unknown_marker_and_lockdir_are_rejected(self) -> None:
        self.backup.write_text("unrelated", encoding="utf-8")
        result = self.install()
        self.assertEqual(result.returncode, 2)
        self.assertFalse(self.hook.exists())
        self.backup.unlink()
        self.hook.write_text("#!/bin/sh\n# KATANA_REFERENCE_DISPATCHER_V0\n", encoding="utf-8")
        self.hook.chmod(0o700)
        before = self.hook.read_bytes()
        self.assertEqual(self.install().returncode, 2)
        self.assertEqual(self.hook.read_bytes(), before)
        self.hook.unlink()
        (self.hooks / ".katana-reference-install.lock").mkdir()
        self.assertEqual(self.install().returncode, 2)

    def test_symlink_hook_and_symlink_hooks_directory_are_rejected(self) -> None:
        target = self.root / "foreign-hook"
        target.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
        self.hook.symlink_to(target)
        self.assertEqual(self.install().returncode, 2)
        self.assertTrue(self.hook.is_symlink())
        self.hook.unlink()
        foreign = self.root / "foreign-hooks"
        self.hooks.rename(foreign)
        self.hooks.symlink_to(foreign, target_is_directory=True)
        before = foreign / "reference-transaction"
        before.write_text("owner", encoding="utf-8")
        self.assertEqual(self.install().returncode, 2)
        self.assertEqual(before.read_text(), "owner")

    def test_custom_hooks_path_outside_common_git_directory_is_rejected(self) -> None:
        outside = self.root / "outside-hooks"
        outside.mkdir()
        subprocess.run(["git", "config", "core.hooksPath", str(outside)], cwd=self.root,
                       check=True, env=self.clean_env())
        result = self.install()
        self.assertEqual(result.returncode, 2)
        self.assertFalse((outside / "reference-transaction").exists())

    def test_missing_runtime_is_rejected_before_owner_mutation(self) -> None:
        original, _ = self._owner()
        runtime = next(path for path in self.source.glob("*.py") if path.name != "cleanup_reference_dispatcher.py")
        runtime.unlink()
        result = self.install()
        self.assertEqual(result.returncode, 2)
        self.assertEqual(self.hook.read_bytes(), original)
        self.assertFalse(self.backup.exists())


if __name__ == "__main__":
    unittest.main()
