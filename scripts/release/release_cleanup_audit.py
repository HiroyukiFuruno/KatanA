"""Native Git synchronization and retention audit for post-publication cleanup."""

from __future__ import annotations

import os
from pathlib import Path
import re

from bounded_git_output import bounded_git
from downstream_git_binding import GitBindingError

GIT_OUTPUT_LIMIT = 4 * 1024 * 1024
GIT_DEADLINE_SECONDS = 30
CANONICAL_OID = re.compile(r"(?:[0-9a-f]{40}|[0-9a-f]{64})\Z")


class CleanupRefused(ValueError):
    """Preserve repository state when cleanup cannot be safely established."""


def git(root: Path, *arguments: str) -> bytes:
    try:
        return bounded_git(root, list(arguments), stdout_limit=GIT_OUTPUT_LIMIT,
                           timeout=GIT_DEADLINE_SECONDS)
    except GitBindingError as error:
        raise CleanupRefused("native Git cleanup check failed") from error


def common_directory(root: Path) -> Path:
    return Path(os.fsdecode(git(root, "rev-parse", "--path-format=absolute", "--git-common-dir").strip())).resolve()


def require_repository_scope(root: Path, controller: Path) -> None:
    if root.resolve() != Path(os.fsdecode(git(root, "rev-parse", "--show-toplevel").strip())).resolve():
        raise CleanupRefused("cleanup source is not a repository root")
    actual = Path(os.fsdecode(git(controller, "rev-parse", "--show-toplevel").strip())).resolve()
    if actual != controller.resolve() or common_directory(root) != common_directory(controller):
        raise CleanupRefused("default controller is not an existing worktree of this repository")


def require_controller(root: Path, controller: Path, default: str) -> None:
    require_repository_scope(root, controller)
    git(controller, "check-ref-format", "refs/heads/" + default)
    if git(controller, "symbolic-ref", "HEAD").strip() != ("refs/heads/" + default).encode():
        raise CleanupRefused("explicit controller is not on the default branch")
    if git(controller, "status", "--porcelain=v1", "-z", "--untracked-files=all"):
        raise CleanupRefused("default controller has uncommitted files")
    if git(controller, "stash", "list"):
        raise CleanupRefused("repository contains a stash")


def synchronize_default(root: Path, controller: Path, default: str, *, destination: str = "origin") -> str:
    require_controller(root, controller, default)
    # 別worktreeで使われているdefaultをrelease側へ強制移動しない。
    git(controller, "switch", "--", default)
    git(controller, "pull", "--ff-only", "--no-autostash", destination, default)
    require_controller(root, controller, default)
    head = git(controller, "rev-parse", "HEAD").decode().strip()
    remote = git(controller, "ls-remote", "--exit-code", destination, "refs/heads/" + default)
    if remote.split() != [head.encode(), ("refs/heads/" + default).encode()]:
        raise CleanupRefused("default moved during synchronization")
    return head


def _local_branch_oid(controller: Path, branch: str) -> str | None:
    reference = "refs/heads/" + branch
    raw = git(controller, "for-each-ref", "--format=%(refname)%00%(objectname)", "--", reference)
    records = [line.split(b"\0") for line in raw.split(b"\n") if line]
    exact = [fields[1] for fields in records if len(fields) == 2 and fields[0] == reference.encode()]
    if not exact:
        return None
    if len(exact) != 1:
        raise CleanupRefused("local cleanup target is ambiguous")
    return exact[0].decode("ascii")


def _retention(controller: Path, default: str, branch: str, oid: str,
               has_local: bool, integrated_oid: str | None = None) -> dict[str, str]:
    inventory = git(controller, "worktree", "list", "--porcelain", "-z")
    if ("branch refs/heads/" + branch).encode() in inventory.split(b"\0"):
        return {"branch": branch, "oid": oid, "state": "retain", "reason": "worktree in use"}
    try:
        git(controller, "merge-base", "--is-ancestor", integrated_oid or oid, "refs/heads/" + default)
    except CleanupRefused:
        return {"branch": branch, "oid": oid, "state": "retain", "reason": "merge not established"}
    reason = "merged unused local branch" if has_local else "merged target without a local branch"
    return {"branch": branch, "oid": oid, "state": "eligible", "reason": reason}


def audit_branch(controller: Path, default: str, branch: str, expected_oid: str, *,
                 allow_absent: bool = False, integrated_oid: str | None = None) -> dict[str, str]:
    git(controller, "check-ref-format", "refs/heads/" + branch)
    if branch in {default, "master", "main"} or not branch.startswith("release/"):
        raise CleanupRefused("cleanup target must be an explicit non-default release branch")
    if CANONICAL_OID.fullmatch(expected_oid) is None:
        raise CleanupRefused("cleanup target requires a full commit identity")
    if integrated_oid is not None and CANONICAL_OID.fullmatch(integrated_oid) is None:
        raise CleanupRefused("cleanup integration requires a full commit identity")
    oid = _local_branch_oid(controller, branch)
    if oid is None and not allow_absent:
        raise CleanupRefused("cleanup target local branch is missing")
    if oid is not None and oid != expected_oid:
        raise CleanupRefused("cleanup target no longer matches the audited commit")
    if oid is not None or integrated_oid is None:
        if git(controller, "cat-file", "-t", expected_oid).strip() != b"commit":
            raise CleanupRefused("cleanup target is not a commit")
        canonical = git(controller, "rev-parse", "--verify", expected_oid + "^{commit}").decode().strip()
        if canonical != expected_oid:
            raise CleanupRefused("cleanup target must use its full canonical commit identity")
    return _retention(controller, default, branch, expected_oid, oid is not None, integrated_oid)
