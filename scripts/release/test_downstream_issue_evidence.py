#!/usr/bin/env python3
"""Tests for Issue evidence marker and JSON parsing."""

from __future__ import annotations

import json
import unittest

from downstream_issue_evidence import EvidenceSchemaError, parse_issue_evidence
from test_downstream_evidence_fields import REPOSITORY, valid_document


MARKER = "<!-- katana-dependency-evidence:v1 -->"


def issue_body(document: dict[str, object]) -> str:
    return f"context\n{MARKER}\n```json\n{json.dumps(document)}\n```\n"


class IssueEvidenceParsingTests(unittest.TestCase):
    def test_absent_marker_is_unclassified(self) -> None:
        self.assertIsNone(parse_issue_evidence("ordinary Issue prose", REPOSITORY))

    def test_accepts_one_valid_marker_and_block(self) -> None:
        self.assertEqual(parse_issue_evidence(issue_body(valid_document()), REPOSITORY), valid_document())

    def test_rejects_duplicate_marker_or_block(self) -> None:
        body = issue_body(valid_document()) + MARKER
        with self.assertRaises(EvidenceSchemaError):
            parse_issue_evidence(body, REPOSITORY)
        body = issue_body(valid_document()) + "```json\n{}\n```\n"
        with self.assertRaises(EvidenceSchemaError):
            parse_issue_evidence(body, REPOSITORY)

    def test_rejects_malformed_json_and_duplicate_keys(self) -> None:
        for payload in ('{"schema_version":', '{"schema_version":1,"schema_version":1}'):
            body = f"{MARKER}\n```json\n{payload}\n```\n"
            with self.assertRaises(EvidenceSchemaError):
                parse_issue_evidence(body, REPOSITORY)

    def test_rejects_wrong_repository_and_missing_required_records(self) -> None:
        document = valid_document()
        document["repository"] = "Other/Repository"
        with self.assertRaises(EvidenceSchemaError):
            parse_issue_evidence(issue_body(document), REPOSITORY)
        document = valid_document()
        document["changes"][0]["migration_note"] = ""
        with self.assertRaises(EvidenceSchemaError):
            parse_issue_evidence(issue_body(document), REPOSITORY)

    def test_deeply_nested_json_has_a_typed_rejection(self) -> None:
        payload = "[" * 2000 + "0" + "]" * 2000
        body = f"{MARKER}\n```json\n{payload}\n```\n"
        with self.assertRaises(EvidenceSchemaError):
            parse_issue_evidence(body, REPOSITORY)

    def test_accepts_crlf_and_cr_line_endings(self) -> None:
        document = valid_document()
        for ending in ("\r\n", "\r"):
            with self.subTest(ending=ending):
                body = issue_body(document).replace("\n", ending)
                self.assertEqual(parse_issue_evidence(body, REPOSITORY), document)


if __name__ == "__main__":
    unittest.main()
