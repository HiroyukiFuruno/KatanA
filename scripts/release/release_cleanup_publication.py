"""Public-release admission precedes every cleanup synchronization mutation."""

from pathlib import Path
import time
from urllib.parse import quote

from github_publication import _gh_json, check_release, check_repository
from push_repository_context import remote_repository
from release_cleanup_audit import CleanupRefused, audit_branch, git, require_controller, synchronize_default
from release_cleanup_tag import require_published_commit

PUBLICATION_DEADLINE_SECONDS = 30


def require_no_url_rewrites(worktree: Path) -> None:
    records = git(worktree, "config", "--null", "--list").split(b"\0")
    for record in records:
        key = record.partition(b"\n")[0].lower()
        if key.startswith(b"url.") and key.endswith((b".insteadof", b".pushinsteadof")):
            # effective URLにも再度書換えが適用されるため、曖昧なtransportを保持する。
            raise CleanupRefused("cleanup refuses Git URL rewrite configuration")


def cleanup_destination(root: Path, controller: Path, repository: str) -> str:
    destination = None
    for worktree in (root, controller):
        require_no_url_rewrites(worktree)
        for mode in ([], ["--push"]):
            urls = git(worktree, "remote", "get-url", *mode, "--all", "origin").decode().splitlines()
            if len(urls) != 1 or remote_repository(urls[0]).casefold() != repository.casefold():
                raise CleanupRefused("cleanup fetch/push origin and publication repository differ")
            if worktree == controller and not mode:
                destination = urls[0]
    if destination is None:
        raise CleanupRefused("cleanup fetch destination is missing")
    return destination


def published_audit(root: Path, controller: Path, repository: str, version: str,
                    branch: str, expected_oid: str) -> dict[str, str]:
    destination = cleanup_destination(root, controller, repository)
    end = time.monotonic() + PUBLICATION_DEADLINE_SECONDS
    metadata = _gh_json(f"/repos/{quote(repository, safe='/')}", end)
    check_repository(metadata, repository)
    tag = "v" + version
    release = _gh_json(f"/repos/{quote(repository, safe='/')}/releases/tags/{quote(tag, safe='')}", end)
    check_release(release, repository, tag, version, f"https://github.com/{repository}/releases/tag/{tag}")
    default = metadata.get("default_branch")
    if not isinstance(default, str) or not default:
        raise CleanupRefused("published repository default branch is missing")
    require_controller(root, controller, default)
    # 入力誤りを同期前に拒否し、同期後の最新defaultで統合状態を再判定する。
    audit_branch(controller, default, branch, expected_oid)
    published_commit = require_published_commit(controller, destination, tag, expected_oid)
    # 検証後のorigin設定差し替えで別repoを取り込まないよう検証済URLを使う。
    default_oid = synchronize_default(root, controller, default, destination=destination)
    result = audit_branch(controller, default, branch, expected_oid)
    return {**result, "default_oid": default_oid, "published_commit": published_commit,
            "publication_url": str(release["html_url"])}
