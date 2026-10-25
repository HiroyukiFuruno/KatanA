"""Native executable contracts for cleanup owner delegation."""

import os
from pathlib import Path
import shlex
import signal
import subprocess
import sys
import tempfile
import unittest

from release_cleanup_transaction import MAX_TRANSACTION_BYTES

DISPATCHER = Path(__file__).with_name("cleanup_reference_dispatcher.py")


class CleanupOwnerDelegationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.owner = self.root / "existing-hook"
        self.received = self.root / "received"
        self.owner.write_text(
            "#!/bin/sh\ncat > " + shlex.quote(str(self.received)) + "\nexit 23\n"
        )
        self.owner.chmod(0o700)

    def _run(self, owner: str, payload: bytes, environment: dict[str, str] | None = None):
        return subprocess.run(
            [sys.executable, str(DISPATCHER), "--intent", str(self.root / "missing.json"),
             "--intent-sha256", "0" * 64, "--existing-hook", owner, "prepared"],
            cwd=self.root, env=environment, input=payload, capture_output=True, check=False,
        )

    def test_oversize_input_reaches_existing_owner_before_guard(self) -> None:
        payload = b"x" * (MAX_TRANSACTION_BYTES + 1)
        result = self._run(str(self.owner), payload)
        self.assertEqual(result.returncode, 23, result.stderr)
        self.assertEqual(self.received.read_bytes(), payload)

    def test_relative_owner_is_not_replaced_by_path_lookup(self) -> None:
        foreign = self.root / "foreign"
        foreign.mkdir()
        replacement = foreign / self.owner.name
        replacement.write_text("#!/bin/sh\nexit 0\n")
        replacement.chmod(0o700)
        environment = os.environ | {"PATH": str(foreign) + os.pathsep + os.environ["PATH"]}
        payload = b"original owner input\n"
        result = self._run(self.owner.name, payload, environment)
        self.assertEqual(result.returncode, 23, result.stderr)
        self.assertEqual(self.received.read_bytes(), payload)

    def test_successful_owner_receives_full_oversize_input_before_refusal(self) -> None:
        self.owner.write_text("#!/bin/sh\ncat > " + shlex.quote(str(self.received)) + "\nexit 0\n")
        payload = b"x" * (MAX_TRANSACTION_BYTES * 2)
        result = self._run(str(self.owner), payload)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(b"reference transaction exceeds the input bound", result.stderr)
        self.assertEqual(self.received.read_bytes(), payload)

    def test_early_owner_exit_preserves_status_without_broken_pipe_traceback(self) -> None:
        self.owner.write_text("#!/bin/sh\nexit 23\n")
        result = self._run(str(self.owner), b"x" * (MAX_TRANSACTION_BYTES * 32))
        self.assertEqual(result.returncode, 23, result.stderr)
        self.assertEqual(result.stderr, b"")

    def test_owner_signal_termination_is_preserved(self) -> None:
        self.owner.write_text("#!/bin/sh\nkill -TERM $$\n")
        result = self._run(str(self.owner), b"transaction\n")
        self.assertEqual(result.returncode, -signal.SIGTERM, result.stderr)


if __name__ == "__main__":
    unittest.main()
