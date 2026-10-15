"""Native Git/private-record boundary, not public-release authorization."""

from pathlib import Path
import os
import subprocess
import sys
import tempfile
import unittest

from cleanup_intent import write_intent
from cleanup_remote_push import RemoteCleanupPush
from release_cleanup_audit import CleanupRefused, git


REPOSITORY = "HiroyukiFuruno/KatanA"
URL = "https://github.com/HiroyukiFuruno/KatanA.git"
BRANCH = "release/fixture"


class RemotePushBoundaryTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        parent = Path(self.temporary.name)
        self.root = parent / "client"
        self.root.mkdir()
        git(self.root, "init", "--initial-branch=master")
        for key, value in (("user.name", "Fixture"), ("user.email", "fixture@example.invalid"),
                           ("commit.gpgsign", "false")):
            git(self.root, "config", key, value)
        git(self.root, "commit", "--allow-empty", "-m", "initial")
        self.oid = git(self.root, "rev-parse", "HEAD").decode().strip()
        git(self.root, "branch", BRANCH)
        git(self.root, "remote", "add", "origin", URL)
        private = parent / "private"
        private.mkdir(mode=0o700)
        self.path, self.digest = write_intent(private, self.root, BRANCH, self.oid,
                                              operation="remote-delete", repository=REPOSITORY)
        self.request = RemoteCleanupPush(self.root, REPOSITORY, "refs/heads/master", "origin", URL)
        self.payload = f"(delete) {'0' * len(self.oid)} refs/heads/{BRANCH} {self.oid}\n".encode()

    def verify(self, payload=None, *, request=None):
        (request or self.request).verify(self.payload if payload is None else payload,
                                         self.path, self.digest)

    def test_exact_advertised_deletion_is_admitted_without_mutation(self):
        self.verify()
        self.assertEqual(git(self.root, "rev-parse", "refs/heads/" + BRANCH).decode().strip(), self.oid)

    def test_remote_only_is_admitted_without_creating_a_local_branch(self):
        git(self.root, "branch", "-d", BRANCH)
        self.verify()
        self.assertEqual(git(self.root, "for-each-ref", "--format=%(refname)", "refs/heads/" + BRANCH), b"")

    def test_no_intent_is_noop_but_partial_intent_is_refused(self):
        self.request.verify(b"ordinary push", None, None)
        for path, digest in ((self.path, None), (None, self.digest)):
            with self.subTest(path=path), self.assertRaises(CleanupRefused):
                self.request.verify(self.payload, path, digest)

    def test_changed_oid_ref_multiple_and_nondeletion_are_refused(self):
        for payload in (self.payload.replace(self.oid.encode(), b"f" * len(self.oid)),
                        self.payload.replace(BRANCH.encode(), b"release/other"),
                        self.payload + self.payload, b"", self.payload.replace(b"(delete)", b"HEAD")):
            with self.subTest(payload=payload), self.assertRaises(CleanupRefused):
                self.verify(payload)

    def test_wrong_repository_destination_and_default_are_refused(self):
        cases = ((REPOSITORY, "refs/heads/master", "other", URL),
                 ("other/Repo", "refs/heads/master", "origin", URL),
                 (REPOSITORY, "refs/heads/master", "origin", "https://github.com/other/Repo.git"),
                 (REPOSITORY, "refs/heads/" + BRANCH, "origin", URL))
        for repository, default, remote, url in cases:
            request = RemoteCleanupPush(self.root, repository, default, remote, url)
            with self.subTest(repository=repository, default=default), self.assertRaises(CleanupRefused):
                self.verify(request=request)

    def test_dirty_controller_is_retained(self):
        (self.root / "keep.txt").write_text("user data")
        with self.assertRaises(CleanupRefused):
            self.verify()
        self.assertEqual((self.root / "keep.txt").read_text(), "user data")

    def test_changed_local_branch_is_retained(self):
        git(self.root, "commit", "--allow-empty", "-m", "advance")
        changed = git(self.root, "rev-parse", "HEAD").decode().strip()
        git(self.root, "update-ref", "refs/heads/" + BRANCH, changed, self.oid)
        with self.assertRaises(CleanupRefused):
            self.verify()
        self.assertEqual(git(self.root, "rev-parse", "refs/heads/" + BRANCH).decode().strip(), changed)

    def test_empty_configured_remote_cleanup_is_rejected_before_admission(self):
        checker = Path(__file__).with_name("check-push-dependency-evidence.py")
        environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        environment.update(KATANA_REMOTE_CLEANUP_INTENT=str(self.path),
                           KATANA_REMOTE_CLEANUP_INTENT_SHA256=self.digest)
        result = subprocess.run(
            [sys.executable, "-B", str(checker), "--repository", REPOSITORY,
             "--root", str(self.root), "origin", URL],
            input=b"", capture_output=True, env=environment, check=False,
        )
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn(b"remote cleanup requires a native deletion", result.stderr)


if __name__ == "__main__":
    unittest.main()
