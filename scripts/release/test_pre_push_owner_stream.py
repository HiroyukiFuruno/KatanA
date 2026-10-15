"""Native pre-push owner stream and status contracts."""

import os
from pathlib import Path
import shlex
import signal
import subprocess
import sys
import tempfile
import unittest


HERE = Path(__file__).resolve().parent
DISPATCHER = HERE / "pre_push_dispatcher.py"
MAX_STDIN_BYTES = 1024 * 1024


class PrePushOwnerStreamTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        source = self.root / "scripts/release"
        source.mkdir(parents=True)
        for path in HERE.glob("*.py"):
            if not path.name.startswith("test"):
                target = source / path.name
                target.write_bytes(path.read_bytes())
        subprocess.run(["git", "init", "-q", str(self.root)], check=True, env=self.clean_env())
        for key, value in (("user.name", "Fixture"), ("user.email", "fixture@example.invalid"),
                           ("commit.gpgsign", "false")):
            subprocess.run(["git", "config", key, value], cwd=self.root, check=True, env=self.clean_env())
        subprocess.run(["git", "commit", "--allow-empty", "-m", "init"], cwd=self.root,
                       check=True, env=self.clean_env(), capture_output=True)
        self.owner = self.root / "existing-hook"
        self.owner_log = self.root / "owner-input"
        self.checker_log = self.root / "checker-input"
        self.checker = source / "check-push-dependency-evidence.py"
        self._write_checker()
        self._write_owner()

    @staticmethod
    def clean_env() -> dict[str, str]:
        return {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}

    def _write_owner(self, status: str = "0", signal_number: int | None = None) -> None:
        action = f"kill -{signal_number} $$" if signal_number else f"exit {status}"
        self.owner.write_text(
            "#!/bin/sh\n"
            f"cat > {shlex.quote(str(self.owner_log))}\n"
            f"{action}\n"
        )
        self.owner.chmod(0o700)

    def _write_checker(self) -> None:
        self.checker.write_text(
            "#!/usr/bin/env python3\nimport pathlib\n"
            "def main(arguments=None, *, verified_source=None, payload=None):\n"
            f" pathlib.Path({str(self.checker_log)!r}).write_bytes(payload or b'')\n"
            " return 0\n"
        )
        self.checker.chmod(0o700)

    def _run(self, payload: bytes, *hook_args: str, environment: dict[str, str] | None = None,
             existing_hook: Path | None = None):
        return subprocess.run(
            [sys.executable, str(DISPATCHER), "--repository", "ACME/KatanA",
             "--existing-hook", str(existing_hook or self.owner), *hook_args],
            cwd=self.root, env=self.clean_env() | (environment or {}), input=payload,
            capture_output=True, check=False,
        )

    def test_oversize_input_reaches_owner_and_owner_23_is_preserved(self) -> None:
        payload = b"x" * (MAX_STDIN_BYTES + 257)
        self._write_owner("23")
        result = self._run(payload, "origin", "url")
        self.assertEqual(result.returncode, 23, result.stderr)
        self.assertEqual(self.owner_log.read_bytes(), payload)
        self.assertFalse(self.checker_log.exists())

    def test_oversize_owner_zero_logs_all_input_then_refuses_before_checker(self) -> None:
        payload = b"y" * (MAX_STDIN_BYTES + 1)
        result = self._run(payload, "origin", "url")
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(self.owner_log.read_bytes(), payload)
        self.assertFalse(self.checker_log.exists())

    def test_lefthook_zero_runs_owner_then_preserves_owner_or_refuses(self) -> None:
        payload = b"transaction\n"
        self._write_owner("23")
        result = self._run(payload, "origin", "url", environment={"LEFTHOOK": "0"})
        self.assertEqual(result.returncode, 23, result.stderr)
        self.assertEqual(self.owner_log.read_bytes(), payload)
        self._write_owner("0")
        result = self._run(payload, "origin", "url", environment={"LEFTHOOK": "0"})
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(self.owner_log.read_bytes(), payload)
        self.assertFalse(self.checker_log.exists())

    def test_relative_owner_basename_does_not_resolve_to_path_shadow(self) -> None:
        foreign = self.root / "foreign"
        foreign.mkdir()
        shadow = foreign / self.owner.name
        shadow.write_text("#!/bin/sh\nexit 17\n")
        shadow.chmod(0o700)
        environment = {"PATH": str(foreign) + os.pathsep + os.environ.get("PATH", "")}
        self._write_owner("23")
        result = self._run(b"payload\n", "origin", "url", environment=environment,
                           existing_hook=Path(self.owner.name))
        self.assertEqual(result.returncode, 23, result.stderr)
        self.assertEqual(self.owner_log.read_bytes(), b"payload\n")

    def test_owner_signal_status_is_negative_signal_without_traceback(self) -> None:
        self._write_owner(signal_number=signal.SIGTERM)
        result = self._run(b"payload\n", "origin", "url")
        self.assertEqual(result.returncode, -signal.SIGTERM)
        self.assertEqual(result.stderr, b"")


if __name__ == "__main__":
    unittest.main()
