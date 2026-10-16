"""Real subprocess tests for the bounded CI command observer."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
CLI = ROOT / "scripts/ci/observe-ci-command.py"


def run_observer(directory: Path, command: list[str], cache: str = "",
                 cwd: Path | None = None, source_sha: str = "abc123") -> subprocess.CompletedProcess[str]:
    output = directory / "timing.json"
    environment = dict(os.environ)
    environment["OBSERVER_TEST_VALUE"] = "passthrough"
    environment["GITHUB_SHA"] = source_sha
    environment["GITHUB_STEP_SUMMARY"] = str(directory / "summary.md")
    return subprocess.run(
        [sys.executable, str(CLI), "--output", str(output), "--cache-hit", cache, "--", *command],
        cwd=cwd or ROOT, env=environment, capture_output=True, text=True, timeout=30,
    )


class ObserveCiCommandTests(unittest.TestCase):
    def test_success_passthrough_and_single_execution(self):
        with tempfile.TemporaryDirectory() as name:
            directory = Path(name)
            counter = directory / "counter"
            code = ("import os,pathlib,sys; p=pathlib.Path(os.environ['COUNTER']); "
                    "p.write_text(p.read_text()+'x' if p.exists() else 'x'); "
                    "print(os.environ['OBSERVER_TEST_VALUE']); print(sys.argv[1]); "
                    "print('child-error-stream', file=sys.stderr)")
            environment = dict(os.environ, COUNTER=str(counter), OBSERVER_TEST_VALUE="passthrough")
            environment["GITHUB_STEP_SUMMARY"] = str(directory / "summary.md")
            command = [sys.executable, str(CLI), "--output", str(directory / "timing.json"),
                       "--cache-hit", "true", "--", sys.executable, "-c", code, "argument"]
            result = subprocess.run(command, cwd=ROOT, env=environment, capture_output=True,
                                    text=True, timeout=30)
            self.assertEqual(result.returncode, 0, result.stderr)
            payload = json.loads((directory / "timing.json").read_text())
            self.assertEqual(counter.read_text(), "x")
            self.assertEqual(payload["command_argv"][-1], "argument")
            self.assertTrue(payload["executed_once"])
            self.assertTrue(payload["cache_hit"])
            self.assertFalse(payload["reuse_authorized"])
            self.assertIn("passthrough", result.stdout)
            self.assertIn("argument", result.stdout)
            self.assertIn("child-error-stream", result.stderr)

    def test_failure_preserves_child_status_and_records_summary(self):
        with tempfile.TemporaryDirectory() as name:
            directory = Path(name)
            result = run_observer(directory, [sys.executable, "-c", "raise SystemExit(7)"], "false")
            self.assertEqual(result.returncode, 7)
            payload = json.loads((directory / "timing.json").read_text())
            self.assertEqual(payload["exit"], 7)
            self.assertEqual(payload["exit_code"], 7)
            self.assertFalse(payload["cache_hit"])
            summary = (directory / "summary.md").read_text()
            self.assertIn("diagnostic only", summary)
            self.assertIn('"GITHUB_SHA": "abc123"', summary)
            self.assertIn('"cargo_lock_sha256":', summary)

    def test_unknown_cache_is_null_and_no_quality_authority(self):
        with tempfile.TemporaryDirectory() as name:
            directory = Path(name)
            result = run_observer(directory, [sys.executable, "-c", "pass"], "")
            self.assertEqual(result.returncode, 0, result.stderr)
            payload = json.loads((directory / "timing.json").read_text())
            self.assertIsNone(payload["cache_hit"])
            self.assertEqual(payload["build_cache_state"], "not_measured")
            self.assertEqual(payload["cold_warm_equivalence"], "not_evaluated")

    @unittest.skipUnless(os.name == "posix", "signal exit codes are POSIX-specific")
    def test_signal_exit_is_normalized_and_raw_status_is_retained(self):
        with tempfile.TemporaryDirectory() as name:
            directory = Path(name)
            code = "import os,signal; os.kill(os.getpid(), signal.SIGTERM)"
            result = run_observer(directory, [sys.executable, "-c", code])
            payload = json.loads((directory / "timing.json").read_text())
            self.assertEqual(result.returncode, 143)
            self.assertEqual(payload["exit"], -signal.SIGTERM)
            self.assertEqual(payload["exit_code"], 143)

    def test_spawn_failure_is_not_reported_as_execution(self):
        with tempfile.TemporaryDirectory() as name:
            directory = Path(name)
            result = run_observer(directory, [str(directory / "missing-command")])
            payload = json.loads((directory / "timing.json").read_text())
            self.assertEqual(result.returncode, 127)
            self.assertFalse(payload["executed_once"])

    def test_lock_digests_are_per_path_and_missing_is_explicit(self):
        with tempfile.TemporaryDirectory() as name:
            directory = Path(name)
            result = run_observer(directory, [sys.executable, "-c", "pass"], cwd=directory)
            missing = json.loads((directory / "timing.json").read_text())["cargo_lock_sha256"]
            self.assertEqual(result.returncode, 0)
            self.assertIsNone(missing["Cargo.lock"])
            self.assertIsNone(missing["scripts/screenshot/Cargo.lock"])
            (directory / "Cargo.lock").write_bytes(b"root-lock\n")
            (directory / "scripts/screenshot").mkdir(parents=True)
            (directory / "scripts/screenshot/Cargo.lock").write_bytes(b"shot-lock\n")
            result = run_observer(directory, [sys.executable, "-c", "pass"], cwd=directory)
            payload = json.loads((directory / "timing.json").read_text())
            self.assertEqual(result.returncode, 0)
            self.assertEqual(payload["cargo_lock_sha256"]["Cargo.lock"],
                             hashlib.sha256(b"root-lock\n").hexdigest())
            self.assertEqual(payload["cargo_lock_sha256"]["scripts/screenshot/Cargo.lock"],
                             hashlib.sha256(b"shot-lock\n").hexdigest())
            self.assertEqual(payload["cwd"], str(directory.resolve()))
            self.assertEqual(payload["lock_observation_phase"], "after_command")

    def test_lock_metadata_failure_preserves_child_failure(self):
        with tempfile.TemporaryDirectory() as name:
            directory = Path(name)
            (directory / "Cargo.lock").mkdir()
            result = run_observer(directory, [sys.executable, "-c", "raise SystemExit(9)"], cwd=directory)
            self.assertEqual(result.returncode, 9)
            self.assertFalse((directory / "timing.json").exists())

    def test_unknown_cache_is_not_injected_into_summary(self):
        with tempfile.TemporaryDirectory() as name:
            directory = Path(name)
            result = run_observer(directory, [sys.executable, "-c", "pass"], "\n### injected",
                                  source_sha="</pre><script>injected</script>")
            self.assertEqual(result.returncode, 0)
            summary = (directory / "summary.md").read_text()
            self.assertNotIn("### injected", summary)
            self.assertNotIn("<script>", summary)
            self.assertIn("&lt;script&gt;", summary)
            self.assertIsNone(json.loads((directory / "timing.json").read_text())["cache_hit"])

    def test_atomic_save_failure_cleans_temporary_and_preserves_child_status(self):
        with tempfile.TemporaryDirectory() as name:
            directory = Path(name)
            (directory / "timing.json").mkdir()
            for status, expected in ((0, 2), (9, 9)):
                result = run_observer(directory, [sys.executable, "-c", f"raise SystemExit({status})"])
                self.assertEqual(result.returncode, expected)
                self.assertEqual(list(directory.glob(".timing.json.*")), [])

    def test_summary_failure_returns_two_on_success_and_child_failure_wins(self):
        with tempfile.TemporaryDirectory() as name:
            directory = Path(name)
            summary = directory / "summary-dir"
            summary.mkdir()
            environment = dict(os.environ, GITHUB_STEP_SUMMARY=str(summary))
            command = [sys.executable, str(CLI), "--output", str(directory / "ok.json"),
                       "--cache-hit", "true", "--", sys.executable, "-c", "pass"]
            success = subprocess.run(command, cwd=ROOT, env=environment, capture_output=True,
                                     text=True, timeout=30)
            self.assertEqual(success.returncode, 2)
            failed_command = command[:-4] + ["--", sys.executable, "-c", "raise SystemExit(9)"]
            failed = subprocess.run(failed_command, cwd=ROOT, env=environment,
                                    capture_output=True, text=True, timeout=30)
            self.assertEqual(failed.returncode, 9)


if __name__ == "__main__":
    unittest.main()
