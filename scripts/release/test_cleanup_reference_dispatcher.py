"""Real Git contracts for private cleanup intent and owner-first dispatch."""

import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys
import tempfile
import unittest

from release_cleanup_audit import CleanupRefused, git
from cleanup_intent import read_intent, write_intent
from cleanup_intent_io import MAX_INTENT_BYTES


DISPATCHER = Path(__file__).with_name("cleanup_reference_dispatcher.py")


class CleanupReferenceDispatcherTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        base = Path(self.temporary.name)
        self.root = base / "source"
        self.hooks = base / "hooks"
        self.private = base / "private"
        self.existing = base / "existing-hook"
        self.log = base / "existing.log"
        self.root.mkdir()
        self.hooks.mkdir()
        self.private.mkdir(mode=0o700)
        git(self.root, "init", "--initial-branch=master", "--object-format=sha1")
        git(self.root, "config", "user.email", "cleanup@example.invalid")
        git(self.root, "config", "user.name", "Cleanup fixture")
        git(self.root, "config", "commit.gpgsign", "false")
        git(self.root, "commit", "--allow-empty", "-m", "initial")
        git(self.root, "branch", "release/completed")
        git(self.root, "config", "core.hooksPath", str(self.hooks))
        tip = git(self.root, "rev-parse", "refs/heads/release/completed").decode().strip()
        self.intent, self.digest = write_intent(self.private, self.root, "release/completed", tip)

    def _write_existing(self, exit_code: int = 0) -> None:
        self.existing.write_text(
            "#!/bin/sh\n"
            f"while IFS= read -r line; do printf '%s\\t%s\\n' \"$1\" \"$line\" >> {shlex.quote(str(self.log))}; done\n"
            f"exit {exit_code}\n"
        )
        self.existing.chmod(0o700)

    def _install_dispatcher_hook(self) -> None:
        hook = self.hooks / "reference-transaction"
        command = [
            shlex.quote(sys.executable), shlex.quote(str(DISPATCHER)), "--intent", shlex.quote(str(self.intent)),
            "--intent-sha256", shlex.quote(self.digest), "--existing-hook", shlex.quote(str(self.existing)),
        ]
        hook.write_text("#!/bin/sh\nexec " + " ".join(command) + " \"$1\"\n")
        hook.chmod(0o700)

    def _records(self) -> list[str]:
        return self.log.read_text().splitlines() if self.log.exists() else []

    def _rewrite_intent(self, **changes: object) -> tuple[Path, str]:
        payload = json.loads(self.intent.read_text(encoding="utf-8"))
        payload.update(changes)
        raw = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode()
        self.intent.write_bytes(raw)
        return self.intent, hashlib.sha256(raw).hexdigest()

    def test_native_delete_runs_existing_hook_and_commits(self) -> None:
        self._write_existing()
        self._install_dispatcher_hook()
        git(self.root, "branch", "-d", "release/completed")
        self.assertEqual(git(self.root, "for-each-ref", "--format=%(refname)", "refs/heads/release/completed"), b"")
        self.assertTrue(any(line.startswith("prepared\t") for line in self._records()))
        self.assertTrue(any(line.startswith("committed\t") for line in self._records()))

    def test_existing_hook_rejection_precedes_checker_and_preserves_ref(self) -> None:
        self._write_existing(23)
        self._install_dispatcher_hook()
        tip = git(self.root, "rev-parse", "HEAD").decode().strip()
        with self.assertRaises(CleanupRefused):
            git(self.root, "branch", "-d", "release/completed")
        self.assertEqual(git(self.root, "rev-parse", "HEAD").decode().strip(), tip)
        self.assertEqual(git(self.root, "rev-parse", "refs/heads/release/completed").decode().strip(), tip)
        records = self._records()
        self.assertTrue(records)
        self.assertIn(records[0].split("\t")[0], ("preparing", "prepared"))
        self.assertNotIn("committed", "\n".join(records))

    def test_existing_failure_status_is_preserved_before_malformed_intent(self) -> None:
        self._write_existing(23)
        self.intent.write_text("not-json", encoding="utf-8")
        result = subprocess.run(
            [sys.executable, str(DISPATCHER), "--intent", str(self.intent), "--intent-sha256", self.digest,
             "--existing-hook", str(self.existing), "prepared"], cwd=self.root,
            input=b"old new refs/heads/release/completed\n", capture_output=True, check=False,
        )
        self.assertEqual(result.returncode, 23)
        self.assertEqual(self._records(), ["prepared\told new refs/heads/release/completed"])

    def test_malformed_intent_still_runs_existing_hook_with_same_phase_and_stdin(self) -> None:
        self._write_existing()
        self.intent.write_text("not-json", encoding="utf-8")
        self.digest = hashlib.sha256(self.intent.read_bytes()).hexdigest()
        result = subprocess.run(
            [sys.executable, str(DISPATCHER), "--intent", str(self.intent), "--intent-sha256", self.digest,
             "--existing-hook", str(self.existing), "prepared"], cwd=self.root,
            input=b"old new refs/heads/release/completed\n", capture_output=True, check=False,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(self._records(), ["prepared\told new refs/heads/release/completed"])

    def test_foreign_root_is_rejected_and_target_ref_is_unchanged(self) -> None:
        foreign = Path(self.temporary.name) / "foreign"
        foreign.mkdir()
        git(foreign, "init", "--initial-branch=master", "--object-format=sha1")
        git(foreign, "config", "user.email", "cleanup@example.invalid")
        git(foreign, "config", "user.name", "Cleanup fixture")
        git(foreign, "config", "commit.gpgsign", "false")
        git(foreign, "commit", "--allow-empty", "-m", "foreign repository")
        git(foreign, "branch", "release/completed")
        actual = git(foreign, "rev-parse", "HEAD").decode().strip()
        self._write_existing()
        self._install_dispatcher_hook()
        git(foreign, "config", "core.hooksPath", str(self.hooks))
        tip = git(self.root, "rev-parse", "refs/heads/release/completed").decode().strip()
        with self.assertRaises(CleanupRefused):
            git(foreign, "branch", "-d", "release/completed")
        self.assertEqual(git(foreign, "rev-parse", "refs/heads/release/completed").decode().strip(), actual)
        self.assertEqual(git(self.root, "rev-parse", "refs/heads/release/completed").decode().strip(), tip)

    def test_tampered_intent_digest_preserves_ref_after_existing_hook(self) -> None:
        self._write_existing()
        self._install_dispatcher_hook()
        self.intent.write_text(self.intent.read_text(encoding="utf-8") + "\n", encoding="utf-8")
        tip = git(self.root, "rev-parse", "refs/heads/release/completed").decode().strip()
        with self.assertRaises(CleanupRefused):
            git(self.root, "branch", "-d", "release/completed")
        self.assertEqual(git(self.root, "rev-parse", "refs/heads/release/completed").decode().strip(), tip)
        self.assertTrue(any(line.startswith("prepared\t") for line in self._records()))

    def test_intent_rejects_symlink_oversize_mode_and_foreign_digest(self) -> None:
        with self.assertRaises(CleanupRefused):
            read_intent(self.intent, "0" * 64)
        os.chmod(self.intent, 0o644)
        with self.assertRaises(CleanupRefused):
            read_intent(self.intent, self.digest)
        os.chmod(self.intent, 0o600)
        self.intent.write_bytes(self.intent.read_bytes() + b"x" * (MAX_INTENT_BYTES + 1))
        with self.assertRaises(CleanupRefused):
            read_intent(self.intent, hashlib.sha256(self.intent.read_bytes()).hexdigest())

    def test_intent_rejects_symlink_parent_and_unknown_schema_or_keys(self) -> None:
        linked = Path(self.temporary.name) / "private-link"
        linked.symlink_to(self.private, target_is_directory=True)
        with self.assertRaises(CleanupRefused):
            read_intent(linked / self.intent.name, self.digest)
        path, digest = self._rewrite_intent(schema="unknown")
        with self.assertRaises(CleanupRefused):
            read_intent(path, digest)
        path, digest = self._rewrite_intent(schema=1, unexpected=True)
        with self.assertRaises(CleanupRefused):
            read_intent(path, digest)

    def test_disabled_existing_owner_cannot_admit_native_deletion(self) -> None:
        self._write_existing()
        self._install_dispatcher_hook()
        tip = git(self.root, "rev-parse", "HEAD").decode().strip()
        environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        result = subprocess.run(
            ["git", "branch", "-d", "release/completed"], cwd=self.root,
            env=environment | {"LEFTHOOK": "0"}, capture_output=True, check=False,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(git(self.root, "rev-parse", "refs/heads/release/completed").decode().strip(), tip)
        self.assertTrue(self._records())


if __name__ == "__main__":
    unittest.main()
