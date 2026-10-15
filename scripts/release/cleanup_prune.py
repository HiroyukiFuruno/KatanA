"""Never let repository-global prune consume unrelated linked-worktree state."""

import os
from pathlib import Path

from release_cleanup_audit import CleanupRefused, common_directory, git


def verify_prune_scope(controller: Path) -> None:
    registry = common_directory(controller) / "worktrees"
    if registry.is_symlink() or (registry.exists() and not registry.is_dir()):
        raise CleanupRefused("worktree registry is not a native directory; retaining state")
    if registry.is_dir() and any(registry.iterdir()):
        # Git pruneはpathを指定できないため、別worktreeのmetadataがあれば削除しない。
        raise CleanupRefused("linked worktree metadata requires explicit separate cleanup")
    inventory = git(controller, "worktree", "list", "--porcelain", "-z")
    roots = [os.fsdecode(item.removeprefix(b"worktree "))
             for item in inventory.split(b"\0") if item.startswith(b"worktree ")]
    if len(roots) != 1 or Path(roots[0]).resolve() != controller.resolve():
        raise CleanupRefused("prune requires a single audited default checkout")


def prune_empty_registry(controller: Path) -> None:
    verify_prune_scope(controller)
    git(controller, "worktree", "prune")
    verify_prune_scope(controller)
