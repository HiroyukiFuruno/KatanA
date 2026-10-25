"""Real Git and Issue-payload binding without fake transport or publication."""

import copy
import json
import os
from pathlib import Path
import tempfile
import unittest

from downstream_git_binding import GitBindingError
from downstream_issue_evidence import MARKER
from issue_commit_binding import IssueCommitBindingError, bind_issue_commit
from test_downstream_evidence_fields import REPOSITORY
from test_downstream_git_object_format import create_record, git


class IssueCommitBindingTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        self.document = create_record(self.root, self.environment)
        self.commit = self.document["changes"][0]["commit"]
        self.issue_metadata = {"number": 335, "state": "open",
                        "repository_url": f"https://api.github.com/repos/{REPOSITORY}",
                        "html_url": f"https://github.com/{REPOSITORY}/issues/335"}

    def payload(self, document=None):
        payload = copy.deepcopy(self.issue_metadata)
        payload["body"] = f"{MARKER}\n```json\n{json.dumps(document or self.document)}\n```\n"
        return payload

    def bind(self, payload, commit=None):
        return bind_issue_commit(payload, REPOSITORY, 335, self.root,
                                 self.commit if commit is None else commit)

    def test_declared_current_commit_binds_to_real_manifest_and_lock_blobs(self):
        self.assertEqual(self.bind(self.payload()), self.document)

    def test_historical_evidence_cannot_stand_in_for_the_expected_commit(self):
        (self.root / "source.txt").write_text("next\n", encoding="utf-8")
        git(self.root, self.environment, "add", ".")
        git(self.root, self.environment, "commit", "-qm", "next source (#335)")
        head = git(self.root, self.environment, "rev-parse", "HEAD")
        with self.assertRaises(IssueCommitBindingError):
            self.bind(self.payload(), head)

    def test_body_without_evidence_is_unclassified_not_an_exemption(self):
        for body in (None, "", "normal Issue"):
            payload = self.payload()
            payload["body"] = body
            with self.subTest(body=body):
                self.assertIsNone(self.bind(payload))

    def test_non_issue_wrong_identity_and_missing_body_are_rejected(self):
        changes = (("pull_request", {}), ("number", 346), ("repository_url", "other"),
                   ("body", []))
        for key, value in changes:
            payload = self.payload()
            payload[key] = value
            with self.subTest(key=key), self.assertRaises(IssueCommitBindingError):
                self.bind(payload)
        with self.assertRaises(IssueCommitBindingError):
            self.bind(self.issue_metadata)

    def test_bad_expected_source_or_declared_blob_is_rejected(self):
        for commit in (self.commit[:40], "f" * 64, "--help"):
            with self.subTest(commit=commit), self.assertRaises(GitBindingError):
                self.bind(self.payload(), commit)
        document = copy.deepcopy(self.document)
        document["changes"][0]["manifests"][0]["blob"] = "f" * 64
        with self.assertRaises(GitBindingError):
            self.bind(self.payload(document))

    def test_nonpositive_expected_issue_numbers_are_rejected(self):
        for number in (0, -1):
            payload = self.payload()
            payload["number"] = number
            payload["html_url"] = f"https://github.com/{REPOSITORY}/issues/{number}"
            payload["body"] = None
            with self.subTest(number=number), self.assertRaises(IssueCommitBindingError):
                bind_issue_commit(payload, REPOSITORY, number, self.root, self.commit)
