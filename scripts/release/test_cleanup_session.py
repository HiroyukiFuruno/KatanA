"""Native private-record lifetime and repository-wide cleanup serialization."""

from pathlib import Path
import os
import subprocess
import sys
import tempfile
import unittest

from cleanup_intent import read_intent
from cleanup_session import CleanupSession
from release_cleanup_audit import CleanupRefused, common_directory, git

CHILD_TIMEOUT_SECONDS = 30


class CleanupSessionTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        git(self.root, "init", "--initial-branch=master")
        self.expected = "a" * 40

    def test_record_exists_only_inside_its_single_operation(self):
        with CleanupSession(self.root) as session:
            with session.intent("release/fixture", self.expected) as (path, digest):
                self.assertEqual(read_intent(path, digest)["expected"], self.expected)
                self.assertTrue(path.is_relative_to(common_directory(self.root)))
            self.assertFalse(path.exists())
            with self.assertRaises(CleanupRefused):
                read_intent(path, digest)

    def test_failure_invalidates_record_and_releases_repository_lock(self):
        with self.assertRaisesRegex(RuntimeError, "operation failed"):
            with CleanupSession(self.root) as session:
                with session.intent("release/fixture", self.expected) as (path, digest):
                    raise RuntimeError("operation failed")
        self.assertFalse(path.exists())
        with CleanupSession(self.root):
            pass

    def test_concurrent_repository_session_is_refused_without_stealing_lock(self):
        with CleanupSession(self.root):
            with self.assertRaises(CleanupRefused):
                with CleanupSession(self.root):
                    self.fail("second cleanup acquired an active lock")
            self.assertTrue((common_directory(self.root) / "katana-cleanup.lock").is_dir())
        with CleanupSession(self.root):
            pass

    def test_nested_operation_is_refused_and_original_record_remains_valid(self):
        with CleanupSession(self.root) as session:
            with session.intent("release/fixture", self.expected) as (path, digest):
                with self.assertRaises(CleanupRefused):
                    with session.intent("release/other", self.expected):
                        self.fail("nested intent was admitted")
                self.assertEqual(read_intent(path, digest)["branch"], "release/fixture")

    def test_inactive_session_cannot_create_intent(self):
        session = CleanupSession(self.root)
        with self.assertRaises(CleanupRefused):
            with session.intent("release/fixture", self.expected):
                self.fail("inactive session created a record")

    def test_other_process_cannot_acquire_active_repository_lock(self):
        program = (
            "from pathlib import Path\n"
            "from cleanup_session import CleanupSession\n"
            "from release_cleanup_audit import CleanupRefused\n"
            "import sys\n"
            "try:\n"
            "    with CleanupSession(Path(sys.argv[1])): sys.exit(3)\n"
            "except CleanupRefused: sys.exit(0)\n"
        )
        environment = {key: value for key, value in os.environ.items()
                       if not key.startswith("GIT_")}
        with CleanupSession(self.root):
            result = subprocess.run([sys.executable, "-B", "-c", program, str(self.root)],
                                    cwd=Path(__file__).parent, env=environment,
                                    capture_output=True, timeout=CHILD_TIMEOUT_SECONDS)
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_unknown_lock_contents_are_retained_not_recursively_removed(self):
        session = CleanupSession(self.root)
        with self.assertRaises(CleanupRefused):
            with session:
                unknown = session.lock / "unknown"
                unknown.write_text("keep")
        self.assertEqual(unknown.read_text(), "keep")
        with self.assertRaises(CleanupRefused):
            with CleanupSession(self.root):
                self.fail("uncertain lock was silently stolen")

    def test_remote_record_expires_and_each_operation_gets_a_new_nonce(self):
        with CleanupSession(self.root) as session:
            with session.intent("release/fixture", self.expected, operation="remote-delete",
                                repository="HiroyukiFuruno/KatanA") as (first, digest):
                nonce = read_intent(first, digest, operation="remote-delete")["nonce"]
            with session.intent("release/fixture", self.expected, operation="remote-delete",
                                repository="HiroyukiFuruno/KatanA") as (second, digest):
                self.assertNotEqual(first, second)
                self.assertNotEqual(nonce, read_intent(second, digest, operation="remote-delete")["nonce"])
            self.assertFalse(first.exists())
            self.assertFalse(second.exists())

    def test_unknown_replacement_record_is_retained(self):
        session = CleanupSession(self.root)
        with self.assertRaises(CleanupRefused):
            with session:
                with session.intent("release/fixture", self.expected) as (path, digest):
                    replacement = path.with_name("replacement")
                    replacement.write_text("unknown replacement")
                    os.replace(replacement, path)
        self.assertEqual(path.read_text(), "unknown replacement")
        self.assertTrue(session.lock.exists())

    def test_in_place_modified_record_is_retained(self):
        session = CleanupSession(self.root)
        with self.assertRaises(CleanupRefused):
            with session:
                with session.intent("release/fixture", self.expected) as (path, digest):
                    path.write_text("unknown content")
        self.assertEqual(path.read_text(), "unknown content")
        self.assertTrue(session.lock.exists())


if __name__ == "__main__":
    unittest.main()
