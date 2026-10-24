"""Real Git/Just process tests for receipt binding, not production quality results."""

import json
import os
from pathlib import Path
import shlex
import signal
import subprocess
import sys
import tempfile
import time
import unittest

from owner_verification import RECEIPT_NAME, VerificationSession, source_revision
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "ci"))
from cargo_target_lease import LeaseBusy, target_lease

RUNNER = Path(__file__).resolve().parent / "run-owned-push-check.py"


class OwnerVerificationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name).resolve()
        self.environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        subprocess.run(["git", "init", "-q", str(self.root)], check=True, env=self.environment)

    def tearDown(self):
        self.temporary.cleanup()

    def fixture(self, recipe: str):
        (self.root / "Justfile").write_text("check:\n    " + recipe + "\n")
        (self.root / "tracked.txt").write_text("unchanged\n")
        subprocess.run(["git", "add", "Justfile", "tracked.txt"], cwd=self.root, check=True, env=self.environment)
        subprocess.run(["git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-qm", "fixture"],
                       cwd=self.root, check=True, env=self.environment)
        return subprocess.run(["git", "rev-parse", "HEAD"], cwd=self.root, check=True,
                              capture_output=True, text=True, env=self.environment).stdout.strip()

    def run_owner(self, session):
        environment = session.environment()
        environment = {key: value for key, value in environment.items() if not key.startswith("GIT_")}
        return subprocess.run([sys.executable, str(RUNNER)], cwd=self.root, env=environment,
                              check=False, capture_output=True, text=True)

    def test_fixed_owner_command_issues_one_matching_receipt(self):
        revision = self.fixture("printf 'fixed check executed\\n'")
        with VerificationSession(self.root, revision) as session:
            result = self.run_owner(session)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("fixed check executed", result.stdout)
            self.assertEqual(session.verified_source(), revision)
            second = self.run_owner(session)
            self.assertEqual(second.returncode, 2)

    def test_skipped_and_failed_command_never_provide_a_receipt(self):
        revision = self.fixture("exit 7")
        with VerificationSession(self.root, revision) as session:
            self.assertIsNone(session.verified_source())
            result = self.run_owner(session)
            self.assertNotEqual(result.returncode, 0)
            self.assertIsNone(session.verified_source())

    def test_source_changes_during_owner_check_are_rejected(self):
        revision = self.fixture("printf changed >> tracked.txt")
        with VerificationSession(self.root, revision) as session:
            result = self.run_owner(session)
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertIn("clean tracked source", result.stderr)
            self.assertIsNone(session.verified_source())

    def test_another_session_receipt_cannot_be_reused(self):
        revision = self.fixture("printf checked")
        with VerificationSession(self.root, revision) as first, VerificationSession(self.root, revision) as second:
            self.assertEqual(self.run_owner(first).returncode, 0)
            (second.directory / RECEIPT_NAME).write_bytes((first.directory / RECEIPT_NAME).read_bytes())
            with self.assertRaisesRegex(ValueError, "does not match"):
                second.verified_source()

    def test_author_exit_zero_without_owner_token_is_rejected(self):
        revision = self.fixture("printf checked")
        with VerificationSession(self.root, revision) as session:
            (session.directory / RECEIPT_NAME).write_text(json.dumps({"source": revision, "exitCode": 0}))
            with self.assertRaisesRegex(ValueError, "does not match"):
                session.verified_source()

    def test_cargo_owner_check_holds_shared_target_lease(self):
        (self.root / "Cargo.toml").write_text(
            '[package]\nname = "owner-lease-fixture"\nversion = "0.1.0"\nedition = "2021"\n',
            encoding="utf-8",
        )
        (self.root / "Cargo.lock").write_text(
            'version = 3\n\n[[package]]\nname = "owner-lease-fixture"\nversion = "0.1.0"\n',
            encoding="utf-8",
        )
        (self.root / "src").mkdir()
        (self.root / "src/main.rs").write_text("fn main() {}\n", encoding="utf-8")
        probe = self.root / "lease_probe.py"
        probe.write_text(
            "import sys\n"
            f"sys.path.insert(0, {str(Path(__file__).resolve().parents[1] / 'ci')!r})\n"
            "from pathlib import Path\n"
            "from cargo_target_lease import LeaseBusy, target_lease\n"
            "try:\n"
            "    with target_lease(Path('target'), exclusive=True, nonblocking=True):\n"
            "        raise SystemExit('owner check did not hold a shared target lease')\n"
            "except LeaseBusy:\n"
            "    print('owner lease held')\n",
            encoding="utf-8",
        )
        revision = self.fixture(f"{shlex.quote(sys.executable)} {shlex.quote(str(probe))}")
        with VerificationSession(self.root, revision) as session:
            result = self.run_owner(session)
            self.assertEqual(0, result.returncode, result.stdout + result.stderr)
            self.assertIn("owner lease held", result.stdout)
            self.assertEqual(revision, session.verified_source())

    def test_owner_check_refuses_while_cleanup_lease_is_active(self):
        (self.root / "Cargo.toml").write_text(
            '[package]\nname = "owner-cleanup-lease-fixture"\nversion = "0.1.0"\nedition = "2021"\n',
            encoding="utf-8",
        )
        (self.root / "Cargo.lock").write_text(
            'version = 3\n\n[[package]]\nname = "owner-cleanup-lease-fixture"\nversion = "0.1.0"\n',
            encoding="utf-8",
        )
        (self.root / "src").mkdir()
        (self.root / "src/main.rs").write_text("fn main() {}\n", encoding="utf-8")
        marker = self.root / "check-started"
        revision = self.fixture(f"touch {shlex.quote(str(marker))}")
        with VerificationSession(self.root, revision) as session:
            with target_lease(self.root / "target", exclusive=True, nonblocking=True):
                result = self.run_owner(session)
            self.assertNotEqual(0, result.returncode)
            self.assertIn("target lease is held", result.stderr)
            self.assertFalse(marker.exists())
            self.assertIsNone(session.verified_source())

    def test_failed_cargo_owner_check_releases_shared_lease(self):
        (self.root / "Cargo.toml").write_text(
            '[package]\nname = "owner-failure-lease-fixture"\nversion = "0.1.0"\nedition = "2021"\n',
            encoding="utf-8",
        )
        (self.root / "Cargo.lock").write_text(
            'version = 3\n\n[[package]]\nname = "owner-failure-lease-fixture"\nversion = "0.1.0"\n',
            encoding="utf-8",
        )
        (self.root / "src").mkdir()
        (self.root / "src/main.rs").write_text("fn main() {}\n", encoding="utf-8")
        revision = self.fixture("exit 7")
        with VerificationSession(self.root, revision) as session:
            result = self.run_owner(session)
            self.assertNotEqual(0, result.returncode)
            with target_lease(self.root / "target", exclusive=True, nonblocking=True):
                pass

    def test_owner_signal_keeps_lease_until_real_check_child_exits(self):
        if os.name != "posix":
            self.skipTest("process-group signal forwarding is validated on POSIX")
        (self.root / "Cargo.toml").write_text(
            '[package]\nname = "owner-signal-lease-fixture"\nversion = "0.1.0"\nedition = "2021"\n',
            encoding="utf-8",
        )
        (self.root / "Cargo.lock").write_text(
            'version = 3\n\n[[package]]\nname = "owner-signal-lease-fixture"\nversion = "0.1.0"\n',
            encoding="utf-8",
        )
        (self.root / "src").mkdir()
        (self.root / "src/main.rs").write_text("fn main() {}\n", encoding="utf-8")

        for label, owner_signal in (("term", signal.SIGTERM), ("interrupt", signal.SIGINT),
                                    ("kill", signal.SIGKILL)):
            with self.subTest(label=label):
                ready = self.root / f"{label}-ready"
                received = self.root / f"{label}-received"
                release = self.root / f"{label}-release"
                finished = self.root / f"{label}-finished"
                child = self.root / f"{label}-check-child.py"
                leader = self.root / f"{label}-check-leader.py"
                child.write_text(
                    "import os, signal, time\n"
                    "from pathlib import Path\n"
                    f"ready=Path({str(ready)!r}); received=Path({str(received)!r}); "
                    f"release=Path({str(release)!r}); finished=Path({str(finished)!r})\n"
                    "def hold(signum, _frame):\n"
                    " received.write_text(str(signum))\n"
                    " while not release.exists(): time.sleep(0.01)\n"
                    "signal.signal(signal.SIGTERM, hold)\n"
                    "signal.signal(signal.SIGINT, hold)\n"
                    "ready.write_text(str(os.getpid()))\n"
                    "while not release.exists(): time.sleep(0.01)\n"
                    "finished.write_text('done')\n",
                    encoding="utf-8",
                )
                leader.write_text(
                    "import subprocess, sys\n"
                    f"raise SystemExit(subprocess.run([sys.executable, {str(child)!r}], "
                    "close_fds=True).returncode)\n",
                    encoding="utf-8",
                )
                owner_script = self.root / f"{label}-owner.py"
                owner_script.write_text(
                    "import sys\n"
                    f"sys.path.insert(0, {str(Path(__file__).resolve().parent)!r})\n"
                    f"sys.path.insert(0, {str(Path(__file__).resolve().parents[1] / 'ci')!r})\n"
                    "from pathlib import Path\n"
                    "import owner_verification as owner\n"
                    f"owner.COMMAND = [sys.executable, {str(leader)!r}]\n"
                    "raise SystemExit(owner.run_owned_check(Path.cwd()))\n",
                    encoding="utf-8",
                )
                environment = self.environment | {"CARGO_TARGET_DIR": str(self.root / "target")}
                owner_process = subprocess.Popen(
                    [sys.executable, str(owner_script)], cwd=self.root, env=environment,
                    start_new_session=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                    text=True,
                )
                try:
                    self.wait_for_path(ready, owner_process)
                    if owner_signal != signal.SIGKILL:
                        owner_process.send_signal(owner_signal)
                        self.wait_for_path(received, owner_process)
                    else:
                        owner_process.kill()
                        owner_process.wait(timeout=10)
                        self.assertEqual(-signal.SIGKILL, owner_process.returncode)

                    with self.assertRaisesRegex(LeaseBusy, "target lease is held"):
                        with target_lease(self.root / "target", exclusive=True,
                                          nonblocking=True):
                            self.fail("live check child must retain its target lease")

                    release.touch()
                    if owner_signal != signal.SIGKILL:
                        output, _ = owner_process.communicate(timeout=15)
                        self.assertEqual(128 + owner_signal, owner_process.returncode, output)
                    self.wait_for_lease_release(self.root / "target")
                    self.assertTrue(finished.is_file())
                    if owner_signal == signal.SIGKILL:
                        output, _ = owner_process.communicate(timeout=10)
                        self.assertEqual(-signal.SIGKILL, owner_process.returncode, output)
                finally:
                    release.touch()
                    if owner_process.poll() is None:
                        owner_process.kill()
                    if owner_process.stdout and not owner_process.stdout.closed:
                        owner_process.communicate(timeout=10)

    def wait_for_path(self, marker: Path, process: subprocess.Popen, timeout: float = 30):
        deadline = time.monotonic() + timeout
        while not marker.exists():
            if process.poll() is not None:
                output, _ = process.communicate()
                self.fail(f"process exited before {marker.name}: {process.returncode} {output}")
            if time.monotonic() >= deadline:
                self.fail(f"timed out waiting for {marker.name}")
            time.sleep(0.01)

    def wait_for_lease_release(self, target: Path):
        deadline = time.monotonic() + 30
        while True:
            try:
                with target_lease(target, exclusive=True, nonblocking=True):
                    return
            except LeaseBusy:
                if time.monotonic() >= deadline:
                    self.fail("check process exited without releasing its target lease")
                time.sleep(0.01)


if __name__ == "__main__":
    unittest.main()
