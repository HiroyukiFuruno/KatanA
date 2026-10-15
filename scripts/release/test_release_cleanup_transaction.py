"""Observe native Git reference transactions used by cleanup candidates."""

from pathlib import Path
import shlex
import tempfile
import unittest

from release_cleanup_audit import CleanupRefused, git


ZERO_OID = "0" * 40


class CleanupReferenceTransactionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name) / "source"
        self.hooks = Path(self.temporary.name) / "hooks"
        self.log = Path(self.temporary.name) / "transactions.log"
        self.root.mkdir()
        self.hooks.mkdir()
        git(self.root, "init", "--initial-branch=master", "--object-format=sha1")
        git(self.root, "config", "user.email", "cleanup@example.invalid")
        git(self.root, "config", "user.name", "Cleanup fixture")
        git(self.root, "config", "commit.gpgsign", "false")
        git(self.root, "commit", "--allow-empty", "-m", "initial")
        git(self.root, "branch", "release/completed")
        # WHY: 実repoの共通hookへ副作用を漏らさず、一時repoだけで通知を観測する。
        git(self.root, "config", "core.hooksPath", str(self.hooks))

    def _install_hook(self, reject_prepared: bool = False) -> None:
        rejection = "exit 1" if reject_prepared else "exit 0"
        hook = self.hooks / "reference-transaction"
        hook.write_text(
            "#!/bin/sh\n"
            f"while IFS= read -r line; do printf '%s\\t%s\\n' \"$1\" \"$line\" >> {shlex.quote(str(self.log))}; done\n"
            "if [ \"$1\" = prepared ]; then " + rejection + "; fi\n"
            "exit 0\n"
        )
        hook.chmod(0o755)

    def _records(self) -> list[tuple[str, str, str, str]]:
        records = []
        for line in self.log.read_text().splitlines():
            phase, payload = line.split("\t", 1)
            old, new, ref = payload.split()
            records.append((phase, old, new, ref))
        return records

    def test_branch_delete_reports_transaction_and_commits(self) -> None:
        self._install_hook()
        tip = git(self.root, "rev-parse", "refs/heads/release/completed").decode().strip()
        git(self.root, "branch", "-d", "release/completed")
        records = self._records()
        self.assertGreaterEqual(len(records), 1)
        prepared = [record for record in records if record[0] == "prepared"]
        committed = [record for record in records if record[0] == "committed"]
        self.assertTrue(prepared)
        self.assertTrue(committed)
        for record in prepared + committed:
            self.assertIn(record[1], {ZERO_OID, tip})
            self.assertEqual(record[2:], (ZERO_OID, "refs/heads/release/completed"))
        self.assertNotEqual(tip, ZERO_OID)
        self.assertEqual(
            git(self.root, "for-each-ref", "--format=%(refname)", "refs/heads/release/completed"),
            b"",
        )

    def test_prepared_rejection_preserves_ref_and_head(self) -> None:
        self._install_hook(reject_prepared=True)
        tip = git(self.root, "rev-parse", "HEAD").decode().strip()
        with self.assertRaises(CleanupRefused):
            git(self.root, "branch", "-d", "release/completed")
        self.assertEqual(git(self.root, "rev-parse", "HEAD").decode().strip(), tip)
        self.assertEqual(
            git(self.root, "rev-parse", "refs/heads/release/completed").decode().strip(), tip
        )
        records = self._records()
        self.assertTrue(records)
        prepared = [record for record in records if record[0] == "prepared"]
        self.assertTrue(prepared)
        self.assertIn(prepared[0][1], {ZERO_OID, tip})
        self.assertEqual(prepared[0][2:], (ZERO_OID, "refs/heads/release/completed"))
        self.assertNotIn("committed", [record[0] for record in records])


if __name__ == "__main__":
    unittest.main()
