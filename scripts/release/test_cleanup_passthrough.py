"""Owner-first passthrough contracts for cleanup reference dispatch."""

import os
from pathlib import Path
import shlex
import signal
import subprocess
import sys
import tempfile
import unittest

from release_cleanup_transaction import MAX_TRANSACTION_BYTES
from release_cleanup_audit import git
from cleanup_intent import write_intent

DISPATCHER = Path(__file__).with_name("cleanup_reference_dispatcher.py")


class CleanupPassthroughTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        root = Path(self.temporary.name)
        self.owner = root / "existing-hook"
        self.received = root / "received"
        self.environment = root / "environment"
        self.transactions = root / "transactions"
        self.phases = root / "phases"
        self._write_owner()

    def _write_owner(self, exit_code: int = 0, signal_number: int | None = None) -> None:
        action = f"kill -{signal_number} $$" if signal_number else f"exit {exit_code}"
        self.owner.write_text(
            "#!/bin/sh\n"
            f"printf '%s\\n' \"$1\" > {shlex.quote(str(self.environment))}\n"
            f"printf '%s\\n' \"$1\" >> {shlex.quote(str(self.phases))}\n"
            f"printf '%s\\n' \"${{KATANA_CLEANUP_INTENT-unset}}\" >> {shlex.quote(str(self.environment))}\n"
            f"printf '%s\\n' \"${{KATANA_CLEANUP_INTENT_SHA256-unset}}\" >> {shlex.quote(str(self.environment))}\n"
            f"cat > {shlex.quote(str(self.received))}\n"
            f"cat {shlex.quote(str(self.received))} >> {shlex.quote(str(self.transactions))}\n{action}\n"
        )
        self.owner.chmod(0o700)

    def _run(self, phase: str = "prepared", payload: bytes = b"full transaction\n",
             environment: dict[str, str] | None = None) -> subprocess.CompletedProcess[bytes]:
        clean = {key: value for key, value in (environment or os.environ).items()
                 if not key.startswith("GIT_")}
        return subprocess.run(
            [sys.executable, str(DISPATCHER), "--existing-hook", str(self.owner), phase],
            cwd=self.owner.parent, env=clean, input=payload, capture_output=True, check=False,
        )

    def test_no_intent_forwards_phase_and_full_stdin_with_owner_exit(self) -> None:
        payload = b"old new refs/heads/master\n" * 2
        result = self._run("committed", payload, os.environ | {
            "KATANA_CLEANUP_INTENT": "/foreign/intent",
            "KATANA_CLEANUP_INTENT_SHA256": "f" * 64,
        })
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.received.read_bytes(), payload)
        self.assertEqual(self.environment.read_text().splitlines(), ["committed", "unset", "unset"])

    def test_owner_nonzero_is_returned_before_one_sided_intent_rejection(self) -> None:
        self._write_owner(23)
        result = self._run(environment=os.environ | {"KATANA_CLEANUP_INTENT": "/tmp/intent"})
        self.assertEqual(result.returncode, 23, result.stderr)
        self.assertEqual(self.received.read_bytes(), b"full transaction\n")

    def test_one_sided_intent_is_typed_refusal_after_owner_success(self) -> None:
        for field, value in (("KATANA_CLEANUP_INTENT", "/tmp/intent"),
                             ("KATANA_CLEANUP_INTENT_SHA256", "f" * 64)):
            with self.subTest(field=field):
                result = self._run(environment=os.environ | {field: value})
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(b"cleanup intent requires both path and digest", result.stderr)
                self.assertEqual(self.received.read_bytes(), b"full transaction\n")

    def test_no_intent_does_not_apply_cleanup_input_bound(self) -> None:
        payload = b"x" * (MAX_TRANSACTION_BYTES * 2)
        result = self._run(payload=payload)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.received.read_bytes(), payload)

    def test_lefthook_zero_does_not_add_no_intent_cleanup_refusal(self) -> None:
        result = self._run(environment=os.environ | {"LEFTHOOK": "0"})
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_owner_signal_status_is_preserved_without_dispatcher_traceback(self) -> None:
        self._write_owner(signal_number=signal.SIGTERM)
        result = self._run(payload=b"transaction\n")
        self.assertEqual(result.returncode, -signal.SIGTERM)
        self.assertEqual(result.stderr, b"")

    def _repository(self, name: str) -> Path:
        repository = self.owner.parent / name
        repository.mkdir()
        git(repository, "init", "--initial-branch=master", "--object-format=sha1")
        git(repository, "config", "user.email", "cleanup@example.invalid")
        git(repository, "config", "user.name", "Cleanup fixture")
        git(repository, "config", "commit.gpgsign", "false")
        git(repository, "commit", "--allow-empty", "-m", "initial")
        self._install(repository)
        return repository

    def _install(self, repository: Path) -> None:
        hooks = self.owner.parent / (repository.name + "-hooks")
        hooks.mkdir()
        git(repository, "config", "core.hooksPath", str(hooks))
        hook = hooks / "reference-transaction"
        hook.write_text("#!/bin/sh\nexec " + shlex.join(
            [sys.executable, str(DISPATCHER), "--existing-hook", str(self.owner)]
        ) + " \"$1\"\n")
        hook.chmod(0o700)

    def test_native_git_commit_uses_no_intent_passthrough(self) -> None:
        repository = self._repository("repository")
        before = git(repository, "rev-parse", "HEAD")
        git(repository, "commit", "--allow-empty", "-m", "passthrough")
        self.assertNotEqual(git(repository, "rev-parse", "HEAD"), before)
        records = self.transactions.read_text().splitlines()
        self.assertTrue(any(record.split()[-1] == "refs/heads/master" for record in records))
        self.assertIn("committed", self.phases.read_text().splitlines())

    def test_native_git_pull_uses_no_intent_passthrough(self) -> None:
        upstream = self._repository("upstream")
        controller = self.owner.parent / "controller"
        git(upstream, "clone", "--no-local", str(upstream), str(controller))
        self._install(controller)
        git(upstream, "commit", "--allow-empty", "-m", "next")
        git(controller, "pull", "--ff-only", str(upstream), "master")
        self.assertEqual(git(controller, "rev-parse", "HEAD"), git(upstream, "rev-parse", "HEAD"))
        self.assertEqual(self.environment.read_text().splitlines(), ["committed", "unset", "unset"])

    def test_native_delete_consumes_environment_intent_without_leaking_to_owner(self) -> None:
        repository = self._repository("repository")
        git(repository, "branch", "release/completed")
        tip = git(repository, "rev-parse", "HEAD").decode().strip()
        private = self.owner.parent / "private"
        private.mkdir(mode=0o700)
        path, digest = write_intent(private, repository, "release/completed", tip)
        environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        result = subprocess.run(
            ["git", "branch", "-d", "release/completed"], cwd=repository,
            env=environment | {"KATANA_CLEANUP_INTENT": str(path), "KATANA_CLEANUP_INTENT_SHA256": digest},
            input=b"", capture_output=True, check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(git(repository, "for-each-ref", "refs/heads/release/completed"), b"")
        self.assertEqual(self.environment.read_text().splitlines(), ["committed", "unset", "unset"])


if __name__ == "__main__":
    unittest.main()
