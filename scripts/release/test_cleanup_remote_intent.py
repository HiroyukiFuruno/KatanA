"""Remote cleanup records cannot be replayed as local cleanup requests."""

from pathlib import Path
import tempfile
import unittest

from cleanup_intent import read_intent, write_intent
from release_cleanup_audit import CleanupRefused, git


REPOSITORY = "HiroyukiFuruno/KatanA"
EXPECTED = "a" * 40


class RemoteIntentTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        git(self.root, "init", "--initial-branch=master")
        self.private = self.root / "private"
        self.private.mkdir(mode=0o700)

    def write_remote(self, **changes):
        options = {"operation": "remote-delete", "repository": REPOSITORY} | changes
        return write_intent(self.private, self.root, "release/fixture", EXPECTED, **options)

    def test_remote_record_retains_repository_and_explicit_operation(self):
        path, digest = self.write_remote()
        value = read_intent(path, digest, operation="remote-delete")
        self.assertEqual(value["operation"], "remote-delete")
        self.assertEqual(value["repository"], REPOSITORY)
        self.assertEqual(value["expected"], EXPECTED)

    def test_remote_record_is_refused_by_local_default_reader(self):
        path, digest = self.write_remote()
        with self.assertRaises(CleanupRefused):
            read_intent(path, digest)

    def test_local_record_is_refused_by_remote_reader(self):
        path, digest = write_intent(self.private, self.root, "release/fixture", EXPECTED)
        with self.assertRaises(CleanupRefused):
            read_intent(path, digest, operation="remote-delete")

    def test_unknown_operation_is_refused_before_creating_record(self):
        with self.assertRaises(CleanupRefused):
            self.write_remote(operation="unknown")
        self.assertFalse((self.private / "request.json").exists())

    def test_invalid_remote_repository_is_refused_before_creating_record(self):
        for repository in (None, "owner//repo", "https://github.com/owner/repo"):
            with self.subTest(repository=repository), self.assertRaises(CleanupRefused):
                self.write_remote(repository=repository)
        self.assertFalse((self.private / "request.json").exists())

    def test_local_writer_does_not_silently_ignore_remote_repository(self):
        with self.assertRaises(CleanupRefused):
            write_intent(self.private, self.root, "release/fixture", EXPECTED,
                         repository=REPOSITORY)
        self.assertFalse((self.private / "request.json").exists())


if __name__ == "__main__":
    unittest.main()
