"""Keep cleanup outcomes outside any removable working directory."""

from datetime import datetime, timezone
import json
import os
from pathlib import Path
import secrets

from cleanup_intent_io import _directory
from release_cleanup_audit import common_directory

MAX_FAILURE_CHARACTERS = 1024


class CleanupReceipt:
    def __init__(self, controller: Path, request: dict[str, object]):
        directory = common_directory(controller) / "katana-cleanup-receipts"
        directory.mkdir(mode=0o700, exist_ok=True)
        parent = _directory(directory)
        name = "run-" + secrets.token_hex(16) + ".jsonl"
        try:
            descriptor = os.open(name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                                 0o600, dir_fd=parent)
        finally:
            os.close(parent)
        self.path = directory / name
        self.stream = os.fdopen(descriptor, "w", encoding="utf-8")
        self.request = request

    def __enter__(self):
        try:
            self.record("cleanup", "started", self.request)
            print("cleanup receipt: " + str(self.path), flush=True)
            return self
        except BaseException:
            self.stream.close()
            raise

    def record(self, action: str, state: str, detail: object) -> None:
        value = {"schema": 1, "observed_at": datetime.now(timezone.utc).isoformat(),
                 "action": action, "state": state, "detail": detail}
        self.stream.write(json.dumps(value, ensure_ascii=True, sort_keys=True) + "\n")
        self.stream.flush()
        os.fsync(self.stream.fileno())

    def __exit__(self, kind, error, traceback):
        try:
            if error is None:
                self.record("cleanup", "completed", self.request)
            else:
                self.record("cleanup", "failed", str(error)[:MAX_FAILURE_CHARACTERS])
        finally:
            self.stream.close()
