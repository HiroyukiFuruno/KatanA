"""Serialize cooperative cleanup and expire each private deletion record."""

from contextlib import contextmanager
import os
from pathlib import Path
import tempfile

from cleanup_intent import LOCAL_DELETE, write_intent
from cleanup_intent_expiration import expire_record, record_identity
from cleanup_intent_io import INTENT_NAME
from release_cleanup_audit import CleanupRefused, common_directory


class CleanupSession:
    def __init__(self, root: Path):
        self.root = root.resolve()
        self.lock = common_directory(self.root) / "katana-cleanup.lock"
        self.active = False
        self.operation_active = False

    def __enter__(self):
        if self.active:
            raise CleanupRefused("cleanup session is already active")
        try:
            self.lock.mkdir(mode=0o700)
        except OSError as error:
            raise CleanupRefused("repository cleanup is active or its lock requires inspection") from error
        self.active = True
        return self

    def __exit__(self, kind, error, traceback):
        self.active = False
        # 不明な残存物を再帰削除せず、失敗時はlockを保持して再実行を拒否する。
        try:
            self.lock.rmdir()
        except OSError as failure:
            if kind is None:
                raise CleanupRefused("cleanup lock retains uncertain state") from failure

    @contextmanager
    def intent(self, branch: str, expected: str, *, operation: str = LOCAL_DELETE,
               repository: str | None = None):
        if not self.active or self.operation_active:
            raise CleanupRefused("cleanup intent requires an active idle session")
        directory = Path(tempfile.mkdtemp(prefix="operation-", dir=self.lock))
        descriptor = os.open(directory, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        self.operation_active = True
        identity, digest = None, None
        try:
            path, digest = write_intent(directory, self.root, branch, expected,
                                        operation=operation, repository=repository)
            identity = record_identity(os.stat(INTENT_NAME, dir_fd=descriptor, follow_symlinks=False))
            yield path, digest
        finally:
            try:
                expire_record(descriptor, identity, digest)
            finally:
                os.close(descriptor)
                self.operation_active = False
            directory.rmdir()
