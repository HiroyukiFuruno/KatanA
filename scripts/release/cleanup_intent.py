"""Bind a private single-operation record; not publication authorization."""

import hashlib
import json
from pathlib import Path
import re
import secrets

from cleanup_intent_io import read_private, write_private
from release_cleanup_audit import CleanupRefused, common_directory, git
from release_cleanup_tag import OID

SHA256 = re.compile(r"[0-9a-f]{64}\Z")
FIELDS = {"schema", "nonce", "root", "common_git_dir", "branch", "expected"}


def _unique_object(pairs: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for name, item in pairs:
        if name in value:
            raise CleanupRefused("cleanup intent contains duplicate fields")
        value[name] = item
    return value


def _validate(value: object) -> dict[str, object]:
    if not isinstance(value, dict) or set(value) != FIELDS:
        raise CleanupRefused("cleanup intent fields do not match its schema")
    if type(value["schema"]) is not int or value["schema"] != 1:
        raise CleanupRefused("cleanup intent schema is unsupported")
    if any(not isinstance(value[key], str) for key in FIELDS - {"schema"}):
        raise CleanupRefused("cleanup intent fields must be strings")
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


def write_intent(directory: Path, root: Path, branch: str, expected: str) -> tuple[Path, str]:
    directory = directory.absolute()
    root = root.resolve()
    value = {"schema": 1, "nonce": secrets.token_hex(32), "root": str(root),
             "common_git_dir": str(common_directory(root)), "branch": branch, "expected": expected}
    _validate(value)
    payload = json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")
    try:
        path = write_private(directory, payload)
    except OSError as error:
        raise CleanupRefused("cleanup intent could not be created privately") from error
    return path, hashlib.sha256(payload).hexdigest()


def read_intent(path: Path, digest: str) -> dict[str, object]:
    if SHA256.fullmatch(digest) is None:
        raise CleanupRefused("cleanup intent digest is malformed")
    try:
        payload = read_private(path)
        if hashlib.sha256(payload).hexdigest() != digest:
            raise CleanupRefused("cleanup intent digest changed")
        value = json.loads(payload.decode("utf-8"), object_pairs_hook=_unique_object)
    except (OSError, UnicodeError, ValueError) as error:
        raise CleanupRefused("cleanup intent is not a verified private record") from error
    return _validate(value)
