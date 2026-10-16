"""Real Git hooks must validate the repository whose transaction is locked."""

from pathlib import Path
import shlex
import sys
import tempfile
import unittest

from release_cleanup_audit import CleanupRefused, git


class CleanupTransactionRepositoryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)
        self.source = self._repository("source")
        self.other = self._repository("other")

    def _repository(self, name: str) -> Path:
        root = self.base / name
        root.mkdir()
        git(root, "init", "--initial-branch=master", "--object-format=sha1")
        git(root, "config", "user.email", "cleanup@example.invalid")
        git(root, "config", "user.name", "Cleanup binding fixture")
        git(root, "config", "commit.gpgsign", "false")
        git(root, "commit", "--allow-empty", "-m", name)
        git(root, "branch", "release/completed")
        return root

    def _install(self, actual: Path, claimed: Path, expected: str) -> None:
        hooks = self.base / "hooks"
        hooks.mkdir()
        runtime = Path(__file__).resolve().with_name("release_cleanup_transaction.py")
        command = [sys.executable, str(runtime), "--root", str(claimed),
                   "--branch", "release/completed", "--expected", expected]
        hook = hooks / "reference-transaction"
        hook.write_text("#!/bin/sh\nexec " + shlex.join(command) + " \"$@\"\n")
        hook.chmod(0o755)
        git(actual, "config", "core.hooksPath", str(hooks))

    def test_foreign_root_cannot_admit_the_actual_repository_deletion(self) -> None:
        claimed = git(self.source, "rev-parse", "HEAD").decode().strip()
        actual = git(self.other, "rev-parse", "HEAD").decode().strip()
        self.assertNotEqual(claimed, actual)
        self._install(self.other, self.source, claimed)
        with self.assertRaises(CleanupRefused):
            git(self.other, "branch", "-d", "release/completed")
        self.assertEqual(
            git(self.other, "rev-parse", "refs/heads/release/completed").decode().strip(), actual
        )
        self.assertEqual(git(self.other, "rev-parse", "HEAD").decode().strip(), actual)
        self.assertEqual(
            git(self.source, "rev-parse", "refs/heads/release/completed").decode().strip(), claimed
        )


if __name__ == "__main__":
    unittest.main()
