"""Real Git/Just process tests for receipt binding, not production quality results."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from owner_verification import RECEIPT_NAME, VerificationSession, source_revision

RUNNER = Path(__file__).resolve().parent / "run-owned-push-check.py"


class OwnerVerificationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name).resolve()
        self.environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        subprocess.run(["git", "init", "-q", str(self.root)], check=True, env=self.environment)

    def tearDown(self):
        self.temporary.cleanup()

    def fixture(self, recipe: str):
        (self.root / "Justfile").write_text("check:\n    " + recipe + "\n")
        (self.root / "tracked.txt").write_text("unchanged\n")
        subprocess.run(["git", "add", "Justfile", "tracked.txt"], cwd=self.root, check=True, env=self.environment)
        subprocess.run(["git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-qm", "fixture"],
                       cwd=self.root, check=True, env=self.environment)
        return subprocess.run(["git", "rev-parse", "HEAD"], cwd=self.root, check=True,
                              capture_output=True, text=True, env=self.environment).stdout.strip()

    def run_owner(self, session):
        environment = session.environment()
        environment = {key: value for key, value in environment.items() if not key.startswith("GIT_")}
        return subprocess.run([sys.executable, str(RUNNER)], cwd=self.root, env=environment,
                              check=False, capture_output=True, text=True)

    def test_fixed_owner_command_issues_one_matching_receipt(self):
        revision = self.fixture("printf 'fixed check executed\\n'")
        with VerificationSession(self.root, revision) as session:
            result = self.run_owner(session)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("fixed check executed", result.stdout)
            self.assertEqual(session.verified_source(), revision)
            second = self.run_owner(session)
            self.assertEqual(second.returncode, 2)

    def test_skipped_and_failed_command_never_provide_a_receipt(self):
        revision = self.fixture("exit 7")
        with VerificationSession(self.root, revision) as session:
            self.assertIsNone(session.verified_source())
            result = self.run_owner(session)
            self.assertNotEqual(result.returncode, 0)
            self.assertIsNone(session.verified_source())

    def test_source_changes_during_owner_check_are_rejected(self):
        revision = self.fixture("printf changed >> tracked.txt")
        with VerificationSession(self.root, revision) as session:
            result = self.run_owner(session)
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertIn("clean tracked source", result.stderr)
            self.assertIsNone(session.verified_source())

    def test_another_session_receipt_cannot_be_reused(self):
        revision = self.fixture("printf checked")
        with VerificationSession(self.root, revision) as first, VerificationSession(self.root, revision) as second:
            self.assertEqual(self.run_owner(first).returncode, 0)
            (second.directory / RECEIPT_NAME).write_bytes((first.directory / RECEIPT_NAME).read_bytes())
            with self.assertRaisesRegex(ValueError, "does not match"):
                second.verified_source()

    def test_author_exit_zero_without_owner_token_is_rejected(self):
        revision = self.fixture("printf checked")
        with VerificationSession(self.root, revision) as session:
            (session.directory / RECEIPT_NAME).write_text(json.dumps({"source": revision, "exitCode": 0}))
            with self.assertRaisesRegex(ValueError, "does not match"):
                session.verified_source()


if __name__ == "__main__":
    unittest.main()
