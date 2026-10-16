"""Actual Git range selection with explicit new-ref bases and failure boundaries."""

import os
from pathlib import Path
import tempfile
import unittest

from downstream_git_binding import GitBindingError
from push_commit_ranges import (MAX_SELECTED_COMMITS, PushRangeError,
                                branch_commit_selection, parse_branch_commit_listing)
from push_ref_updates import PushRefUpdate
from test_downstream_git_object_format import create_record, git


class CommitListingTests(unittest.TestCase):
    def test_exact_limit_is_accepted(self):
        raw = (("a" * 64 + "\n") * MAX_SELECTED_COMMITS).encode("ascii")
        self.assertEqual(len(parse_branch_commit_listing(raw)), MAX_SELECTED_COMMITS)

    def test_over_limit_is_rejected_before_object_validation(self):
        raw = (("a" * 64 + "\n") * (MAX_SELECTED_COMMITS + 1)).encode("ascii")
        with self.assertRaises(PushRangeError):
            parse_branch_commit_listing(raw)

    def test_invalid_listing_is_rejected(self):
        for raw in (b"\xff\n", b"--help\n", b"a" * 39 + b"\n", b"A" * 64 + b"\n"):
            with self.subTest(raw=raw), self.assertRaises(PushRangeError):
                parse_branch_commit_listing(raw)
        self.assertEqual(parse_branch_commit_listing(b""), ())


class PushCommitRangeTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        document = create_record(self.root, self.environment)
        self.base = document["changes"][0]["commit"]
        (self.root / "source.txt").write_text("first\n", encoding="utf-8")
        git(self.root, self.environment, "add", ".")
        git(self.root, self.environment, "commit", "-qm", "source change (#335)")
        self.head = git(self.root, self.environment, "rev-parse", "HEAD")

    def update(self, head=None, base=None):
        return PushRefUpdate("HEAD", self.head if head is None else head,
                             "refs/heads/release/test", self.base if base is None else base)

    def test_fast_forward_and_unchanged_updates_have_exact_selection(self):
        self.assertEqual(branch_commit_selection(self.root, self.update()), (self.head,))
        self.assertEqual(branch_commit_selection(self.root, self.update(base=self.head)), ())

    def test_new_ref_requires_explicit_actual_base(self):
        update = self.update(base="0" * 64)
        with self.assertRaises(PushRangeError):
            branch_commit_selection(self.root, update)
        self.assertEqual(branch_commit_selection(self.root, update, new_ref_base=self.base), (self.head,))

    def test_grafted_ancestry_cannot_omit_a_real_commit(self):
        (self.root / "source.txt").write_text("second\n", encoding="utf-8")
        git(self.root, self.environment, "add", ".")
        git(self.root, self.environment, "commit", "-qm", "second source change (#335)")
        latest = git(self.root, self.environment, "rev-parse", "HEAD")
        (self.root / ".git" / "info" / "grafts").write_text(
            f"{latest} {self.base}\n", encoding="ascii")
        observed = git(self.root, self.environment, "--no-replace-objects",
                       "-c", "advice.graftFileDeprecated=false", "rev-list", "--reverse",
                       f"{self.base}..{latest}")
        self.assertEqual(observed.splitlines(), [latest])
        self.assertEqual(branch_commit_selection(self.root, self.update(head=latest)),
                         (self.head, latest))

    def test_non_fast_forward_missing_or_abbreviated_objects_are_rejected(self):
        for update in (self.update(head=self.base, base=self.head),
                       self.update(head="f" * 64), self.update(head=self.head[:40])):
            with self.subTest(update=update), self.assertRaises(GitBindingError):
                branch_commit_selection(self.root, update)

    def test_deleted_branch_is_selection_only_not_cleanup_authority(self):
        update = PushRefUpdate("(delete)", "0" * 64, "refs/heads/release/test", self.head)
        self.assertEqual(branch_commit_selection(self.root, update), ())
        missing = PushRefUpdate("(delete)", "0" * 64, "refs/heads/release/test", "f" * 64)
        with self.assertRaises(GitBindingError):
            branch_commit_selection(self.root, missing)

    def test_tags_and_unsafe_raw_oids_are_not_branch_admission(self):
        tag = PushRefUpdate("refs/tags/v1", self.head, "refs/tags/v1", self.base)
        with self.assertRaises(PushRangeError):
            branch_commit_selection(self.root, tag)
        with self.assertRaises(PushRangeError):
            branch_commit_selection(self.root, self.update(head="--help"))

    def test_shallow_merge_history_cannot_admit_an_incomplete_range(self):
        tree = git(self.root, self.environment, "rev-parse", "HEAD^{tree}")
        latest = git(self.root, self.environment, "commit-tree", tree, "-p", self.head,
                     "-m", "second source change (#335)")
        alternate = git(self.root, self.environment, "commit-tree", tree, "-p", self.base,
                        "-m", "alternate source change (#335)")
        merged = git(self.root, self.environment, "commit-tree", tree, "-p", latest,
                     "-p", alternate, "-m", "merge source changes (#335)")
        (self.root / ".git" / "shallow").write_text(f"{latest}\n", encoding="ascii")
        git(self.root, self.environment, "merge-base", "--is-ancestor", self.base, merged)
        observed = git(self.root, self.environment, "rev-list", f"{self.base}..{merged}")
        self.assertNotIn(self.head, observed.splitlines())
        self.assertIn(latest, observed.splitlines())
        with self.assertRaises(PushRangeError):
            branch_commit_selection(self.root, self.update(head=merged))


if __name__ == "__main__":
    unittest.main()
