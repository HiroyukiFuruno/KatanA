"""Strict, non-live validation of Issue dependency evidence fields."""

from __future__ import annotations

import re
from urllib.parse import unquote, urlsplit
from collections.abc import Mapping


class EvidenceSchemaError(ValueError):
    """Evidence does not satisfy the syntax and identity contract."""


OWNER_RE = re.compile(r"[A-Za-z0-9_.-]+")
SHA_RE = re.compile(r"[0-9a-f]+")
SEMVER_RE = re.compile(
    r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)"
    r"(?:-(?:0|[1-9][0-9]*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*)"
    r"(?:\.(?:0|[1-9][0-9]*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*))*)?"
    r"(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?\Z"
)


def _mapping(value: object, label: str) -> Mapping[str, object]:
    if not isinstance(value, Mapping) or any(not isinstance(key, str) for key in value):
        raise EvidenceSchemaError(f"{label} must be an object")
    return value


def _exact(value: object, fields: set[str], label: str) -> Mapping[str, object]:
    result = _mapping(value, label)
    if set(result) != fields:
        raise EvidenceSchemaError(f"{label} fields are not exact")
    return result


def _array(value: object, label: str) -> list[object]:
    if not isinstance(value, list) or not value:
        raise EvidenceSchemaError(f"{label} must be nonempty")
    return value


def _text(value: object, label: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise EvidenceSchemaError(f"{label} must be nonempty text")
    return value


def _sha(value: object, lengths: tuple[int, ...], label: str) -> str:
    text = _text(value, label)
    if len(text) not in lengths or SHA_RE.fullmatch(text) is None:
        raise EvidenceSchemaError(f"{label} must be lowercase Git SHA")
    return text


def _repository(value: object, label: str) -> str:
    text = _text(value, label)
    parts = text.split("/")
    if len(parts) != 2 or any(part in {".", ".."} or OWNER_RE.fullmatch(part) is None for part in parts):
        raise EvidenceSchemaError(f"{label} must be OWNER/NAME")
    return text


def _safe_path(value: object, label: str) -> str:
    path = _text(value, label)
    parts = path.split("/")
    if path.startswith("/") or parts[0].startswith("-") or re.match(r"[A-Za-z]:", path):
        raise EvidenceSchemaError(f"{label} must be relative")
    if any(not part or part in {".", ".."} or "\\" in part for part in parts):
        raise EvidenceSchemaError(f"{label} contains unsafe components")
    if any(any(ord(char) < 32 or 127 <= ord(char) <= 159 for char in part) for part in parts):
        raise EvidenceSchemaError(f"{label} contains control characters")
    return path


def _release_tag(value: str) -> None:
    tag = unquote(value)
    if any(part in {"", ".", ".."} for part in tag.split("/")):
        raise EvidenceSchemaError("release_url contains unsafe tag components")
    if any(char.isspace() or ord(char) < 32 or 127 <= ord(char) <= 159 for char in tag):
        raise EvidenceSchemaError("release_url contains unsafe tag characters")


def _release_url(value: object, repository: str) -> str:
    url = _text(value, "release_url")
    if any(char.isspace() or ord(char) < 32 or 127 <= ord(char) <= 159 for char in url):
        raise EvidenceSchemaError("release_url contains unsafe characters")
    try:
        parsed = urlsplit(url)
    except ValueError as error:
        raise EvidenceSchemaError("release_url is malformed") from error
    prefix = f"/{repository}/releases/tag/"
    if parsed.scheme != "https" or parsed.netloc.casefold() != "github.com":
        raise EvidenceSchemaError("release_url must be HTTPS github.com")
    if parsed.username or parsed.password or parsed.port or parsed.query or parsed.fragment:
        raise EvidenceSchemaError("release_url has forbidden URL parts")
    if not parsed.path.startswith(prefix) or not parsed.path[len(prefix):]:
        raise EvidenceSchemaError("release_url must identify a release tag")
    _release_tag(parsed.path[len(prefix):])
    return url


def _package(value: object) -> str:
    package = _text(value, "package")
    if not re.fullmatch(r"[A-Za-z0-9_][A-Za-z0-9_-]*", package):
        raise EvidenceSchemaError("package has invalid Cargo name")
    return package


def _upstream(value: object) -> None:
    record = _exact(value, {"repository", "package", "version", "release_url", "registry_checksum"}, "upstream")
    repository = _repository(record["repository"], "upstream.repository")
    _package(record["package"])
    version = _text(record["version"], "version")
    if SEMVER_RE.fullmatch(version) is None:
        raise EvidenceSchemaError("version must be SemVer")
    _release_url(record["release_url"], repository)
    _sha(record["registry_checksum"], (64,), "registry_checksum")


def _path_record(value: object, label: str) -> str:
    record = _exact(value, {"path", "blob"}, label)
    path = _safe_path(record["path"], f"{label}.path")
    _sha(record["blob"], (40, 64), f"{label}.blob")
    return path


def _verification(value: object, commit: str) -> None:
    record = _exact(value, {"command", "exit_code", "log_path", "log_sha256", "source_commit"}, "verification")
    _text(record["command"], "verification.command")
    if type(record["exit_code"]) is not int or record["exit_code"] != 0:
        raise EvidenceSchemaError("verification.exit_code must be integer zero")
    _safe_path(record["log_path"], "verification.log_path")
    _sha(record["log_sha256"], (64,), "verification.log_sha256")
    if record["source_commit"] != commit:
        raise EvidenceSchemaError("verification.source_commit must match commit")


def _change(value: object, paths: set[str], commits: set[str]) -> None:
    record = _exact(value, {"commit", "upstream", "migration_note", "manifests", "lockfiles", "verification"}, "change")
    commit = _sha(record["commit"], (40, 64), "commit")
    if commit in commits:
        raise EvidenceSchemaError("duplicate commit record")
    commits.add(commit)
    for upstream in _array(record["upstream"], "upstream"):
        _upstream(upstream)
    _text(record["migration_note"], "migration_note")
    for label in ("manifests", "lockfiles"):
        for entry in _array(record[label], label):
            path = _path_record(entry, label)
            if path in paths:
                raise EvidenceSchemaError("repeated manifest or lockfile path")
            paths.add(path)
    for verification in _array(record["verification"], "verification"):
        _verification(verification, commit)


def validate_evidence_document(document: object, repository: str) -> None:
    """Validate syntax only; no publication, command, or CI claim is made."""
    top = _exact(document, {"schema_version", "kind", "repository", "changes"}, "evidence")
    if type(top["schema_version"]) is not int or top["schema_version"] != 1:
        raise EvidenceSchemaError("schema_version must be integer 1")
    if top["kind"] != "downstream" or top["repository"] != repository:
        raise EvidenceSchemaError("kind or repository does not match")
    changes = _array(top["changes"], "changes")
    commits: set[str] = set()
    for change in changes:
        _change(change, set(), commits)
