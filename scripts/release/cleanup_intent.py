"""Bind a private single-operation record; not publication authorization."""

import hashlib
import json
from pathlib import Path
import re
import secrets

from cleanup_intent_io import read_private, write_private
from issue_reference_syntax import parse_repository
from release_cleanup_audit import CleanupRefused, common_directory, git
from release_cleanup_tag import OID

SHA256 = re.compile(r"[0-9a-f]{64}\Z")
FIELDS = {"schema", "nonce", "root", "common_git_dir", "branch", "expected"}
LOCAL_DELETE = "local-delete"
REMOTE_DELETE = "remote-delete"


def _schema(operation: str) -> tuple[int, set[str]]:
    if operation == LOCAL_DELETE:
        return 1, FIELDS
    if operation == REMOTE_DELETE:
        return 2, FIELDS | {"operation", "repository"}
    raise CleanupRefused("cleanup operation is unsupported")


def _validate_remote(value: dict[str, object]) -> None:
    if value["operation"] != REMOTE_DELETE:
        raise CleanupRefused("cleanup intent operation does not match")
    try:
        parse_repository(value["repository"])
    except ValueError as error:
        raise CleanupRefused("cleanup intent repository is malformed") from error


def _unique_object(pairs: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for name, item in pairs:
        if name in value:
            raise CleanupRefused("cleanup intent contains duplicate fields")
        value[name] = item
    return value


def _validate(value: object, operation: str = LOCAL_DELETE) -> dict[str, object]:
    version, fields = _schema(operation)
    if not isinstance(value, dict) or set(value) != fields:
        raise CleanupRefused("cleanup intent fields do not match its schema")
    if type(value["schema"]) is not int or value["schema"] != version:
        raise CleanupRefused("cleanup intent schema is unsupported")
    if any(not isinstance(value[key], str) for key in fields - {"schema"}):
        raise CleanupRefused("cleanup intent fields must be strings")
    if operation == REMOTE_DELETE:
        _validate_remote(value)
    if SHA256.fullmatch(value["nonce"]) is None or OID.fullmatch(value["expected"]) is None:
        raise CleanupRefused("cleanup intent identity is malformed")
    root = Path(value["root"])
    if not root.is_absolute() or str(root.resolve()) != value["root"]:
        raise CleanupRefused("cleanup intent root is not canonical")
    if not value["branch"].startswith("release/"):
        raise CleanupRefused("cleanup intent requires a release branch")
    git(root, "check-ref-format", "refs/heads/" + value["branch"])
    if str(common_directory(root)) != value["common_git_dir"]:
        raise CleanupRefused("cleanup intent common repository changed")
    return value


def write_intent(directory: Path, root: Path, branch: str, expected: str, *,
                 operation: str = LOCAL_DELETE, repository: str | None = None) -> tuple[Path, str]:
    version, _ = _schema(operation)
    if operation == LOCAL_DELETE and repository is not None:
        raise CleanupRefused("local cleanup cannot carry a remote repository")
    directory = directory.absolute()
    root = root.resolve()
    value = {"schema": version, "nonce": secrets.token_hex(32), "root": str(root),
             "common_git_dir": str(common_directory(root)), "branch": branch, "expected": expected}
    if operation == REMOTE_DELETE:
        value.update(operation=operation, repository=repository)
    _validate(value, operation)
    payload = json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")
    try:
        path = write_private(directory, payload)
    except OSError as error:
        raise CleanupRefused("cleanup intent could not be created privately") from error
    return path, hashlib.sha256(payload).hexdigest()


def read_intent(path: Path, digest: str, *, operation: str = LOCAL_DELETE) -> dict[str, object]:
    if SHA256.fullmatch(digest) is None:
        raise CleanupRefused("cleanup intent digest is malformed")
    try:
        payload = read_private(path)
        if hashlib.sha256(payload).hexdigest() != digest:
            raise CleanupRefused("cleanup intent digest changed")
        value = json.loads(payload.decode("utf-8"), object_pairs_hook=_unique_object)
    except (OSError, UnicodeError, ValueError) as error:
        raise CleanupRefused("cleanup intent is not a verified private record") from error
    return _validate(value, operation)
