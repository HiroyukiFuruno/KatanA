"""Real remote tag graphs; no GitHub publication or cleanup completion claim."""

from pathlib import Path
import tempfile
import unittest

from release_cleanup_audit import CleanupRefused, git
from release_cleanup_tag import advertised_tag, require_published_commit


class CleanupPublishedTagTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name) / "source"
        self.remote = Path(self.temporary.name) / "remote.git"
        self.root.mkdir()
        git(self.root, "init", "--initial-branch=master")
        git(self.root, "config", "user.email", "cleanup@example.invalid")
        git(self.root, "config", "user.name", "Cleanup fixture")
        git(self.root, "config", "commit.gpgsign", "false")
        git(self.root, "config", "tag.gpgsign", "false")
        git(self.root, "commit", "--allow-empty", "-m", "initial")
        self.initial = git(self.root, "rev-parse", "HEAD").decode().strip()
        git(self.root, "init", "--bare", str(self.remote))
        git(self.root, "remote", "add", "origin", str(self.remote))
        git(self.root, "commit", "--allow-empty", "-m", "candidate")
        self.candidate = git(self.root, "rev-parse", "HEAD").decode().strip()

    def publish_tag(self, name, *arguments):
        git(self.root, "tag", *arguments, name)
        git(self.root, "push", "origin", "refs/tags/" + name)

    def retained_state(self):
        return tuple(git(self.root, *command) for command in (
            ("rev-parse", "HEAD"), ("show-ref",), ("status", "--porcelain=v1"),
        ))

    def test_lightweight_published_tag_contains_candidate(self):
        self.publish_tag("v1.0.0")
        before = self.retained_state()
        actual = require_published_commit(self.root, "origin", "v1.0.0", self.candidate)
        self.assertEqual(actual, self.candidate)
        self.assertEqual(self.retained_state(), before)
        self.assertFalse((self.root / ".git/FETCH_HEAD").exists())

    def test_annotated_tag_peels_to_real_commit(self):
        self.publish_tag("v1.0.0", "-a", "-m", "published")
        advertised, peeled = advertised_tag(self.root, "origin", "v1.0.0")
        self.assertNotEqual(advertised, peeled)
        self.assertEqual(peeled, self.candidate)
        self.assertEqual(require_published_commit(self.root, "origin", "v1.0.0", self.initial), peeled)

    def test_older_release_does_not_admit_newer_candidate(self):
        git(self.root, "tag", "v1.0.0", self.initial)
        git(self.root, "push", "origin", "refs/tags/v1.0.0")
        before = self.retained_state()
        with self.assertRaisesRegex(CleanupRefused, "not included"):
            require_published_commit(self.root, "origin", "v1.0.0", self.candidate)
        self.assertEqual(self.retained_state(), before)

    def test_missing_remote_tag_preserves_local_state(self):
        before = self.retained_state()
        with self.assertRaises(CleanupRefused):
            require_published_commit(self.root, "origin", "v9.9.9", self.candidate)
        self.assertEqual(self.retained_state(), before)

    def test_tree_tag_does_not_admit_cleanup(self):
        tree = git(self.root, "rev-parse", "HEAD^{tree}").decode().strip()
        git(self.root, "tag", "v1.0.0", tree)
        git(self.root, "push", "origin", "refs/tags/v1.0.0")
        before = self.retained_state()
        with self.assertRaises(CleanupRefused):
            require_published_commit(self.root, "origin", "v1.0.0", self.candidate)
        self.assertEqual(self.retained_state(), before)

    def test_non_full_candidate_identity_is_rejected(self):
        for value in ("HEAD", self.candidate[:8], "--all", "0" * 39):
            with self.subTest(value=value), self.assertRaises(CleanupRefused):
                require_published_commit(self.root, "origin", "v1.0.0", value)

    def test_fresh_consumer_fetches_tag_objects_without_creating_refs(self):
        self.publish_tag("v1.0.0", "-a", "-m", "published")
        consumer = Path(self.temporary.name) / "consumer"
        consumer.mkdir()
        git(consumer, "init", "--initial-branch=master")
        with self.assertRaises(CleanupRefused):
            git(consumer, "cat-file", "-t", self.candidate)
        actual = require_published_commit(consumer, str(self.remote), "v1.0.0", self.candidate)
        self.assertEqual(actual, self.candidate)
        self.assertEqual(git(consumer, "for-each-ref", "--format=%(refname)"), b"")
        self.assertFalse((consumer / ".git/FETCH_HEAD").exists())

    def test_existing_fetch_head_is_preserved(self):
        self.publish_tag("v1.0.0")
        git(self.root, "fetch", "origin", "refs/tags/v1.0.0")
        path = self.root / ".git/FETCH_HEAD"
        before = path.read_bytes()
        self.assertEqual(require_published_commit(self.root, "origin", "v1.0.0", self.candidate),
                         self.candidate)
        self.assertEqual(path.read_bytes(), before)

    def test_prune_configuration_cannot_remove_local_tags_with_literal_remote(self):
        self.verify_local_tag_retention(str(self.remote))

    def test_prune_configuration_cannot_remove_local_tags_with_named_remote(self):
        self.verify_local_tag_retention("origin")

    def verify_local_tag_retention(self, destination):
        self.publish_tag("v1.0.0")
        git(self.root, "tag", "local-only", self.initial)
        git(self.root, "config", "fetch.prune", "true")
        git(self.root, "config", "fetch.pruneTags", "true")
        before = self.retained_state()
        self.assertEqual(require_published_commit(self.root, destination, "v1.0.0", self.candidate),
                         self.candidate)
        self.assertEqual(self.retained_state(), before)

    def test_tag_fetch_does_not_visit_populated_submodule_remote(self):
        git(self.root, "-c", "protocol.file.allow=always", "submodule", "add", str(self.root), "module")
        git(self.root, "commit", "-m", "populated module")
        candidate = git(self.root, "rev-parse", "HEAD").decode().strip()
        self.publish_tag("v1.0.0")
        module = self.root / "module"
        git(module, "remote", "set-url", "origin", str(Path(self.temporary.name) / "missing.git"))
        git(self.root, "config", "fetch.recurseSubmodules", "true")
        before = git(module, "rev-parse", "HEAD")
        self.assertEqual(require_published_commit(self.root, str(self.remote), "v1.0.0", candidate),
                         candidate)
        self.assertEqual(git(module, "rev-parse", "HEAD"), before)

    def test_sha256_full_identity_passes_and_sha1_length_prefix_is_rejected(self):
        root, remote, candidate = self.sha256_fixture()
        self.assertEqual(len(candidate), 64)
        self.assertEqual(require_published_commit(root, str(remote), "v1.0.0", candidate), candidate)
        before = git(root, "show-ref")
        with self.assertRaises(CleanupRefused):
            require_published_commit(root, str(remote), "v1.0.0", candidate[:40])
        self.assertEqual(git(root, "show-ref"), before)

    def sha256_fixture(self):
        root = Path(self.temporary.name) / "sha256"
        remote = Path(self.temporary.name) / "sha256.git"
        root.mkdir()
        git(root, "init", "--object-format=sha256", "--initial-branch=master")
        git(root, "config", "user.email", "cleanup@example.invalid")
        git(root, "config", "user.name", "Cleanup fixture")
        git(root, "config", "commit.gpgsign", "false")
        git(root, "config", "tag.gpgsign", "false")
        git(root, "commit", "--allow-empty", "-m", "sha256 candidate")
        git(root, "init", "--bare", "--object-format=sha256", str(remote))
        git(root, "tag", "v1.0.0")
        git(root, "push", str(remote), "refs/tags/v1.0.0")
        return root, remote, git(root, "rev-parse", "HEAD").decode().strip()


if __name__ == "__main__":
    unittest.main()
