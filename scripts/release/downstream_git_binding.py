"""Bind validated Issue declarations to real immutable Git tree objects."""

from __future__ import annotations

import os
from pathlib import Path
import subprocess

from downstream_evidence_fields import EvidenceSchemaError, validate_evidence_document


class GitBindingError(EvidenceSchemaError):
    """A declared source identity is not present in the real Git tree."""


def _git(root: Path, arguments: list[str]) -> bytes:
    # hook環境・replace参照・暗黙fetchで、別objectや未取得の証拠へ照合を逸らさない。
    environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
    try:
        result = subprocess.run(
            ["git", "--no-lazy-fetch", "--no-replace-objects", "--literal-pathspecs",
             "-C", str(root), *arguments],
            env=environment, capture_output=True, check=True,
        )
    except (OSError, UnicodeError, subprocess.CalledProcessError) as error:
        raise GitBindingError("Git source identity could not be read") from error
    return result.stdout


def _source_commit(root: Path, commit: str) -> None:
    if _git(root, ["rev-parse", "--verify", commit]).strip() != commit.encode("ascii"):
        raise GitBindingError("declared source identity is not a full commit OID")
    if _git(root, ["cat-file", "-t", commit]).strip() != b"commit":
        raise GitBindingError("declared source object is not a commit")


def _path_binding(root: Path, commit: str, record: dict[str, str], label: str) -> None:
    path = record["path"]
    expected_name = "Cargo.toml" if label == "manifests" else "Cargo.lock"
    if path.rsplit("/", 1)[-1] != expected_name:
        raise GitBindingError("manifest and lockfile classification does not match")
    try:
        encoded_path = path.encode("utf-8")
    except UnicodeError as error:
        raise GitBindingError("declared Git path is not UTF-8") from error
    listing = _git(root, ["ls-tree", "-r", "-z", "--full-tree", commit, "--", path])
    entries = listing.split(b"\0")
    if len(entries) != 2 or entries[-1] != b"":
        raise GitBindingError("declared path is missing or ambiguous")
    metadata, separator, actual_path = entries[0].partition(b"\t")
    if not separator or actual_path != encoded_path:
        raise GitBindingError("Git tree path does not match")
    fields = metadata.split(b" ")
    if len(fields) != 3 or fields[0] not in (b"100644", b"100755") or fields[1] != b"blob":
        raise GitBindingError("declared path is not a regular file blob")
    if fields[2] != record["blob"].encode("ascii"):
        raise GitBindingError("declared blob does not match the commit tree")
    if _git(root, ["cat-file", "-t", record["blob"]]).strip() != b"blob":
        raise GitBindingError("declared object is not an available blob")


def validate_commit_tree_bindings(document: dict[str, object], repository_root: Path,
                                  repository: str) -> None:
    """Verify tree identities only, not publication or verification execution."""
    validate_evidence_document(document, repository)
    root = repository_root.resolve()
    for change in document["changes"]:
        commit = change["commit"]
        _source_commit(root, commit)
        for label in ("manifests", "lockfiles"):
            for record in change[label]:
                _path_binding(root, commit, record, label)
