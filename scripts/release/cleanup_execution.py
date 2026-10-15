"""Execute an admitted cleanup through ordinary owner-preserving Git hooks."""

import os
from pathlib import Path
import subprocess
import stat
import sys

from cleanup_existing_hook import DIGEST_ENV, INTENT_ENV, REMOTE_DIGEST_ENV, REMOTE_INTENT_ENV
from cleanup_existing_hook import owner_environment
from cleanup_intent import REMOTE_DELETE
from cleanup_remote_inventory import remote_tip
from cleanup_session import CleanupSession
from hook_installation_scope import clean_environment, validate_hooks_scope, validate_regular
from reference_hook_installation import BACKUP_NAME, installed
from release_cleanup_audit import CleanupRefused, _local_branch_oid, audit_branch, common_directory
from release_cleanup_audit import git, require_controller
from release_cleanup_publication import cleanup_destination


class CleanupExecutor:
    def __init__(self, controller: Path, repository: str, default: str,
                 branch: str, expected: str):
        self.controller, self.repository = controller.resolve(), repository
        self.default, self.branch, self.expected = default, branch, expected
        self.reference_snapshot = None
        self.remote_snapshot = None

    def _command(self, arguments: list[str], scope: dict[str, str] | None = None) -> None:
        environment = owner_environment(clean_environment())
        environment.update(scope or {})
        subprocess.run(arguments, cwd=self.controller, env=environment, check=True)

    def prepare_hooks(self, *, include_remote: bool) -> None:
        require_controller(self.controller, self.controller, self.default)
        runtime = self.controller / "scripts" / "release"
        self._command([sys.executable, "-B", str(runtime / "install-reference-dispatcher.py")])
        self.reference_snapshot = self._reference_hook_snapshot()
        if include_remote:
            self._command([sys.executable, "-B", str(runtime / "install-pre-push-dispatcher.py"),
                           "--repository", self.repository])
            self.remote_snapshot = self._push_hook_snapshot()

    def _audit(self) -> None:
        require_controller(self.controller, self.controller, self.default)
        result = audit_branch(self.controller, self.default, self.branch, self.expected,
                              allow_absent=True)
        if result["state"] != "eligible":
            raise CleanupRefused("cleanup target must be retained: " + result["reason"])

    def _hooks(self) -> Path:
        raw = git(self.controller, "rev-parse", "--path-format=absolute", "--git-path", "hooks")
        hooks = Path(os.fsdecode(raw.strip()))
        validate_hooks_scope(hooks, common_directory(self.controller))
        return hooks

    def _reference_hook_snapshot(self) -> tuple:
        hooks = self._hooks()
        hook, backup = hooks / "reference-transaction", hooks / BACKUP_NAME
        validate_regular(hook, "reference target")
        validate_regular(backup, "reference owner")
        if not installed(hook, backup):
            raise CleanupRefused("cleanup reference hook changed after preparation")
        return self._snapshot([hook, backup] if backup.exists() else [hook])

    def _reference_hook(self) -> None:
        if self.reference_snapshot is None or self._reference_hook_snapshot() != self.reference_snapshot:
            raise CleanupRefused("cleanup reference owner changed or has not been prepared")

    def _snapshot(self, paths: list[Path]) -> tuple:
        records = []
        for path in paths:
            validate_regular(path, "cleanup hook")
            if not path.is_file() or not os.access(path, os.X_OK):
                raise CleanupRefused("cleanup hook is missing or not executable")
            records.append((path, path.read_bytes(), stat.S_IMODE(path.stat().st_mode)))
        return tuple(records)

    def _push_hook_snapshot(self) -> tuple:
        hooks = self._hooks()
        return self._snapshot([hooks / "pre-push", hooks / "pre-push.katana-existing"])

    def verify_remote_hook(self) -> None:
        if self.remote_snapshot is None or self._push_hook_snapshot() != self.remote_snapshot:
            raise CleanupRefused("cleanup push hook changed or has not been prepared")

    def delete_local(self, session: CleanupSession) -> str:
        self._audit()
        self._reference_hook()
        if _local_branch_oid(self.controller, self.branch) is None:
            return "absent"
        with session.intent(self.branch, self.expected) as (path, digest):
            self._command(["git", "branch", "-d", "--", self.branch],
                          {INTENT_ENV: str(path), DIGEST_ENV: digest})
        if _local_branch_oid(self.controller, self.branch) is not None:
            raise CleanupRefused("local cleanup target still exists")
        return "deleted"

    def delete_remote(self, session: CleanupSession) -> str:
        self._audit()
        self.verify_remote_hook()
        destination = cleanup_destination(self.controller, self.controller, self.repository)
        if remote_tip(self.controller, destination, self.branch, self.expected) is None:
            return "absent"
        with session.intent(self.branch, self.expected, operation=REMOTE_DELETE,
                            repository=self.repository) as (path, digest):
            self._command(["git", "push", "origin", "--delete", "refs/heads/" + self.branch],
                          {REMOTE_INTENT_ENV: str(path), REMOTE_DIGEST_ENV: digest})
        if remote_tip(self.controller, destination, self.branch, self.expected) is not None:
            raise CleanupRefused("remote cleanup target still exists")
        return "deleted"
