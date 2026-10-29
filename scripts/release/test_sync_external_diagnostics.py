"""Updater failure metadata is preserved without publishing raw helper output."""
import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location("sync_fixture", Path(__file__).with_name("test-sync-external.py"))
FIXTURE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(FIXTURE)


class SyncExternalDiagnosticsTests(unittest.TestCase):
    setUp = FIXTURE.SyncExternalTests.setUp
    _write_command = FIXTURE.SyncExternalTests._write_command
    _run = FIXTURE.SyncExternalTests._run
    _result = FIXTURE.SyncExternalTests._result
    _targets = staticmethod(FIXTURE.SyncExternalTests._targets)
    _assert_secret_not_logged = FIXTURE.SyncExternalTests._assert_secret_not_logged

    def test_http_and_exit_code_are_retained_without_secret_output(self):
        self._write_command("update-homebrew.sh", '#!/bin/sh\n'
                            'echo "$HOMEBREW_KATANA_GIT_TOKEN" >&2\n'
                            'echo "Error: failed update (HTTP 401)." >&2\nexit 9\n', in_project=True)
        result = self._run()
        self.assertEqual(result.returncode, 0, result.stderr.decode())
        targets = self._targets(self._result())
        self.assertEqual(targets["homebrew"]["updater_exit_code"], 9)
        self.assertEqual(targets["homebrew"]["http_status"], 401)
        self.assertEqual(targets["homebrew"]["reason"], "updater_failed")
        self.assertEqual(targets["linuxbrew"]["status"], "success")
        self.assertIn("exit=9, http=401", result.stdout.decode())
        self._assert_secret_not_logged(result.stdout + result.stderr)

    def test_unclassified_output_is_not_echoed_or_invented_as_http_status(self):
        self._write_command("update-homebrew.sh", '#!/bin/sh\n'
                            'echo "$HOMEBREW_KATANA_GIT_TOKEN" >&2\n'
                            'echo "unclassified output (HTTP 999)" >&2\nexit 7\n', in_project=True)
        result = self._run()
        self.assertEqual(result.returncode, 0, result.stderr.decode())
        target = self._targets(self._result())["homebrew"]
        self.assertEqual(target["updater_exit_code"], 7)
        self.assertIsNone(target["http_status"])
        self.assertNotIn("unclassified output", result.stdout.decode())
        self._assert_secret_not_logged(result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
