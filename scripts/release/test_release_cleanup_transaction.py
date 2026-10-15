"""Observe native Git reference transactions used by cleanup candidates."""

from pathlib import Path
import shlex
import sys
import tempfile
import unittest

from release_cleanup_audit import CleanupRefused, git
from release_cleanup_transaction import MAX_TRANSACTION_BYTES, verify_prepared_deletion


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

    def _install_hook(self, reject_prepared: bool = False, expected_tip: str | None = None) -> None:
        rejection = "exit 1" if reject_prepared else "exit 0"
        if expected_tip is not None:
            rejection = (
                "actual=$(git rev-parse --verify refs/heads/release/completed) || exit 1; "
                f"[ \"$actual\" = {shlex.quote(expected_tip)} ] || exit 1; exit 0"
            )
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

    def test_prepared_tip_check_preserves_a_moved_but_merged_branch(self) -> None:
        audited = git(self.root, "rev-parse", "HEAD").decode().strip()
        git(self.root, "commit", "--allow-empty", "-m", "next merged commit")
        moved = git(self.root, "rev-parse", "HEAD").decode().strip()
        git(self.root, "switch", "release/completed")
        git(self.root, "merge", "--ff-only", "master")
        git(self.root, "switch", "master")
        self._install_hook(expected_tip=audited)
        with self.assertRaises(CleanupRefused):
            git(self.root, "branch", "-d", "release/completed")
        self.assertNotEqual(audited, moved)
        self.assertEqual(
            git(self.root, "rev-parse", "refs/heads/release/completed").decode().strip(), moved
        )
        self.assertEqual(git(self.root, "rev-parse", "HEAD").decode().strip(), moved)
        self.assertNotIn("committed", [record[0] for record in self._records()])

    def test_prepared_tip_check_allows_the_exact_audited_branch(self) -> None:
        audited = git(self.root, "rev-parse", "HEAD").decode().strip()
        self._install_hook(expected_tip=audited)
        git(self.root, "branch", "-d", "release/completed")
        self.assertEqual(
            git(self.root, "for-each-ref", "--format=%(refname)", "refs/heads/release/completed"), b""
        )
        self.assertIn("committed", [record[0] for record in self._records()])

    def _install_runtime_guard(self, expected: str) -> None:
        runtime = Path(__file__).resolve().with_name("release_cleanup_transaction.py")
        command = [sys.executable, str(runtime), "--root", str(self.root),
                   "--branch", "release/completed", "--expected", expected]
        hook = self.hooks / "reference-transaction"
        hook.write_text("#!/bin/sh\nexec " + shlex.join(command) + " \"$@\"\n")
        hook.chmod(0o755)

    def test_runtime_guard_allows_exact_native_deletion(self) -> None:
        expected = git(self.root, "rev-parse", "HEAD").decode().strip()
        self._install_runtime_guard(expected)
        git(self.root, "branch", "-d", "release/completed")
        self.assertEqual(
            git(self.root, "for-each-ref", "--format=%(refname)", "refs/heads/release/completed"), b""
        )
        self.assertEqual(git(self.root, "rev-parse", "HEAD").decode().strip(), expected)

    def test_runtime_guard_preserves_branch_that_moved_to_another_merged_commit(self) -> None:
        expected = git(self.root, "rev-parse", "HEAD").decode().strip()
        git(self.root, "commit", "--allow-empty", "-m", "new controller commit")
        newer = git(self.root, "rev-parse", "HEAD").decode().strip()
        git(self.root, "switch", "release/completed")
        git(self.root, "merge", "--ff-only", "master")
        git(self.root, "switch", "master")
        self._install_runtime_guard(expected)
        with self.assertRaises(CleanupRefused):
            git(self.root, "branch", "-d", "release/completed")
        self.assertEqual(
            git(self.root, "rev-parse", "refs/heads/release/completed").decode().strip(), newer
        )
        self.assertEqual(git(self.root, "rev-parse", "HEAD").decode().strip(), newer)

    def test_runtime_guard_rejects_foreign_or_malformed_transaction_input(self) -> None:
        expected = git(self.root, "rev-parse", "HEAD").decode().strip()
        valid = f"{ZERO_OID} {ZERO_OID} refs/heads/release/completed\n".encode()
        invalid = (
            b"", b"\xff", valid + valid, b"x" * (MAX_TRANSACTION_BYTES + 1),
            valid.replace(b"release/completed", b"master"),
            f"{ZERO_OID} {expected} refs/heads/release/completed\n".encode(),
            valid.replace(ZERO_OID.encode(), b"0" * 64),
        )
        for payload in invalid:
            with self.subTest(size=len(payload), prefix=payload[:16]):
                with self.assertRaises(CleanupRefused):
                    verify_prepared_deletion(self.root, "release/completed", expected, payload)
        self.assertEqual(
            git(self.root, "rev-parse", "refs/heads/release/completed").decode().strip(), expected
        )

    def test_runtime_guard_refuses_non_release_or_abbreviated_identity(self) -> None:
        expected = git(self.root, "rev-parse", "HEAD").decode().strip()
        valid = f"{ZERO_OID} {ZERO_OID} refs/heads/release/completed\n".encode()
        for branch, oid in (("master", expected), ("release/completed", expected[:-1])):
            with self.subTest(branch=branch, oid=oid):
                with self.assertRaises(CleanupRefused):
                    verify_prepared_deletion(self.root, branch, oid, valid)

    def test_runtime_guard_refuses_symbolic_branch_even_at_expected_commit(self) -> None:
        expected = git(self.root, "rev-parse", "HEAD").decode().strip()
        git(self.root, "symbolic-ref", "refs/heads/release/completed", "refs/heads/master")
        valid = f"{ZERO_OID} {ZERO_OID} refs/heads/release/completed\n".encode()
        with self.assertRaises(CleanupRefused):
            verify_prepared_deletion(self.root, "release/completed", expected, valid)
        self.assertEqual(
            git(self.root, "symbolic-ref", "refs/heads/release/completed").strip(), b"refs/heads/master"
        )


if __name__ == "__main__":
    unittest.main()
