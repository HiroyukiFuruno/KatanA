"""Process-level tests for target cleanup leases and managed Cargo checks."""

from __future__ import annotations

import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest

import cargo_target_lease as lease
import cargo_sweep_guard
from cargo_target_lease import _validate_directory_chain

ROOT = Path(__file__).resolve().parents[2]
CI = ROOT / "scripts/ci"
CLEAN = CI / "cargo_clean_guard.py"
SWEEP = CI / "cargo_sweep_guard.py"


class CargoTargetLeaseProcessTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="katana-target-lease-")
        self.root = Path(self.temporary.name)
        self.manifest = self.make_crate()
        self.target = self.root / "target"
        self.runtime = lease.lease_path(self.target).parent
        self.environment = os.environ.copy()
        self.environment.update(
            CARGO_TARGET_DIR=str(self.target),
            CARGO_NET_OFFLINE="true",
        )

    def tearDown(self):
        self.temporary.cleanup()

    def test_running_test_holds_next_binary_against_clean_and_sweep(self):
        self.environment["LEASE_TEST_BARRIER"] = str(self.root / "barrier")
        self.environment["LEASE_TEST_SECOND"] = str(self.root / "second-ran")
        runner = self.root / "run-managed-cargo-test.py"
        runner.write_text(
            "import os, subprocess, sys\n"
            f"sys.path.insert(0, {str(CI)!r})\n"
            "from pathlib import Path\n"
            "from cargo_target_lease import target_lease\n"
            "target = Path(os.environ['CARGO_TARGET_DIR'])\n"
            "command = ['cargo', 'test', '--manifest-path', "
            f"{str(self.manifest)!r}, '--test', 'a_first', '--test', 'b_second', "
            "'--', '--test-threads=1']\n"
            "with target_lease(target, exclusive=False):\n"
            "    raise SystemExit(subprocess.run(command, env=os.environ.copy()).returncode)\n",
            encoding="utf-8",
        )
        process = subprocess.Popen(
            [sys.executable, str(runner)], env=self.environment,
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
        )
        try:
            self.wait_for(self.root / "barrier", process)
            next_binaries = list((self.target / "debug/deps").glob("b_second-*"))
            self.assertTrue(next_binaries, "Cargo should have built the next test executable")

            clean = self.run_guard(CLEAN)
            self.assertEqual(75, clean.returncode, clean.stdout + clean.stderr)
            sweep = self.run_guard(SWEEP, "--dry-run")
            self.assertEqual(0, sweep.returncode, sweep.stdout + sweep.stderr)
            self.assertIn("Cargo sweep skipped safely", sweep.stdout)
            self.assertTrue(next_binaries[0].is_file())

            (self.root / "release").touch()
            output, _ = process.communicate(timeout=60)
            self.assertEqual(0, process.returncode, output)
            self.assertTrue((self.root / "second-ran").is_file(), output)

            idle_sweep = self.run_guard(SWEEP, "--dry-run")
            self.assertEqual(0, idle_sweep.returncode, idle_sweep.stdout + idle_sweep.stderr)
            if cargo_sweep_guard.fcntl is None or os.name != "posix":
                self.assertIn("Cargo sweep skipped safely", idle_sweep.stdout)
            else:
                self.assertIn("Running cargo-sweep", idle_sweep.stdout)
            self.write_cachedir_tag()
            idle_clean = self.run_guard(CLEAN)
            if cargo_sweep_guard.fcntl is None or os.name != "posix":
                self.assertEqual(75, idle_clean.returncode, idle_clean.stdout + idle_clean.stderr)
                self.assertTrue(self.target.exists())
            else:
                self.assertEqual(0, idle_clean.returncode, idle_clean.stdout + idle_clean.stderr)
                self.assertFalse(self.target.exists())
        finally:
            if process.poll() is None:
                (self.root / "release").touch()
                process.terminate()
                process.communicate(timeout=15)

    def test_failed_or_interrupted_holder_releases_lease(self):
        for exit_code, interrupt in ((23, False), (0, True)):
            with self.subTest(exit_code=exit_code, interrupt=interrupt):
                marker = self.root / f"holder-{exit_code}-{interrupt}"
                code = (
                    "import sys,time\n"
                    f"sys.path.insert(0, {str(CI)!r})\n"
                    "from pathlib import Path\n"
                    "from cargo_target_lease import target_lease\n"
                    f"with target_lease(Path({str(self.target)!r}), exclusive=False, runtime_directory=Path({str(self.runtime)!r})):\n"
                    f" Path({str(marker)!r}).write_text('ready')\n"
                    + (" while True: time.sleep(0.05)\n" if interrupt else " raise SystemExit(23)\n")
                )
                holder = subprocess.Popen([sys.executable, "-c", code])
                self.wait_for(marker, holder)
                if interrupt:
                    holder.send_signal(signal.SIGTERM)
                    holder.wait(timeout=10)
                else:
                    self.assertEqual(23, holder.wait(timeout=10))
                with lease.target_lease(self.target, exclusive=True, nonblocking=True,
                                        runtime_directory=self.runtime):
                    pass

    def test_distinct_targets_are_independent_and_bad_lock_refuses(self):
        with lease.target_lease(self.target, exclusive=False, runtime_directory=self.runtime):
            other = self.root / "other-target"
            code = (
                "import sys\n"
                f"sys.path.insert(0, {str(CI)!r})\n"
                "from pathlib import Path\n"
                "from cargo_target_lease import target_lease\n"
                f"with target_lease(Path({str(other)!r}), exclusive=True, nonblocking=True, runtime_directory=Path({str(self.runtime)!r})):\n"
                " pass\n"
            )
            result = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True)
            self.assertEqual(0, result.returncode, result.stdout + result.stderr)

        malformed_runtime = self.root / "malformed-runtime"
        malformed_runtime.mkdir(mode=0o700)
        lock = lease.lease_path(self.target, malformed_runtime)
        outside = self.root / "outside-lock"
        outside.write_text("untouched")
        lock.symlink_to(outside)
        with self.assertRaises(lease.LeaseUnavailable):
            with lease.target_lease(self.target, exclusive=True,
                                    runtime_directory=malformed_runtime):
                self.fail("symlink lock must not be accepted")
        self.assertEqual("untouched", outside.read_text())

        nonregular_runtime = self.root / "nonregular-runtime"
        nonregular_runtime.mkdir(mode=0o700)
        lease.lease_path(self.target, nonregular_runtime).mkdir()
        with self.assertRaises(lease.LeaseUnavailable):
            with lease.target_lease(self.target, exclusive=True,
                                    runtime_directory=nonregular_runtime):
                self.fail("directory lock must not be accepted")

        if os.name != "nt":
            unsafe_lock_runtime = self.root / "unsafe-lock-runtime"
            unsafe_lock_runtime.mkdir(mode=0o700)
            unsafe_lock = lease.lease_path(self.target, unsafe_lock_runtime)
            unsafe_lock.write_text("", encoding="ascii")
            unsafe_lock.chmod(0o644)
            with self.assertRaisesRegex(lease.LeaseUnavailable, "owner or mode"):
                with lease.target_lease(self.target, exclusive=True,
                                        runtime_directory=unsafe_lock_runtime):
                    self.fail("group-readable lock must not be accepted")

            inside_runtime = self.target / "private-runtime"
            inside_runtime.mkdir(parents=True, mode=0o700)
            with self.assertRaisesRegex(lease.LeaseUnavailable, "outside the Cargo target"):
                lease.lease_path(self.target, inside_runtime)

        if os.name != "nt":
            unsafe_runtime = self.root / "unsafe-runtime"
            unsafe_runtime.mkdir(mode=0o755)
            with self.assertRaisesRegex(lease.LeaseUnavailable, "mode 0700"):
                lease.lease_path(self.target, unsafe_runtime)

    def test_runtime_environment_differences_share_identity(self):
        with lease.target_lease(self.target, exclusive=False):
            alternate_environment = os.environ.copy()
            alternate_environment["XDG_RUNTIME_DIR"] = str(self.root / "alternate-runtime")
            alternate_environment["TMPDIR"] = str(self.root / "alternate-temp")
            code = (
                "import sys\n"
                f"sys.path.insert(0, {str(CI)!r})\n"
                "from pathlib import Path\n"
                "from cargo_target_lease import LeaseBusy, target_lease\n"
                "try:\n"
                f"    with target_lease(Path({str(self.target)!r}), exclusive=True, nonblocking=True):\n"
                "        raise SystemExit('environment changed the lease identity')\n"
                "except LeaseBusy:\n"
                "    print('same lease')\n"
            )
            result = subprocess.run([sys.executable, "-c", code], env=alternate_environment,
                                    capture_output=True, text=True)
            self.assertEqual(0, result.returncode, result.stdout + result.stderr)
            self.assertIn("same lease", result.stdout)

    def test_original_runtime_components_reject_posix_world_writable_and_symlink_paths(self):
        home = self.root / "runtime-home"
        home.mkdir(mode=0o700)
        cache = home / ".cache"
        cache.mkdir(mode=0o700)
        (cache / "katana").mkdir(mode=0o700)
        (cache / "katana/target-leases").mkdir(mode=0o700)
        if os.name != "nt":
            cache.chmod(0o777)
            with self.assertRaisesRegex(lease.LeaseUnavailable, "writable by others"):
                _validate_directory_chain(cache / "katana/target-leases", home)

            cache.chmod(0o700)
        (cache / "katana/target-leases").rmdir()
        (cache / "katana").rmdir()
        private = home / "private-runtime"
        private.mkdir(mode=0o700)
        (private / "target-leases").mkdir(mode=0o700)
        (cache / "katana").symlink_to(private, target_is_directory=True)
        with self.assertRaisesRegex(lease.LeaseUnavailable, "symlink or non-directory"):
            _validate_directory_chain(cache / "katana/target-leases", home)

    def make_crate(self) -> Path:
        (self.root / "Cargo.toml").write_text(
            '[package]\nname = "lease-fixture"\nversion = "0.1.0"\nedition = "2021"\n',
            encoding="utf-8",
        )
        (self.root / "Cargo.lock").write_text(
            'version = 3\n\n[[package]]\nname = "lease-fixture"\nversion = "0.1.0"\n',
            encoding="utf-8",
        )
        tests = self.root / "tests"
        tests.mkdir()
        (tests / "a_first.rs").write_text(
            "use std::{env, fs, thread, time::Duration};\n"
            "#[test] fn waits_for_release() {\n"
            " let barrier = env::var(\"LEASE_TEST_BARRIER\").unwrap();\n"
            " fs::write(&barrier, \"entered\").unwrap();\n"
            " let release = std::path::PathBuf::from(&barrier).with_file_name(\"release\");\n"
            " while !release.exists() { thread::sleep(Duration::from_millis(10)); }\n"
            "}\n",
            encoding="utf-8",
        )
        (tests / "b_second.rs").write_text(
            "use std::{env, fs};\n"
            "#[test] fn next_binary_runs() { fs::write(env::var(\"LEASE_TEST_SECOND\").unwrap(), \"ran\").unwrap(); }\n",
            encoding="utf-8",
        )
        return self.root / "Cargo.toml"

    def run_guard(self, script: Path, *arguments: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(script), "--manifest-path", str(self.manifest), *arguments],
            env=self.environment, capture_output=True, text=True, timeout=60,
        )

    def write_cachedir_tag(self) -> None:
        self.target.mkdir(exist_ok=True)
        (self.target / "CACHEDIR.TAG").write_text(
            "Signature: 8a477f597d28d172789f06886806bc55\n", encoding="ascii"
        )

    def wait_for(self, marker: Path, process: subprocess.Popen) -> None:
        deadline = time.monotonic() + 60
        while not marker.exists():
            if process.poll() is not None:
                output, _ = process.communicate()
                self.fail(f"process exited before barrier ({process.returncode}): {output}")
            if time.monotonic() >= deadline:
                self.fail("process did not reach barrier")
            time.sleep(0.01)


if __name__ == "__main__":
    unittest.main()
