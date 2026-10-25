"""Exercise real child exits and isolated shadow environment construction."""

import json
import os
from pathlib import Path
import signal
import sys
import tempfile
import unittest

from nextest_shadow_execution import execute_stage, phase_environment, prepare_output
from nextest_shadow_execution import snapshot_worker
from native_test_run_identity import file_digest


class ShadowExecutionTests(unittest.TestCase):
    def test_worker_snapshot_survives_target_replacement(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name).resolve()
            source, snapshot = root / "worker", root / "snapshot"
            source.write_bytes(b"#!/bin/sh\nexit 0\n")
            source.chmod(0o755)
            digest = snapshot_worker(source, snapshot)
            source.write_bytes(b"instrumented replacement")
            self.assertEqual(file_digest(snapshot), digest)
            self.assertNotEqual(file_digest(source), digest)
            self.assertEqual(snapshot.stat().st_mode & 0o777, 0o555)

    def test_worker_snapshot_rejects_alias_nonexec_and_existing_destination(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name).resolve()
            source, snapshot = root / "worker", root / "snapshot"
            source.write_bytes(b"worker")
            with self.assertRaises(ValueError):
                snapshot_worker(source, snapshot)
            source.chmod(0o755)
            alias = root / "alias"
            alias.symlink_to(source)
            with self.assertRaises(ValueError):
                snapshot_worker(alias, snapshot)
            snapshot.write_bytes(b"existing")
            with self.assertRaises(FileExistsError):
                snapshot_worker(source, snapshot)
            self.assertEqual(snapshot.read_bytes(), b"existing")

    def test_receipt_failure_does_not_mask_real_child_failure(self):
        for status in (0, 7):
            with tempfile.TemporaryDirectory() as name:
                root = Path(name).resolve()
                code = ("from pathlib import Path; Path('stage/command.json').mkdir(); "
                        f"raise SystemExit({status})")
                command = [sys.executable, "-c", code]
                if status:
                    result = execute_stage(root, root / "stage", command, dict(os.environ))
                    self.assertEqual(result["exit_code"], status)
                else:
                    with self.assertRaises(OSError):
                        execute_stage(root, root / "stage", command, dict(os.environ))

    def test_child_runs_once_and_failure_is_not_retried(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name).resolve()
            marker = root / "marker"
            code = "from pathlib import Path; Path('marker').write_text('once'); raise SystemExit(7)"
            result = execute_stage(root, root / "stage", [sys.executable, "-c", code], dict(os.environ))
            self.assertEqual(result["exit_code"], 7)
            self.assertEqual(marker.read_text(), "once")
            self.assertEqual(json.loads((root / "stage/command.json").read_text()), result)
            self.assertGreaterEqual(result["elapsed_seconds"], 0)
            self.assertTrue(result["started_at"] and result["finished_at"])

    def test_signal_is_retained_as_a_raw_child_status(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name).resolve()
            code = ("import os,signal; os.kill(os.getpid(), signal.SIGTERM)"
                    if os.name == "posix" else "raise SystemExit(9)")
            result = execute_stage(root, root / "stage", [sys.executable, "-c", code], dict(os.environ))
            self.assertEqual(result["exit_code"], -signal.SIGTERM if os.name == "posix" else 9)

    def test_phase_flags_preserve_encoded_rustflags_and_parent(self):
        for encoded in ("", "-D\x1fwarnings"):
            original = {"CARGO_ENCODED_RUSTFLAGS": encoded, "RUSTFLAGS": "ignored-by-rustc"}
            result = phase_environment(original, Path("/physical/run/phase"))
            expected = (encoded + "\x1f" if encoded else "") + "--cfg\x1fcoverage"
            self.assertEqual(result["CARGO_ENCODED_RUSTFLAGS"], expected)
            self.assertEqual(original["CARGO_ENCODED_RUSTFLAGS"], encoded)
            self.assertEqual(result["NEXTEST_STATE_DIR"], "/physical/run/phase/nextest-state")

    def test_phase_flags_preserve_normal_rustflags(self):
        result = phase_environment({"RUSTFLAGS": "-D warnings", "KATANA_SHADOW_PROFILE_ID": "one"},
                                   Path("/run/phase"))
        self.assertEqual(result["RUSTFLAGS"], "-D warnings --cfg coverage")
        self.assertEqual(result["LLVM_PROFILE_FILE_NAME"], "katana-shadow-one-phase-%4m.profraw")

    def test_output_is_fresh_and_below_repository_tmp(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name).resolve()
            output = root / "tmp/shadow/one"
            prepare_output(root, output)
            self.assertTrue(output.is_dir())
            with self.assertRaises(FileExistsError):
                prepare_output(root, output)
            with self.assertRaises(ValueError):
                prepare_output(root, root / "outside")
            with self.assertRaises(ValueError):
                prepare_output(root, root / "tmp/../outside")

    def test_output_alias_is_rejected_without_writing_through_it(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name).resolve()
            destination = root / "destination"
            destination.mkdir()
            (root / "tmp").symlink_to(destination, target_is_directory=True)
            with self.assertRaises(ValueError):
                prepare_output(root, root / "tmp/run")
            self.assertEqual(list(destination.iterdir()), [])


if __name__ == "__main__":
    unittest.main()
