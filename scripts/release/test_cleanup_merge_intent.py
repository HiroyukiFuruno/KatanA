"""Schema compatibility and validation for merge-bound remote cleanup records."""

import json
from pathlib import Path
import tempfile
import unittest

from cleanup_intent import read_intent, write_intent
from release_cleanup_audit import CleanupRefused, git


REPOSITORY = "HiroyukiFuruno/KatanA"
EXPECTED = "a" * 40
MERGE = "b" * 40


class CleanupMergeIntentTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        base = Path(self.temporary.name)
        self.root, self.private = base / "source", base / "private"
        self.root.mkdir()
        self.private.mkdir(mode=0o700)
        git(self.root, "init", "--initial-branch=master")

    def test_native_schema_one_and_two_records_roundtrip(self) -> None:
        local, local_digest = write_intent(self.private, self.root, "release/v1", EXPECTED)
        self.assertEqual(json.loads(local.read_text())["schema"], 1)
        self.assertEqual(read_intent(local, local_digest)["schema"], 1)
        remote_private = self.root.parent / "remote-private"
        remote_private.mkdir(mode=0o700)
        remote, remote_digest = write_intent(
            remote_private, self.root, "release/v1", EXPECTED,
            operation="remote-delete", repository=REPOSITORY)
        self.assertEqual(json.loads(remote.read_text())["schema"], 2)
        self.assertEqual(read_intent(remote, remote_digest, operation="remote-delete")["schema"], 2)

    def test_schema_three_roundtrip_contains_both_remote_fields(self) -> None:
        path, digest = write_intent(
            self.private, self.root, "release/v1", EXPECTED,
            operation="remote-delete", repository=REPOSITORY,
            merge_commit=MERGE, pull_request="42")
        value = read_intent(path, digest, operation="remote-delete")
        self.assertEqual(value["schema"], 3)
        self.assertEqual(value["merge_commit"], MERGE)
        self.assertEqual(value["pull_request"], "42")

    def test_incomplete_pair_is_rejected_for_remote_and_local(self) -> None:
        for kwargs in ({"merge_commit": MERGE}, {"pull_request": "42"}):
            with self.subTest(kwargs=kwargs), self.assertRaises(CleanupRefused):
                write_intent(self.private, self.root, "release/v1", EXPECTED,
                             operation="remote-delete", repository=REPOSITORY, **kwargs)
        with self.assertRaises(CleanupRefused):
            write_intent(self.private, self.root, "release/v1", EXPECTED,
                         merge_commit=MERGE, pull_request="42")

    def test_invalid_merge_oid_and_pull_request_are_rejected(self) -> None:
        for merge in ("b" * 39, "b" * 41, "B" * 40):
            with self.subTest(merge=merge), self.assertRaises(CleanupRefused):
                write_intent(self.private, self.root, "release/v1", EXPECTED,
                             operation="remote-delete", repository=REPOSITORY,
                             merge_commit=merge, pull_request="42")
        path, digest = write_intent(
            self.private, self.root, "release/v1", EXPECTED,
            operation="remote-delete", repository=REPOSITORY,
            merge_commit="b" * 64, pull_request="42")
        self.assertEqual(read_intent(path, digest, operation="remote-delete")["schema"], 3)
        for number in ("0", "-1", "1.0", "01", "decimal"):
            with self.subTest(number=number), self.assertRaises(CleanupRefused):
                write_intent(self.private, self.root, "release/v1", EXPECTED,
                             operation="remote-delete", repository=REPOSITORY,
                             merge_commit=MERGE, pull_request=number)


if __name__ == "__main__":
    unittest.main()
