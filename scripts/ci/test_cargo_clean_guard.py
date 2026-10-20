import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import cargo_sweep_guard as sweep_guard
from test_cargo_sweep_guard import (
    build_environment,
    build_release,
    create_crate,
    create_target_triple_profiles,
    finally_release,
    launch_build,
    wait_for_barrier,
)


class CargoCleanGuardTest(unittest.TestCase):
    def test_active_real_build_is_refused_then_finishes_and_cleans(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = create_crate(root)
            target, barrier, environment = build_environment(root)
            barrier.mkdir()
            create_target_triple_profiles(target)
            process = launch_build(manifest, environment)
            try:
                wait_for_barrier(barrier / "entered", process)
                active_fingerprint = target / "debug" / ".fingerprint"
                refused = run_clean_guard(manifest, environment, "CARGO_NET_OFFLINE=true cargo")
                self.assertEqual(75, refused.returncode, refused.stderr)
                self.assertTrue(active_fingerprint.is_dir())
                barrier.joinpath("release").touch()
                output, _ = process.communicate(timeout=60)
                self.assertEqual(0, process.returncode, output)
                if sweep_guard.fcntl is None or os.name != "posix":
                    cleaned = run_clean_guard(manifest, environment, "CARGO_NET_OFFLINE=true cargo")
                    self.assertEqual(75, cleaned.returncode, cleaned.stderr)
                    self.assertTrue(active_fingerprint.exists())
                else:
                    (target / "CACHEDIR.TAG").write_text(
                        "Signature: 8a477f597d28d172789f06886806bc55\n",
                        encoding="ascii",
                    )
                    cleaned = run_clean_guard(manifest, environment, "env cargo")
                    assert_cleaned(self, cleaned, active_fingerprint)
            finally:
                finally_release(process, barrier)

    def test_busy_target_is_refused_then_idle_target_is_cleaned(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = create_crate(root)
            target, barrier, environment = build_environment(root)
            barrier.mkdir()
            barrier.joinpath("release").touch()
            create_target_triple_profiles(target)
            built = build_release(manifest, environment)
            self.assertEqual(0, built.returncode, built.stdout + built.stderr)
            (target / "CACHEDIR.TAG").write_text(
                "Signature: 8a477f597d28d172789f06886806bc55\n",
                encoding="ascii",
            )
            artifact = target / "release" / "libsweep_guard_probe.rlib"
            self.assertTrue(artifact.is_file())
            if sweep_guard.fcntl is None or os.name != "posix":
                assert_non_posix_refusal(self, manifest, environment, artifact)
                return
            assert_manual_lock_cycle(self, manifest, target, environment, artifact)

    def test_unsafe_and_unknown_targets_fail_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = create_crate(root)
            for configured in (Path("/"), Path.home(), root, root.parent):
                result = run_clean_guard(
                    manifest, {**os.environ, "CARGO_TARGET_DIR": str(configured)}
                )
                self.assertEqual(75, result.returncode, result.stderr)
            if os.name == "posix":
                linked = root / "linked-target"
                linked.symlink_to(root / "target")
                result = run_clean_guard(
                    manifest, {**os.environ, "CARGO_TARGET_DIR": str(linked)}
                )
                self.assertEqual(75, result.returncode, result.stderr)
            target = root / "target"
            (target / "unknown").mkdir(parents=True)
            result = run_clean_guard(
                manifest, {**os.environ, "CARGO_TARGET_DIR": str(target)}
            )
            self.assertEqual(75, result.returncode, result.stderr)
            malformed = run_clean_guard(manifest, os.environ.copy(), "'unterminated")
            self.assertEqual(75, malformed.returncode, malformed.stderr)

    def test_absent_target_is_a_successful_noop(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = create_crate(root)
            target = root / "missing-target"
            result = run_clean_guard(
                manifest, {**os.environ, "CARGO_TARGET_DIR": str(target)}
            )
            self.assertEqual(0, result.returncode, result.stderr)
            self.assertIn("target is absent", result.stdout)


def run_clean_guard(manifest, environment, cargo="cargo"):
    return subprocess.run(
        [
            sys.executable,
            str(Path(__file__).with_name("cargo_clean_guard.py")),
            "--manifest-path",
            str(manifest),
            "--cargo",
            cargo,
        ],
        env={**environment, "CARGO_NET_OFFLINE": "true"},
        capture_output=True,
        text=True,
        timeout=60,
    )


def assert_non_posix_refusal(test_case, manifest, environment, artifact):
    refused = run_clean_guard(manifest, environment)
    test_case.assertEqual(75, refused.returncode, refused.stderr)
    test_case.assertTrue(artifact.is_file())


def assert_manual_lock_cycle(test_case, manifest, target, environment, artifact):
    descriptors = sweep_guard.acquire_profile_locks(sweep_guard.profile_directories(target))
    try:
        busy = run_clean_guard(manifest, environment)
        test_case.assertEqual(75, busy.returncode, busy.stderr)
        test_case.assertTrue(artifact.is_file())
    finally:
        sweep_guard.release_profile_locks(descriptors)
    cleaned = run_clean_guard(manifest, environment)
    test_case.assertEqual(0, cleaned.returncode, cleaned.stderr)
    test_case.assertFalse(artifact.exists())


def assert_cleaned(test_case, result, fingerprint):
    test_case.assertEqual(0, result.returncode, result.stderr)
    test_case.assertFalse(fingerprint.exists())


if __name__ == "__main__":
    unittest.main()
