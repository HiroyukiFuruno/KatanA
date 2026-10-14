"""Bind one fixed owner check to the dispatcher-created push session."""

import json
import os
from pathlib import Path
import secrets
import stat
import subprocess
import tempfile

SESSION_DIRECTORY = "KATANA_OWNER_CHECK_DIRECTORY"
SESSION_TOKEN = "KATANA_OWNER_CHECK_TOKEN"
RECEIPT_NAME = "verification.json"
COMMAND = ["just", "check"]
MAX_RECEIPT_BYTES = 4096


def source_revision(root: Path) -> str:
    result = subprocess.run(["git", "rev-parse", "--verify", "HEAD"], cwd=root,
                            check=True, capture_output=True, text=True)
    dirty = subprocess.run(["git", "status", "--porcelain", "--untracked-files=no"],
                           cwd=root, check=True, capture_output=True)
    if dirty.stdout.strip():
        raise ValueError("owner verification requires clean tracked source")
    return result.stdout.strip()


def _private_directory(path: Path) -> None:
    metadata = path.lstat()
    if not stat.S_ISDIR(metadata.st_mode) or stat.S_IMODE(metadata.st_mode) != 0o700:
        raise ValueError("owner verification directory is not private")
    if hasattr(os, "getuid") and metadata.st_uid != os.getuid():
        raise ValueError("owner verification directory has another owner")


def _read_receipt(path: Path) -> dict:
    descriptor = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_NONBLOCK", 0))
    with os.fdopen(descriptor, "rb") as stream:
        metadata = os.fstat(stream.fileno())
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_size > MAX_RECEIPT_BYTES:
            raise ValueError("owner verification receipt is not a bounded regular file")
        value = json.loads(stream.read(MAX_RECEIPT_BYTES + 1))
    if not isinstance(value, dict):
        raise ValueError("owner verification receipt is not an object")
    return value


class VerificationSession:
    def __init__(self, root: Path, revision: str):
        self.root, self.revision = root.resolve(), revision
        self.temporary = tempfile.TemporaryDirectory(prefix="katana-owner-push-")
        self.directory = Path(self.temporary.name).resolve()
        self.token = secrets.token_hex(32)

    def __enter__(self):
        _private_directory(self.directory)
        return self

    def __exit__(self, *_arguments):
        self.temporary.cleanup()

    def environment(self) -> dict[str, str]:
        return os.environ | {SESSION_DIRECTORY: str(self.directory), SESSION_TOKEN: self.token}

    def verified_source(self) -> str | None:
        path = self.directory / RECEIPT_NAME
        if not path.exists() and not path.is_symlink():
            return None
        value = _read_receipt(path)
        expected = {"schema": 1, "token": self.token, "root": str(self.root),
                    "source": self.revision, "command": COMMAND, "exitCode": 0}
        if value != expected:
            raise ValueError("owner verification receipt does not match this push session")
        return self.revision


def _record_success(directory: Path, token: str, root: Path, revision: str) -> None:
    _private_directory(directory)
    if len(token) != 64 or any(character not in "0123456789abcdef" for character in token):
        raise ValueError("owner verification session token is invalid")
    value = {"schema": 1, "token": token, "root": str(root), "source": revision,
             "command": COMMAND, "exitCode": 0}
    descriptor = os.open(directory / RECEIPT_NAME,
                         os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0), 0o600)
    with os.fdopen(descriptor, "w", encoding="utf-8") as stream:
        json.dump(value, stream, sort_keys=True)


def run_owned_check(root: Path) -> int:
    root = root.resolve()
    directory, token = os.environ.get(SESSION_DIRECTORY), os.environ.get(SESSION_TOKEN)
    if bool(directory) != bool(token):
        raise ValueError("owner verification session is incomplete")
    before = source_revision(root) if directory else None
    if directory:
        _private_directory(Path(directory))
    status = subprocess.run(COMMAND, cwd=root, check=False).returncode
    if status == 0 and directory:
        if source_revision(root) != before:
            raise ValueError("owner check changed its source revision")
        _record_success(Path(directory), token, root, before)
    return status
