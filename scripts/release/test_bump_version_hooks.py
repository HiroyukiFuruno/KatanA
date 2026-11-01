"""Exercise the real local bump script and a real Git pre-commit hook."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("bump-version.sh").resolve()


def command(root, *args):
    return subprocess.run(args, cwd=root, check=True, capture_output=True, text=True, timeout=60)


class BumpHooksTest(unittest.TestCase):
    def test_missing_or_invalid_issue_fails_before_modifying_files(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cargo = root / "Cargo.toml"
            original = '[package]\nname="example"\nversion = "0.1.0"\n'
            cargo.write_text(original)
            for issue in ("", "0", "1;echo invalid", "01"):
                env = os.environ | {"GITHUB_ACTIONS": "false", "KATANA_RELEASE_ISSUE": issue}
                result = subprocess.run(["bash", str(SCRIPT), "0.1.1"], cwd=root, env=env,
                                        capture_output=True, text=True, timeout=60)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("KATANA_RELEASE_ISSUE", result.stderr)
                self.assertEqual(cargo.read_text(), original)

    def test_failed_cargo_resolution_stops_before_commit(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "Cargo.toml").write_text('[package]\nname="broken"\nversion = "0.1.0"\n')
            command(root, "git", "init", "-q")
            command(root, "git", "config", "user.name", "Test")
            command(root, "git", "config", "user.email", "test@example.invalid")
            command(root, "git", "add", ".")
            command(root, "git", "-c", "commit.gpgsign=false", "commit", "-qm", "base")
            env = os.environ | {"GITHUB_ACTIONS": "false", "KATANA_RELEASE_ISSUE": "403"}
            result = subprocess.run(["bash", str(SCRIPT), "0.1.1"], cwd=root, env=env,
                                    capture_output=True, text=True, timeout=60)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("error:", result.stderr)
            self.assertEqual(command(root, "git", "log", "-1", "--format=%s").stdout.strip(), "base")

    def test_same_version_does_not_run_dependency_update(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "Cargo.toml").write_text('[package]\nname="example"\nversion = "0.1.0"\n')
            (root / "Cargo.lock").write_text('version = 4\n')
            plist = root / "crates/katana-ui/Info.plist"
            plist.parent.mkdir(parents=True)
            plist.write_text('<key>CFBundleShortVersionString</key><string>v0.1.0</string>\n')
            command(root, "git", "init", "-q")
            command(root, "git", "config", "user.name", "Test")
            command(root, "git", "config", "user.email", "test@example.invalid")
            command(root, "git", "add", ".")
            command(root, "git", "-c", "commit.gpgsign=false", "commit", "-qm", "base")
            env = os.environ | {"GITHUB_ACTIONS": "false", "KATANA_RELEASE_ISSUE": "403"}
            result = subprocess.run(["bash", "-x", str(SCRIPT), "0.1.0"], cwd=root, env=env,
                                    capture_output=True, text=True, timeout=60)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertNotIn("+ cargo update", result.stderr)

    def test_local_bump_respects_precommit_rejection(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            (root / "src/lib.rs").write_text("pub fn example() {}\n")
            (root / "Cargo.toml").write_text('[package]\nname="example"\nversion = "0.1.0"\nedition="2024"\n')
            plist = root / "crates/katana-ui/Info.plist"
            plist.parent.mkdir(parents=True)
            plist.write_text('<key>CFBundleShortVersionString</key><string>v0.1.0</string>\n')
            command(root, "cargo", "generate-lockfile", "--offline")
            command(root, "git", "init", "-q")
            command(root, "git", "config", "user.name", "Test")
            command(root, "git", "config", "user.email", "test@example.invalid")
            command(root, "git", "add", ".")
            command(root, "git", "-c", "commit.gpgsign=false", "commit", "-qm", "base")
            marker = root / "hook-called"
            hook = root / ".git/hooks/pre-commit"
            hook.write_text(f'#!/bin/sh\nprintf invoked > "{marker}"\nexit 1\n')
            hook.chmod(0o755)
            env = os.environ | {"GITHUB_ACTIONS": "false", "KATANA_RELEASE_ISSUE": "403"}
            result = subprocess.run(["bash", str(SCRIPT), "0.1.1"], cwd=root, env=env,
                                    capture_output=True, text=True, timeout=60)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(marker.read_text(), "invoked")
            self.assertEqual(command(root, "git", "log", "-1", "--format=%s").stdout.strip(), "base")

    def test_local_bump_passes_issue_to_commit_message_hook(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            (root / "src/lib.rs").write_text("pub fn example() {}\n")
            (root / "Cargo.toml").write_text('[package]\nname="example"\nversion = "0.1.0"\nedition="2024"\n')
            plist = root / "crates/katana-ui/Info.plist"
            plist.parent.mkdir(parents=True)
            plist.write_text('<key>CFBundleShortVersionString</key><string>v0.1.0</string>\n')
            command(root, "cargo", "generate-lockfile", "--offline")
            command(root, "git", "init", "-q")
            command(root, "git", "config", "user.name", "Test")
            command(root, "git", "config", "user.email", "test@example.invalid")
            command(root, "git", "add", ".")
            command(root, "git", "-c", "commit.gpgsign=false", "commit", "-qm", "base")
            marker = root / "message"
            hook = root / ".git/hooks/commit-msg"
            hook.write_text(f'#!/bin/sh\ncat "$1" > "{marker}"\nexit 1\n')
            hook.chmod(0o755)
            env = os.environ | {"GITHUB_ACTIONS": "false", "KATANA_RELEASE_ISSUE": "403"}
            result = subprocess.run(["bash", str(SCRIPT), "0.1.1"], cwd=root, env=env,
                                    capture_output=True, text=True, timeout=60)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(marker.read_text().strip(), "chore: v0.1.1のリリース準備 (#403)")
            self.assertEqual(command(root, "git", "log", "-1", "--format=%s").stdout.strip(), "base")


if __name__ == "__main__":
    unittest.main()
