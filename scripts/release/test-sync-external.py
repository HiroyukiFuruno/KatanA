"""Contract tests for optional external registry synchronization orchestration."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).with_name("sync-external.sh")
SECRET = "fixture-secret-must-not-be-logged"


class SyncExternalTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.artifacts = self.root / "artifacts"
        self.artifacts.mkdir()
        self.project = self.root / "project"
        (self.project / "scripts/release").mkdir(parents=True)
        self.script = self.project / "scripts/release/sync-external.sh"
        self.script.write_bytes(SCRIPT.read_bytes())
        self.bin = self.root / "bin"
        self.bin.mkdir()
        self.summary = self.root / "step-summary.md"
        self.command_log = self.root / "komac-calls.jsonl"
        self._write_command("shasum", "#!/bin/sh\nprintf '%064d  %s\\n' 0  \"$3\"\n")
        self._write_command(
            "komac",
            "#!/usr/bin/env python3\n"
            "import json, os, pathlib, sys\n"
            "args = sys.argv[1:]\n"
            "safe_args = [arg for arg in args if arg != os.environ.get('WINGET_GH_TOKEN')]\n"
            "with pathlib.Path(os.environ['KOMAC_LOG']).open('a') as output:\n"
            " output.write(json.dumps(safe_args) + '\\n')\n"
            "mode = os.environ.get('KOMAC_MODE', 'success')\n"
            "if args[0] == 'list' and mode == 'lookup-failure':\n"
            " print('simulated lookup failure', file=sys.stderr)\n"
            " sys.exit(17)\n"
            "if args[0] == 'update' and mode == 'create-ref-failure':\n"
            " print('HiroyukiFuruno does not have the correct permissions to execute `CreateRef`', file=sys.stderr)\n"
            " sys.exit(1)\n",
        )
        self._write_command(
            "update-homebrew.sh",
            "#!/bin/sh\nexit \"${HOMEBREW_HELPER_STATUS:-0}\"\n",
            in_project=True,
        )
        self._write_command(
            "update-linuxbrew.sh",
            "#!/bin/sh\nexit \"${LINUXBREW_HELPER_STATUS:-0}\"\n",
            in_project=True,
        )
        self._write_command("brew", "#!/bin/sh\nexit 0\n")
        for name in ("KatanA-Desktop-1.dmg", "KatanA-linux-x86_64.tar.gz", "KatanA-windows-x86_64.msi"):
            (self.artifacts / name).write_bytes(b"test artifact")

    def _write_command(self, name: str, content: str, *, in_project: bool = False) -> None:
        path = (self.project / "scripts/release" / name) if in_project else (self.bin / name)
        path.write_text(content, encoding="utf-8")
        path.chmod(0o700)

    def _run(
        self,
        *,
        configured: bool = True,
        komac_mode: str = "success",
        homebrew_status: str = "0",
        komac_available: bool = True,
    ) -> subprocess.CompletedProcess[bytes]:
        environment = os.environ.copy()
        environment.update(
            {
                "PATH": str(self.bin) + os.pathsep + environment.get("PATH", ""),
                "GITHUB_STEP_SUMMARY": str(self.summary),
                "KOMAC_LOG": str(self.command_log),
                "KOMAC_MODE": komac_mode,
                "HOMEBREW_HELPER_STATUS": homebrew_status,
            }
        )
        if configured:
            environment["HOMEBREW_KATANA_GIT_TOKEN"] = SECRET
            environment["WINGET_GH_TOKEN"] = SECRET
        else:
            environment.pop("HOMEBREW_KATANA_GIT_TOKEN", None)
            environment.pop("WINGET_GH_TOKEN", None)
        if not komac_available:
            (self.bin / "komac").unlink()
            environment["PATH"] = os.pathsep.join(
                (str(self.bin), str(Path(sys.executable).parent), "/usr/bin", "/bin")
            )
        return subprocess.run(
            ["bash", str(self.script), "0.22.43", "v0.22.43", str(self.artifacts)],
            cwd=self.project,
            env=environment,
            capture_output=True,
            check=False,
        )

    def _result(self) -> dict[str, object]:
        return json.loads((self.artifacts / "external-registry-sync.json").read_text(encoding="utf-8"))

    @staticmethod
    def _targets(result: dict[str, object]) -> dict[str, dict[str, str]]:
        return {target["name"]: target for target in result["targets"]}

    def test_unconfigured_targets_are_recorded_as_skipped(self) -> None:
        result = self._run(configured=False)

        self.assertEqual(result.returncode, 0, result.stderr.decode())
        targets = self._targets(self._result())
        self.assertEqual(set(targets), {"homebrew", "linuxbrew", "winget"})
        self.assertTrue(all(target["status"] == "skipped" for target in targets.values()))
        self.assertIn("| homebrew | skipped | token_not_configured |", self.summary.read_text())
        self.assertNotIn("all", result.stdout.decode().lower())

    def test_configured_targets_succeed_and_preserve_winget_arguments(self) -> None:
        result = self._run()

        self.assertEqual(result.returncode, 0, result.stderr.decode())
        targets = self._targets(self._result())
        self.assertTrue(all(target["status"] == "success" for target in targets.values()))
        calls = [json.loads(line) for line in self.command_log.read_text().splitlines()]
        self.assertEqual([call[0] for call in calls], ["list", "update"])
        self.assertIn("HiroyukiFuruno.katana-desktop", calls[0])
        self.assertIn("https://github.com/HiroyukiFuruno/KatanA/releases/download/v0.22.43/KatanA-windows-x86_64.msi", calls[1])
        self.assertIn("https://github.com/HiroyukiFuruno/KatanA/releases/tag/v0.22.43", calls[1])
        self.assertIn("--submit", calls[1])
        self.assertEqual(targets["winget"]["reason"], "submission_completed")
        self.assertIn("WinGet: submission completed", result.stdout.decode())
        self.assertNotIn("WinGet: synchronized", result.stdout.decode())
        self._assert_secret_not_logged(result.stdout + result.stderr)

    def test_homebrew_failure_is_recorded_and_other_targets_continue(self) -> None:
        result = self._run(homebrew_status="9")

        self.assertEqual(result.returncode, 0, result.stderr.decode())
        targets = self._targets(self._result())
        self.assertEqual(targets["homebrew"]["status"], "failed")
        self.assertEqual(targets["homebrew"]["reason"], "updater_failed")
        self.assertEqual(targets["linuxbrew"]["status"], "success")
        self.assertEqual(targets["winget"]["status"], "success")
        summary = self.summary.read_text()
        self.assertIn("| homebrew | failed | updater_failed |", summary)
        self.assertIn("| linuxbrew | success | updated |", summary)
        self.assertIn("| winget | success | submission_completed |", summary)
        self.assertIn("::warning title=External registry sync failed::", result.stdout.decode())
        self.assertEqual([json.loads(line)[0] for line in self.command_log.read_text().splitlines()], ["list", "update"])

    def test_komac_unavailable_after_install_attempt_is_failed(self) -> None:
        result = self._run(komac_available=False)

        self.assertEqual(result.returncode, 0, result.stderr.decode())
        winget = self._targets(self._result())["winget"]
        self.assertEqual(winget["status"], "failed")
        self.assertEqual(winget["reason"], "komac_unavailable")
        self.assertIn("| winget | failed | komac_unavailable |", self.summary.read_text())
        self.assertIn("::warning title=WinGet同期失敗::", result.stdout.decode())

    def test_winget_lookup_error_is_failed_not_reported_as_missing_package(self) -> None:
        result = self._run(komac_mode="lookup-failure")

        self.assertEqual(result.returncode, 0, result.stderr.decode())
        targets = self._targets(self._result())
        self.assertEqual(targets["winget"]["status"], "failed")
        self.assertEqual(targets["winget"]["reason"], "package_lookup_failed")
        summary = self.summary.read_text()
        self.assertIn("| winget | failed | package_lookup_failed |", summary)
        self.assertIn("::warning title=WinGet同期失敗::", result.stdout.decode())
        self._assert_secret_not_logged(result.stdout + result.stderr)

    def test_create_ref_permission_error_is_failed_and_warned_without_secret(self) -> None:
        result = self._run(komac_mode="create-ref-failure")

        self.assertEqual(result.returncode, 0, result.stderr.decode())
        targets = self._targets(self._result())
        self.assertEqual(targets["winget"]["status"], "failed")
        self.assertEqual(targets["winget"]["reason"], "create_ref_permission_denied")
        self.assertIn("/issues/1142", targets["winget"]["guidance"])
        summary = self.summary.read_text()
        self.assertIn("| winget | failed | Check WinGet fork synchronization and credential access", summary)
        self.assertIn("Komac issue #1142", summary)
        self.assertIn("::warning title=WinGet同期失敗::", result.stdout.decode())
        self.assertIn("1 failed target(s); product release is not blocked", result.stdout.decode())
        self._assert_secret_not_logged(result.stdout + result.stderr)

    def _assert_secret_not_logged(self, output: bytes) -> None:
        result_json = (self.artifacts / "external-registry-sync.json").read_bytes()
        summary = self.summary.read_bytes()
        calls = self.command_log.read_bytes()
        self.assertNotIn(SECRET.encode(), output)
        self.assertNotIn(SECRET.encode(), result_json)
        self.assertNotIn(SECRET.encode(), summary)
        self.assertNotIn(SECRET.encode(), calls)


if __name__ == "__main__":
    unittest.main()
