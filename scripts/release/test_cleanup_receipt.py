"""Native receipt persistence and failure classification, without publication mocks."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from cleanup_receipt import CleanupReceipt
from cleanup_session import CleanupSession
from release_cleanup_audit import CleanupRefused, common_directory, git

CLI_TIMEOUT_SECONDS = 30


class CleanupReceiptTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        git(self.root, "init", "--initial-branch=master")
        git(self.root, "-c", "commit.gpgsign=false", "-c", "user.name=Fixture",
            "-c", "user.email=fixture@example.invalid", "commit", "--allow-empty", "-m", "seed")
        self.oid = git(self.root, "rev-parse", "HEAD").decode().strip()

    def events(self, path):
        return [json.loads(line) for line in path.read_text().splitlines()]

    def test_receipt_is_private_outside_worktree_and_survives_failure(self):
        with self.assertRaisesRegex(RuntimeError, "native failure"):
            with CleanupReceipt(self.root, {"expected": self.oid}) as receipt:
                receipt.record("remote", "deleted", "release/fixture")
                raise RuntimeError("native failure")
        self.assertTrue(receipt.path.is_relative_to(common_directory(self.root)))
        self.assertEqual(receipt.path.stat().st_mode & 0o777, 0o600)
        self.assertEqual([event["state"] for event in self.events(receipt.path)],
                         ["started", "deleted", "failed"])

    def test_lock_release_failure_is_not_recorded_as_completed(self):
        with self.assertRaises(CleanupRefused):
            with CleanupReceipt(self.root, {}) as receipt, CleanupSession(self.root) as session:
                (session.lock / "unknown").write_text("keep")
        self.assertEqual(self.events(receipt.path)[-1]["state"], "failed")
        self.assertTrue((session.lock / "unknown").exists())

    def test_cli_refuses_foreign_transport_before_installation_or_deletion(self):
        git(self.root, "branch", "release/fixture")
        git(self.root, "remote", "add", "origin", str(self.root / "not-a-public-remote"))
        cli = Path(__file__).with_name("run-release-cleanup.py")
        environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        result = subprocess.run(
            [sys.executable, "-B", str(cli), "--root", str(self.root), "--controller", str(self.root),
             "--repository", "HiroyukiFuruno/KatanA", "--version", "0.22.42", "--branch", "release/fixture",
             "--expected", self.oid, "--published", self.oid, "--remote"],
            env=environment, capture_output=True, timeout=CLI_TIMEOUT_SECONDS,
        )
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn(b"explicit GitHub repository", result.stderr)
        self.assertEqual(git(self.root, "rev-parse", "refs/heads/release/fixture").decode().strip(), self.oid)
        common = common_directory(self.root)
        self.assertFalse((common / "hooks/reference-transaction").exists())
        self.assertFalse((common / "katana-cleanup.lock").exists())
        receipts = list((common / "katana-cleanup-receipts").glob("*.jsonl"))
        self.assertEqual(len(receipts), 1)
        self.assertEqual(self.events(receipts[0])[-1]["state"], "failed")

    def test_foreign_controller_receives_no_receipt_or_lock(self):
        other = self.root / "other"
        other.mkdir()
        git(other, "init", "--initial-branch=master")
        cli = Path(__file__).with_name("run-release-cleanup.py")
        environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        result = subprocess.run(
            [sys.executable, "-B", str(cli), "--root", str(self.root), "--controller", str(other),
             "--repository", "HiroyukiFuruno/KatanA", "--version", "0.22.42", "--branch", "release/fixture",
             "--expected", self.oid, "--published", self.oid, "--remote"],
            env=environment, capture_output=True, timeout=CLI_TIMEOUT_SECONDS,
        )
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertFalse((common_directory(other) / "katana-cleanup-receipts").exists())
        self.assertFalse((common_directory(other) / "katana-cleanup.lock").exists())


if __name__ == "__main__":
    unittest.main()
