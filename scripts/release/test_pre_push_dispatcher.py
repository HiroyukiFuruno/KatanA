from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
DISPATCHER = HERE / "pre_push_dispatcher.py"


class DispatcherTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        (self.root / "scripts/release").mkdir(parents=True)
        shutil.copy2(DISPATCHER, self.root / "scripts/release/pre_push_dispatcher.py")
        subprocess.run(["git", "init", "-q", str(self.root)], check=True, env=self.clean_env())
        subprocess.run(["git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "--allow-empty", "-m", "init"], cwd=self.root, check=True, env=self.clean_env(), capture_output=True)
        self.log = self.root / "events"
        self.existing = self.root / "existing-hook"
        self.checker = self.root / "scripts/release/check-push-dependency-evidence.py"
        self._child(self.existing, "existing")
        self._child(self.checker, "checker")

    def tearDown(self) -> None:
        self.temp.cleanup()

    @staticmethod
    def clean_env() -> dict[str, str]:
        return {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}

    def _child(self, path: Path, name: str, status: int = 0) -> None:
        if name == "checker":
            path.write_text(
                "#!/usr/bin/env python3\nimport pathlib\n"
                "def main(arguments=None, *, verified_source=None, payload=None):\n"
                f" p=pathlib.Path({str(self.log)!r})\n p.write_text(p.read_text() + 'checker:' + repr(arguments) + ':' + repr(verified_source) + ':' + payload.decode(), encoding='utf-8') if p.exists() else p.write_text('checker:' + repr(arguments) + ':' + repr(verified_source) + ':' + payload.decode(), encoding='utf-8')\n"
                f" return {status}\n",
                encoding="utf-8",
            )
            path.chmod(0o755)
            return
        path.write_text(
            "#!/usr/bin/env python3\nimport pathlib,sys\n"
            f"p=pathlib.Path({str(self.log)!r})\n"
            f"p.write_text(p.read_text() + {name!r} + ':' + repr(sys.argv[1:]) + ':' + sys.stdin.buffer.read().decode(), encoding='utf-8') if p.exists() else p.write_text({name!r} + ':' + repr(sys.argv[1:]) + ':' + sys.stdin.buffer.read().decode(), encoding='utf-8')\n"
            f"raise SystemExit({status})\n",
            encoding="utf-8",
        )
        path.chmod(0o755)

    def run_dispatcher(self, payload: bytes = b"line\n", *args: str, environment=None) -> subprocess.CompletedProcess[bytes]:
        return subprocess.run(
            [sys.executable, str(DISPATCHER), "--repository", "ACME/KatanA", "--existing-hook", str(self.existing), *args],
            cwd=self.root,
            input=payload,
            capture_output=True,
            env=self.clean_env() | (environment or {}),
        )

    def test_disabled_owner_is_not_successful_verification(self) -> None:
        result = self.run_dispatcher(b"x\n", "origin", "url", environment={"LEFTHOOK": "0"})
        self.assertEqual(result.returncode, 2)
        self.assertIn(b"disabled existing owner", result.stderr)
        self.assertIn("existing:", self.log.read_text())
        self.assertNotIn("checker:", self.log.read_text())

    def test_existing_runs_before_checker_with_same_input_and_args(self) -> None:
        result = self.run_dispatcher(b"refs/heads/main abc\n", "origin", "ssh://remote", "old", "new")
        self.assertEqual(result.returncode, 0, result.stderr)
        data = self.log.read_text()
        self.assertLess(data.index("existing"), data.index("checker"))
        self.assertEqual(data.count("refs/heads/main abc\n"), 2)
        self.assertIn("'origin', 'ssh://remote', 'old', 'new'", data)

    def test_nonzero_existing_aborts_checker(self) -> None:
        self._child(self.existing, "existing", 7)
        result = self.run_dispatcher(b"x\n", "origin", "url")
        self.assertEqual(result.returncode, 7)
        self.assertEqual(self.log.read_text().count("checker"), 0)

    def test_zero_owner_without_execution_receipt_has_no_verified_source(self) -> None:
        result = self.run_dispatcher(b"x\n", "origin", "url")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(":None:x\n", self.log.read_text())

    def test_head_change_by_existing_hook_is_rejected(self) -> None:
        self.existing.write_text(
            "#!/usr/bin/env python3\nimport subprocess\n"
            "subprocess.run(['git', '-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', 'commit', '--allow-empty', '-m', 'changed'], check=True)\n",
            encoding="utf-8",
        )
        self.existing.chmod(0o755)
        result = self.run_dispatcher(b"x\n", "origin", "url")
        self.assertEqual(result.returncode, 2)
        self.assertFalse(self.log.exists())

    def test_input_is_bounded(self) -> None:
        result = self.run_dispatcher(b"x" * (1024 * 1024 + 1), "origin", "url")
        self.assertEqual(result.returncode, 2)
        self.assertTrue(self.log.read_text().endswith("x" * (1024 * 1024 + 1)))
        self.assertNotIn("checker:", self.log.read_text())


if __name__ == "__main__":
    unittest.main()
