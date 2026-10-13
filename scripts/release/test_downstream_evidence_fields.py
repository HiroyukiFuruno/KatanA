#!/usr/bin/env python3
"""Tests for strict downstream evidence field validation."""

from __future__ import annotations

import copy
import unittest

from downstream_evidence_fields import EvidenceSchemaError, validate_evidence_document


REPOSITORY = "HiroyukiFuruno/KatanA"
COMMIT = "a" * 40


def valid_document() -> dict[str, object]:
    return {
        "schema_version": 1,
        "kind": "downstream",
        "repository": REPOSITORY,
        "changes": [{
            "commit": COMMIT,
            "upstream": [{
                "repository": "HiroyukiFuruno/KatanaDocumentViewer",
                "package": "katana-document-viewer",
                "version": "0.5.12",
                "release_url": "https://github.com/HiroyukiFuruno/KatanaDocumentViewer/releases/tag/v0.5.12",
                "registry_checksum": "b" * 64,
            }],
            "migration_note": "API unchanged; renderer adapter remains compatible.",
            "manifests": [{"path": "Cargo.toml", "blob": "c" * 40}],
            "lockfiles": [{"path": "Cargo.lock", "blob": "d" * 64}],
            "verification": [{
                "command": "just check",
                "exit_code": 0,
                "log_path": "tmp/verify.log",
                "log_sha256": "e" * 64,
                "source_commit": COMMIT,
            }],
        }],
    }


class EvidenceFieldTests(unittest.TestCase):
    def assert_invalid(self, document: dict[str, object]) -> None:
        with self.assertRaises(EvidenceSchemaError):
            validate_evidence_document(document, REPOSITORY)

    def test_accepts_valid_document(self) -> None:
        validate_evidence_document(valid_document(), REPOSITORY)

    def test_rejects_unknown_or_missing_fields(self) -> None:
        document = valid_document()
        document["unexpected"] = True
        self.assert_invalid(document)
        document = valid_document()
        del document["changes"]
        self.assert_invalid(document)

    def test_rejects_false_success_types(self) -> None:
        document = valid_document()
        document["schema_version"] = True
        self.assert_invalid(document)
        document = valid_document()
        document["changes"][0]["verification"][0]["exit_code"] = False
        self.assert_invalid(document)

    def test_rejects_invalid_commit_sha_and_source_binding(self) -> None:
        document = valid_document()
        document["changes"][0]["commit"] = "A" * 40
        self.assert_invalid(document)
        document = valid_document()
        document["changes"][0]["verification"][0]["source_commit"] = "f" * 40
        self.assert_invalid(document)

    def test_rejects_invalid_version_url_checksum_and_package(self) -> None:
        for field, value in (
            ("version", "01.2.3"),
            ("version", "1.2.3-01"),
            ("release_url", "https://github.com/HiroyukiFuruno/KatanaDocumentViewer/releases/tag/"),
            ("registry_checksum", "B" * 64),
            ("package", "-bad"),
        ):
            document = valid_document()
            document["changes"][0]["upstream"][0][field] = value
            self.assert_invalid(document)

    def test_rejects_unsafe_or_repeated_paths(self) -> None:
        for path in ("/Cargo.toml", "../Cargo.toml", "a//b", "-tmp/log", "a\\b"):
            document = valid_document()
            document["changes"][0]["verification"][0]["log_path"] = path
            self.assert_invalid(document)
        document = valid_document()
        document["changes"][0]["lockfiles"][0]["path"] = "Cargo.toml"
        self.assert_invalid(document)

    def test_rejects_failed_or_noninteger_verification(self) -> None:
        for exit_code in (1, -1, True, 0.0, "0"):
            with self.subTest(exit_code=exit_code):
                document = valid_document()
                document["changes"][0]["verification"][0]["exit_code"] = exit_code
                self.assert_invalid(document)

    def test_rejects_blank_migration_and_command(self) -> None:
        for blank in (" ", "\t\n"):
            with self.subTest(blank=blank):
                document = valid_document()
                document["changes"][0]["migration_note"] = blank
                self.assert_invalid(document)
                document = valid_document()
                document["changes"][0]["verification"][0]["command"] = blank
                self.assert_invalid(document)

    def test_accepts_same_paths_in_distinct_commit_records(self) -> None:
        document = valid_document()
        second = copy.deepcopy(document["changes"][0])
        second["commit"] = "f" * 40
        second["verification"][0]["source_commit"] = second["commit"]
        document["changes"].append(second)
        validate_evidence_document(document, REPOSITORY)

    def test_rejects_drive_paths_and_control_components(self) -> None:
        for path in ("C:/Cargo.toml", "C:Cargo.toml", "a/\x85b"):
            with self.subTest(path=path):
                document = valid_document()
                document["changes"][0]["manifests"][0]["path"] = path
                self.assert_invalid(document)

    def test_rejects_duplicate_upstream_identity(self) -> None:
        for field in (None, "registry_checksum", "release_url"):
            with self.subTest(field=field):
                document = valid_document()
                duplicate = copy.deepcopy(document["changes"][0]["upstream"][0])
                if field == "registry_checksum":
                    duplicate[field] = "f" * 64
                if field == "release_url":
                    duplicate[field] += "/candidate"
                document["changes"][0]["upstream"].append(duplicate)
                self.assert_invalid(document)

    def test_accepts_distinct_upstream_versions(self) -> None:
        document = valid_document()
        newer = copy.deepcopy(document["changes"][0]["upstream"][0])
        newer["version"] = "0.5.13"
        newer["release_url"] = newer["release_url"].replace("v0.5.12", "v0.5.13")
        document["changes"][0]["upstream"].append(newer)
        validate_evidence_document(document, REPOSITORY)

    def test_rejects_missing_collections_and_duplicate_commits(self) -> None:
        for field in ("upstream", "manifests", "lockfiles", "verification"):
            with self.subTest(field=field):
                document = valid_document()
                document["changes"][0][field] = []
                self.assert_invalid(document)
        document = valid_document()
        document["changes"].append(copy.deepcopy(document["changes"][0]))
        self.assert_invalid(document)

    def test_release_urls_do_not_change_identity_during_normalization(self) -> None:
        prefix = "https://github.com/HiroyukiFuruno/KatanaDocumentViewer/releases/tag/"
        for url in (
            " " + prefix + "v0.5.12", prefix + "\nv0.5.12", prefix + "../../other",
            prefix + "%2e%2e/other", prefix + "v0.5.12/%0aother", "https://[github.com/tag",
            prefix + r"v1\..\..\issues\335", prefix + "v1%5c..%5cissues",
            prefix + "v1%5C..%5Cissues",
        ):
            with self.subTest(url=url):
                document = valid_document()
                document["changes"][0]["upstream"][0]["release_url"] = url
                self.assert_invalid(document)

    def test_accepts_nested_release_tags(self) -> None:
        document = valid_document()
        document["changes"][0]["upstream"][0]["release_url"] += "/candidate"
        validate_evidence_document(document, REPOSITORY)

    def test_rejects_repository_path_traversal(self) -> None:
        for repository in ("../Repo", "Owner/..", "./Repo"):
            with self.subTest(repository=repository):
                document = valid_document()
                upstream = document["changes"][0]["upstream"][0]
                upstream["repository"] = repository
                upstream["release_url"] = f"https://github.com/{repository}/releases/tag/v0.5.12"
                self.assert_invalid(document)


if __name__ == "__main__":
    unittest.main()
