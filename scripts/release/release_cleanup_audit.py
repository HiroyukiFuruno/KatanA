"""Native Git synchronization and retention audit for post-publication cleanup."""

from __future__ import annotations

import os
from pathlib import Path

from bounded_git_output import bounded_git
from downstream_git_binding import GitBindingError

GIT_OUTPUT_LIMIT = 4 * 1024 * 1024
GIT_DEADLINE_SECONDS = 30


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


def require_controller(root: Path, controller: Path, default: str) -> None:
    if root.resolve() != Path(os.fsdecode(git(root, "rev-parse", "--show-toplevel").strip())).resolve():
        raise CleanupRefused("cleanup source is not a repository root")
    actual = Path(os.fsdecode(git(controller, "rev-parse", "--show-toplevel").strip())).resolve()
    if actual != controller.resolve() or common_directory(root) != common_directory(controller):
        raise CleanupRefused("default controller is not an existing worktree of this repository")
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


def audit_branch(controller: Path, default: str, branch: str, expected_oid: str) -> dict[str, str]:
    git(controller, "check-ref-format", "refs/heads/" + branch)
    if branch in {default, "master", "main"} or not branch.startswith("release/"):
        raise CleanupRefused("cleanup target must be an explicit non-default release branch")
    oid = git(controller, "rev-parse", "--verify", "refs/heads/" + branch).decode().strip()
    if oid != expected_oid or git(controller, "cat-file", "-t", oid).strip() != b"commit":
        raise CleanupRefused("cleanup target no longer matches the audited commit")
    inventory = git(controller, "worktree", "list", "--porcelain", "-z")
    if ("branch refs/heads/" + branch).encode() in inventory.split(b"\0"):
        return {"branch": branch, "oid": oid, "state": "retain", "reason": "worktree in use"}
    try:
        git(controller, "merge-base", "--is-ancestor", oid, "refs/heads/" + default)
    except CleanupRefused:
        return {"branch": branch, "oid": oid, "state": "retain", "reason": "merge not established"}
    return {"branch": branch, "oid": oid, "state": "eligible", "reason": "merged unused local branch"}
