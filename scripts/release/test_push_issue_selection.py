"""Real Git selection contracts, not mocked live publication/admission proof."""

import os
from pathlib import Path
import tempfile
import unittest

from push_issue_selection import (
    MAX_COMMIT_OBJECT_BYTES, PushIssueSelectionError, select_commit_issue_references,
)
from push_ref_updates import PushRefUpdate
from test_downstream_git_object_format import create_record, git


REPOSITORY = "HiroyukiFuruno/KatanA"


class PushIssueSelectionTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.environment = {key: value for key, value in os.environ.items()
                            if not key.startswith("GIT_")}
        self.base = create_record(self.root, self.environment)["changes"][0]["commit"]
        self.head = self.commit("source change (#335)")

    def commit(self, message):
        tree = git(self.root, self.environment, "rev-parse", "HEAD^{tree}")
        message_path = self.root / "message.txt"
        message_path.write_text(message, encoding="utf-8")
        commit = git(self.root, self.environment, "commit-tree", tree, "-p",
                     git(self.root, self.environment, "rev-parse", "HEAD"),
                     "-F", str(message_path))
        git(self.root, self.environment, "update-ref", "HEAD", commit)
        return commit

    def update(self, destination="refs/heads/release/test", *, head=None, base=None):
        return PushRefUpdate("HEAD", head or self.head, destination, base or self.base)

    def select(self, updates, *, new_ref_base=None):
        return select_commit_issue_references(self.root, updates, REPOSITORY,
                                             default_ref="refs/heads/master",
                                             new_ref_base=new_ref_base)

    def test_actual_commit_message_not_working_file_is_selected(self):
        (self.root / "message.txt").write_text("uncommitted (#999)", encoding="utf-8")
        result = self.select((self.update(),))
        self.assertEqual([(item.commit, item.numbers) for item in result.commits],
                         [(self.head, (335,))])

    def test_multiple_branch_refs_deduplicate_shared_commit(self):
        result = self.select((self.update(), self.update("refs/heads/release/other")))
        self.assertEqual(len(result.commits), 1)
        self.assertEqual(dict(result.by_branch), {"refs/heads/release/test": (self.head,),
                                                "refs/heads/release/other": (self.head,)})

    def test_missing_foreign_and_pr_url_references_are_rejected(self):
        for message in ("no Issue", "https://github.com/other/project/issues/335",
                        "https://github.com/HiroyukiFuruno/KatanA/pull/335"):
            head = self.commit(message)
            with self.subTest(message=message), self.assertRaises(PushIssueSelectionError):
                self.select((self.update(head=head),))

    def test_prior_unreferenced_commit_cannot_hide_behind_latest_valid_commit(self):
        self.commit("missing Issue")
        head = self.commit("latest source (#335)")
        with self.assertRaises(PushIssueSelectionError):
            self.select((self.update(head=head),))

    def test_empty_default_tags_and_deletion_do_not_prove_branch_admission(self):
        deletion = PushRefUpdate("(delete)", "0" * len(self.head),
                                 "refs/heads/release/old", self.head)
        result = self.select((self.update("refs/heads/master"),
                              self.update("refs/tags/v1"), deletion))
        self.assertEqual(result.commits, ())
        self.assertEqual(len(result.outside_branch_scope), 3)
        self.assertEqual(self.select(()).commits, ())

    def test_new_branch_requires_explicit_base(self):
        update = self.update(base="0" * len(self.head))
        with self.assertRaises(ValueError):
            self.select((update,))
        self.assertEqual(self.select((update,), new_ref_base=self.base).commits[0].commit,
                         self.head)

    def test_large_commit_object_is_rejected_before_body_parsing(self):
        head = self.commit("x" * MAX_COMMIT_OBJECT_BYTES + " (#335)")
        with self.assertRaises(PushIssueSelectionError):
            self.select((self.update(head=head),))

    def test_excessive_numeric_reference_is_typed_failure(self):
        head = self.commit("source (#" + "9" * 5000 + ")")
        with self.assertRaises(PushIssueSelectionError):
            self.select((self.update(head=head),))


if __name__ == "__main__":
    unittest.main()
