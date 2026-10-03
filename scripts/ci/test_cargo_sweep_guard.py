import contextlib
import io
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest

import cargo_sweep_guard as guard


class CargoSweepGuardTest(unittest.TestCase):
    def test_repository_sweep_entries_are_dry_run_and_contract_is_wired(self):
        repository = Path(__file__).resolve().parents[2]
        maintenance = (repository / "just/maintenance.just").read_text(encoding="utf-8")
        tests = (repository / "just/tests.just").read_text(encoding="utf-8")
        linux = (repository / "platforms/linux/ci/Dockerfile").read_text(encoding="utf-8")
        windows = (repository / "platforms/windows/ci/Dockerfile").read_text(encoding="utf-8")
        self.assertIn("scripts/ci/cargo_sweep_guard.py --dry-run", maintenance)
        self.assertRegex(tests, r"(?m)^test:.*test-cargo-sweep-contract")
        self.assertIn("test_cargo_sweep_guard.py", tests)
        for label, source in (("just/tests.just", tests), ("linux Dockerfile", linux),
                              ("windows Dockerfile", windows)):
            with self.subTest(entry=label):
                assert_sweep_commands_are_dry_run(self, source)

    def test_macos_setup_installs_the_declared_sweep_tool_with_lockfile(self):
        repository = Path(__file__).resolve().parents[2]
        setup = (repository / "scripts/setup/setup.sh").read_text(encoding="utf-8")
        self.assertRegex(setup, r"(?m)^#   - cargo-sweep\s+: Clean up unused build artifacts$")
        self.assertIn('• cargo-sweep     (just sweep)', setup)
        self.assertIn("# 8c. cargo-sweep", setup)
        self.assertIn("# 8d. git-cliff", setup)
        self.assertIn('if cargo sweep --version &>/dev/null; then', setup)
        self.assertIn("cargo install cargo-sweep --locked", setup)
        self.assertIn('echo "  cargo-sweep  $(cargo sweep --version)"', setup)

    def test_real_build_blocks_sweep_then_sweep_dry_run_succeeds_or_skips_safely(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            manifest = create_crate(root)
            target, barrier, environment = build_environment(root)
            barrier.mkdir()
            create_target_triple_profiles(target)
            process = launch_build(manifest, environment)
            try:
                wait_for_barrier(barrier / "entered", process)
                skipped = run_guard(manifest, environment)
                self.assertEqual(0, skipped.returncode, skipped.stderr)
                self.assertIn("Cargo sweep skipped safely", skipped.stdout)
                self.assertNotIn("Running cargo-sweep", skipped.stdout)
                (barrier / "release").touch()
                output, _ = process.communicate(timeout=60)
                self.assertEqual(0, process.returncode, output)
                built = build_release(manifest, environment)
                self.assertEqual(0, built.returncode, built.stdout + built.stderr)
                swept = run_guard(manifest, environment)
                self.assertEqual(0, swept.returncode, swept.stderr)
                safe_skip = guard.fcntl is None or os.name != "posix"
                if safe_skip:
                    self.assertIn("Cargo sweep skipped safely", swept.stdout)
                    self.assertNotIn("Running cargo-sweep", swept.stdout)
                else:
                    self.assertIn("Running cargo-sweep", swept.stdout)
                for profile in ("debug", "release"):
                    target_triple_profile = (
                        target / "x86_64-unknown-linux-gnu" / profile
                    )
                    if safe_skip:
                        self.assertTrue((target_triple_profile / ".fingerprint").is_dir())
                        self.assertFalse((target_triple_profile / ".cargo-lock").exists())
                    else:
                        self.assertTrue((target / profile / ".cargo-lock").is_file())
                        self.assertTrue((target_triple_profile / ".cargo-lock").is_file())
            finally:
                finally_release(process, barrier)

    def test_non_regular_target_and_ambiguous_profile_fail_closed(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            manifest = create_crate(root)
            target = root / "target"
            linked = root / "linked-target"
            if os.name == "posix":
                linked.symlink_to(target)
            else:
                linked.write_text("not a target directory", encoding="utf-8")
            previous = os.environ.get("CARGO_TARGET_DIR")
            os.environ["CARGO_TARGET_DIR"] = str(linked)
            try:
                with self.assertRaises(guard.CleanupUnavailable):
                    guard.profile_directories(guard.cargo_target_directory(manifest))
            finally:
                restore_environment("CARGO_TARGET_DIR", previous)
            custom = target / "ci" / ".fingerprint"
            custom.mkdir(parents=True)
            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                self.assertEqual(0, guard.run_guard(manifest, dry_run=True))
            self.assertIn("unknown directory in Cargo target", output.getvalue())
            self.assertFalse((target / "debug").exists())
            self.assertFalse((target / "release").exists())

    def test_coverage_target_profiles_are_discovered_and_locked(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            manifest = create_crate(root)
            target = root / "target"
            coverage = target / "llvm-cov-target"
            profiles = [target / "debug", coverage / "debug",
                        coverage / "x86_64-unknown-linux-gnu" / "release"]
            for profile in profiles:
                (profile / ".fingerprint").mkdir(parents=True)
            (coverage / "tmp").mkdir()
            self.assertEqual(sorted(profiles), guard.profile_directories(target))
            if guard.fcntl is not None and os.name == "posix":
                descriptors = guard.acquire_profile_locks([coverage / "debug"])
                try:
                    result = run_guard(manifest, {**os.environ, "CARGO_TARGET_DIR": str(target)})
                    self.assertEqual(0, result.returncode, result.stderr)
                    self.assertIn("profile lock unavailable", result.stdout)
                    self.assertNotIn("Running cargo-sweep", result.stdout)
                finally:
                    guard.release_profile_locks(descriptors)
                descriptors = guard.acquire_profile_locks(guard.profile_directories(target))
                try:
                    self.assertEqual(len(profiles), len(descriptors))
                finally:
                    guard.release_profile_locks(descriptors)

    def test_coverage_target_keeps_unknown_and_unsafe_paths_fail_closed(self):
        for mutation in ("unknown", "profile_file", "fingerprint_file", "symlink"):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as directory:
                target = Path(directory) / "target"
                coverage = target / "llvm-cov-target"
                coverage.mkdir(parents=True)
                if mutation == "unknown":
                    (coverage / "ci").mkdir()
                elif mutation == "profile_file":
                    (coverage / "debug").write_text("not a profile")
                elif mutation == "fingerprint_file":
                    (coverage / "debug").mkdir()
                    (coverage / "debug" / ".fingerprint").write_text("not a directory")
                elif os.name == "posix":
                    coverage.rmdir()
                    coverage.symlink_to(Path(directory))
                else:
                    (coverage / "ci").mkdir()
                with self.assertRaises(guard.CleanupUnavailable):
                    guard.profile_directories(target)


def create_crate(root):
    (root / "src").mkdir()
    (root / "Cargo.toml").write_text(
        '[package]\nname = "sweep-guard-probe"\nversion = "0.1.0"\nedition = "2024"\n'
    )
    (root / "Cargo.lock").write_text(
        'version = 4\n\n[[package]]\nname = "sweep-guard-probe"\nversion = "0.1.0"\n'
    )
    (root / "src" / "lib.rs").write_text("pub fn ready() -> bool { true }\n")
    (root / "build.rs").write_text(
        "use std::{env, fs, thread, time::Duration};\n"
        "fn main() {\n"
        "  let barrier = std::path::PathBuf::from(env::var_os(\"CARGO_SYNC_BARRIER\").unwrap());\n"
        "  fs::write(barrier.join(\"entered\"), b\"ready\").unwrap();\n"
        "  while !barrier.join(\"release\").exists() {\n"
        "    thread::sleep(Duration::from_millis(10));\n"
        "  }\n"
        "}\n"
    )
    return root / "Cargo.toml"


def build_environment(root):
    barrier = root / "barrier"
    environment = os.environ.copy()
    environment.update(
        CARGO_TARGET_DIR=str(root / "target"),
        CARGO_SYNC_BARRIER=str(barrier),
        CARGO_NET_OFFLINE="true",
    )
    return root / "target", barrier, environment


def create_target_triple_profiles(target):
    for profile in ("debug", "release"):
        (target / "x86_64-unknown-linux-gnu" / profile / ".fingerprint").mkdir(parents=True)


def launch_build(manifest, environment):
    return subprocess.Popen(
        ["cargo", "build", "--manifest-path", str(manifest)],
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )


def build_release(manifest, environment):
    return subprocess.run(
        ["cargo", "build", "--release", "--manifest-path", str(manifest)],
        env=environment,
        capture_output=True,
        text=True,
        timeout=60,
    )


def wait_for_barrier(marker, process):
    deadline = time.monotonic() + 60
    while not marker.exists():
        if process.poll() is not None:
            output, _ = process.communicate()
            raise AssertionError(f"build exited before barrier: {output}")
        if time.monotonic() >= deadline:
            raise AssertionError("build script did not reach barrier")
        time.sleep(0.01)


def run_guard(manifest, environment):
    return subprocess.run(
        [sys.executable, str(Path(guard.__file__)), "--manifest-path", str(manifest), "--dry-run"],
        env=environment,
        capture_output=True,
        text=True,
        timeout=60,
    )


def finally_release(process, barrier):
    if barrier.exists():
        (barrier / "release").touch()
    if process.poll() is None:
        process.terminate()
        process.communicate(timeout=15)


def restore_environment(name, value):
    if value is None:
        os.environ.pop(name, None)
    else:
        os.environ[name] = value


def assert_sweep_commands_are_dry_run(test_case, source):
    commands = [line for line in source.splitlines() if "cargo sweep" in line]
    test_case.assertTrue(commands, "no cargo sweep command found")
    for command in commands:
        test_case.assertIn("--dry-run", command)


if __name__ == "__main__":
    unittest.main()
