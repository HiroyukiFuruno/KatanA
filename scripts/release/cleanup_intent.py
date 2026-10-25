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


def _schema_value(value: dict[str, object], operation: str) -> tuple[int, set[str]]:
    if operation != REMOTE_DELETE:
        if "merge_commit" in value or "pull_request" in value:
            raise CleanupRefused("local cleanup cannot carry remote merge metadata")
        return _schema(operation)
    pair = {"merge_commit", "pull_request"} & value.keys()
    if pair and pair != {"merge_commit", "pull_request"}:
        raise CleanupRefused("remote cleanup merge metadata must be complete")
    if pair:
        return 3, FIELDS | {"operation", "repository", "merge_commit", "pull_request"}
    return _schema(operation)


def _unique_object(pairs: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for name, item in pairs:
        if name in value:
            raise CleanupRefused("cleanup intent contains duplicate fields")
        value[name] = item
    return value


def _validate(value: object, operation: str = LOCAL_DELETE) -> dict[str, object]:
    if not isinstance(value, dict):
        raise CleanupRefused("cleanup intent fields do not match its schema")
    version, fields = _schema_value(value, operation)
    if set(value) != fields:
        raise CleanupRefused("cleanup intent fields do not match its schema")
    if type(value["schema"]) is not int or value["schema"] != version:
        raise CleanupRefused("cleanup intent schema is unsupported")
    if any(not isinstance(value[key], str) for key in fields - {"schema"}):
        raise CleanupRefused("cleanup intent fields must be strings")
    if operation == REMOTE_DELETE:
        _validate_remote(value)
        if version == 3:
            if OID.fullmatch(value["merge_commit"]) is None:
                raise CleanupRefused("cleanup intent merge commit is malformed")
            if re.fullmatch(r"[1-9][0-9]*", value["pull_request"]) is None:
                raise CleanupRefused("cleanup intent pull request is malformed")
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
                 operation: str = LOCAL_DELETE, repository: str | None = None,
                 merge_commit: str | None = None, pull_request: str | None = None) -> tuple[Path, str]:
    if operation == LOCAL_DELETE and (repository is not None or merge_commit is not None
                                      or pull_request is not None):
        raise CleanupRefused("local cleanup cannot carry a remote repository")
    if operation == REMOTE_DELETE and ((merge_commit is None) != (pull_request is None)):
        raise CleanupRefused("remote cleanup merge metadata must be complete")
    version = 3 if operation == REMOTE_DELETE and merge_commit is not None else _schema(operation)[0]
    directory = directory.absolute()
    root = root.resolve()
    value = {"schema": version, "nonce": secrets.token_hex(32), "root": str(root),
             "common_git_dir": str(common_directory(root)), "branch": branch, "expected": expected}
    if operation == REMOTE_DELETE:
        value.update(operation=operation, repository=repository)
        if version == 3:
            value.update(merge_commit=merge_commit, pull_request=pull_request)
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
