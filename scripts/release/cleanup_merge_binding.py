"""Verify server-recorded merge provenance separately from the branch tip."""

from cleanup_target import resolve_target
from release_cleanup_audit import CANONICAL_OID, CleanupRefused


def require_merge_binding(repository: str, version: str, published: str, branch: str,
                          expected: str, merge_commit: str | None,
                          pull_request: int | None) -> str | None:
    if merge_commit is None and pull_request is None:
        return None
    if (not isinstance(merge_commit, str) or CANONICAL_OID.fullmatch(merge_commit) is None
            or type(pull_request) is not int or pull_request <= 0):
        raise CleanupRefused("cleanup merge binding requires full merge OID and positive PR number")
    target = resolve_target(repository, version, published, pull_request)
    require_matching_merge(target, branch, expected, merge_commit, pull_request)
    return merge_commit


def require_matching_merge(target: dict, branch: str, expected: str,
                           merge_commit: str, pull_request: int) -> None:
    if target != {"branch": branch, "expected": expected,
                  "merge_commit": merge_commit, "pull_request": pull_request}:
        raise CleanupRefused("cleanup branch tip or merge identity differs from the merged PR")
