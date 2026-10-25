from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
INSTALLER = HERE / "install-pre-push-dispatcher.py"
DISPATCHER = HERE / "pre_push_dispatcher.py"


class InstallerTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        subprocess.run(["git", "init", "-q", str(self.root)], check=True, env=self.clean_env())
        subprocess.run(["git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "--allow-empty", "-m", "init"], cwd=self.root, check=True, env=self.clean_env(), capture_output=True)
        self.hooks = self.root / ".git/hooks"
        self.hook = self.hooks / "pre-push"
        self.backup = self.hooks / "pre-push.katana-existing"
        source = self.root / "scripts/release"
        source.mkdir(parents=True)
        for path in HERE.glob("*.py"):
            if not path.name.startswith("test"):
                shutil.copy2(path, source / path.name)
        ci = self.root / "scripts/ci"
        ci.mkdir(parents=True)
        shutil.copy2(HERE.parent / "ci/cargo_target_lease.py", ci / "cargo_target_lease.py")

    def tearDown(self) -> None:
        self.temp.cleanup()

    @staticmethod
    def clean_env() -> dict[str, str]:
        return {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}

    def install(self) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(INSTALLER), "--repository", "ACME/KatanA"],
            cwd=self.root,
            text=True,
            capture_output=True,
            env=self.clean_env(),
        )

    def test_preserves_existing_bytes_and_mode_and_is_idempotent(self) -> None:
        original = b"#!/bin/sh\nprintf existing\n"
        self.hook.write_bytes(original)
        os.chmod(self.hook, 0o751)
        config_before = subprocess.run(
            ["git", "config", "--local", "--list"], cwd=self.root, env=self.clean_env(), capture_output=True, check=True
        ).stdout
        result = self.install()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.backup.read_bytes(), original)
        self.assertEqual(self.backup.stat().st_mode & 0o777, 0o751)
        first = self.hook.read_bytes()
        self.assertEqual(self.install().returncode, 0)
        self.assertEqual(self.hook.read_bytes(), first)

        config_after = subprocess.run(
            ["git", "config", "--local", "--list"], cwd=self.root, env=self.clean_env(), capture_output=True, check=True
        ).stdout
        self.assertEqual(config_after, config_before)

    def test_rewraps_when_lefthook_regenerates_the_same_owner(self) -> None:
        original = b"#!/bin/sh\nprintf existing\n"
        self.hook.write_bytes(original)
        os.chmod(self.hook, 0o751)
        self.assertEqual(self.install().returncode, 0)
        self.hook.write_bytes(original)
        os.chmod(self.hook, 0o751)
        result = self.install()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.backup.read_bytes(), original)
        self.assertIn(b"KATANA_PRE_PUSH_DISPATCHER_V1", self.hook.read_bytes())

    def test_missing_required_backup_is_rejected(self) -> None:
        self.hook.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
        self.hook.chmod(0o755)
        self.assertEqual(self.install().returncode, 0)
        self.backup.unlink()
        result = self.install()
        self.assertEqual(result.returncode, 2)

    def test_unknown_wrapper_is_rejected(self) -> None:
        self.hook.write_text("#!/bin/sh\n# KATANA_PRE_PUSH_DISPATCHER_V1\n", encoding="utf-8")
        self.hook.chmod(0o755)
        result = self.install()
        self.assertEqual(result.returncode, 2)

    def test_custom_hooks_path_outside_common_dir_is_rejected(self) -> None:
        outside = self.root / "outside-hooks"
        subprocess.run(
            ["git", "config", "core.hooksPath", str(outside)],
            cwd=self.root,
            env=self.clean_env(),
            check=True,
        )
        result = self.install()
        self.assertEqual(result.returncode, 2)
        self.assertFalse((outside / "pre-push").exists())

    def test_symlink_hook_directory_never_mutates_foreign_hook(self) -> None:
        original = b"#!/bin/sh\nexit 0\n"
        self.hook.write_bytes(original)
        self.hook.chmod(0o751)
        config = self.root / ".git/config"
        original_config = config.read_bytes()
        foreign = self.root / "foreign-hooks"
        self.hooks.rename(foreign)
        self.hooks.symlink_to(foreign, target_is_directory=True)
        result = self.install()
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertEqual((foreign / "pre-push").read_bytes(), original)
        self.assertEqual((foreign / "pre-push").stat().st_mode & 0o777, 0o751)
        self.assertEqual(config.read_bytes(), original_config)
        self.assertFalse((foreign / "pre-push.katana-existing").exists())

    def test_unknown_backup_collision_is_refused(self) -> None:
        self.backup.write_text("unrelated", encoding="utf-8")
        result = self.install()
        self.assertEqual(result.returncode, 2)
        self.assertFalse(self.hook.exists())

    def test_missing_existing_hook_is_refused(self) -> None:
        result = self.install()
        self.assertEqual(result.returncode, 2)
        self.assertFalse(self.hook.exists())

    def test_incompatible_checkout_refuses_before_mutating_hook(self) -> None:
        original = b"#!/bin/sh\nexit 0\n"
        self.hook.write_bytes(original)
        self.hook.chmod(0o755)
        (self.root / "scripts/release/pre_push_dispatcher.py").unlink()
        result = self.install()
        self.assertEqual(result.returncode, 2)
        self.assertIn("shared hook runtime is incompatible", result.stderr)
        self.assertEqual(self.hook.read_bytes(), original)
        self.assertFalse(self.backup.exists())

    def test_symlink_target_is_refused(self) -> None:
        target = self.root / "outside"
        target.write_text("hook", encoding="utf-8")
        self.hook.symlink_to(target)
        result = self.install()
        self.assertEqual(result.returncode, 2)
        self.assertTrue(self.hook.is_symlink())

    def prepare_installed_fixture(self) -> Path:
        source = self.root / "scripts/release"
        source.mkdir(parents=True, exist_ok=True)
        shutil.copy2(DISPATCHER, source / "pre_push_dispatcher.py")
        log = self.root / "events"
        self.hook.write_text(
            "#!/bin/sh\n"
            f"printf existing >> {str(log)!r}\n",
            encoding="utf-8",
        )
        self.hook.chmod(0o755)
        return log

    def install_fixture_checker(self, log: Path) -> None:
        checker = self.root / "scripts/release/check-push-dependency-evidence.py"
        checker.write_text(
            "#!/usr/bin/env python3\nimport pathlib\n"
            f"def main(arguments=None, *, verified_source=None, payload=None): pathlib.Path({str(log)!r}).open('a').write('checker:' + repr(arguments) + ':' + repr(verified_source) + ':' + payload.decode())\n",
            encoding="utf-8",
        )
        checker.chmod(0o755)

    def run_installed_wrapper(self) -> subprocess.CompletedProcess[bytes]:
        return subprocess.run(
            [str(self.hook), "origin", "url", "local", "remote"],
            cwd=self.root,
            input=b"update\n",
            capture_output=True,
            env=self.clean_env(),
        )

    def test_installed_wrapper_runs_retained_hook_then_checker(self) -> None:
        log = self.prepare_installed_fixture()
        self.assertEqual(self.install().returncode, 0)
        self.install_fixture_checker(log)
        result = self.run_installed_wrapper()
        self.assertEqual(result.returncode, 0, result.stderr)
        data = log.read_text()
        self.assertTrue(data.startswith("existing"))
        self.assertIn("checker:", data)
        self.assertIn("update\n", data)


if __name__ == "__main__":
    unittest.main()
