"""Resolve a unique merged release PR without equating tag SHA and branch tip."""

import json
import re
import subprocess
import time
from urllib.parse import quote

from github_publication import MAX_RESPONSE_BYTES, _gh_json, check_repository
from push_repository_context import remote_repository
from release_cleanup_audit import CANONICAL_OID, CleanupRefused

LOOKUP_SECONDS = 30
MAX_ASSOCIATED_PRS = 1000
RELEASE_BRANCH = re.compile(r"release/v([0-9]+\.[0-9]+\.[0-9]+)(-[A-Za-z0-9][A-Za-z0-9._-]*)?")


def release_branch_version(branch: object) -> str | None:
    # WHY: 既存のbranch parserと同じ完全一致grammarでsuffixを版数から分離する。
    match = RELEASE_BRANCH.fullmatch(branch) if isinstance(branch, str) else None
    return match[1] if match is not None else None


def _candidate(value: object, repository: str, default: str, version: str) -> dict | None:
    if not isinstance(value, dict):
        raise CleanupRefused("cleanup PR metadata is not an object")
    head, base = value.get("head"), value.get("base")
    if not isinstance(head, dict) or not isinstance(base, dict):
        raise CleanupRefused("cleanup PR head/base metadata is missing")
    branch = head.get("ref")
    branch_version = release_branch_version(branch)
    if branch_version is None or branch_version != version or base.get("ref") != default or value.get("state") != "closed":
        return None
    merged = value.get("merged_at")
    if not isinstance(merged, str) or not merged.strip():
        return None
    for side in (head, base):
        owner = side.get("repo")
        if not isinstance(owner, dict) or str(owner.get("full_name", "")).casefold() != repository.casefold():
            raise CleanupRefused("cleanup PR belongs to another repository or fork")
    number, expected, merge = value.get("number"), head.get("sha"), value.get("merge_commit_sha")
    if type(number) is not int or number <= 0 or not isinstance(expected, str) or not isinstance(merge, str):
        raise CleanupRefused("cleanup PR identities are malformed")
    if CANONICAL_OID.fullmatch(expected) is None or CANONICAL_OID.fullmatch(merge) is None:
        raise CleanupRefused("cleanup PR requires full commit identities")
    return {"branch": branch, "expected": expected, "pull_request": number, "merge_commit": merge}


def select_target(values: list, repository: str, default: str, version: str) -> dict:
    if len(values) > MAX_ASSOCIATED_PRS:
        raise CleanupRefused("cleanup PR candidate limit exceeded")
    selected = [_candidate(value, repository, default, version) for value in values]
    matches = [value for value in selected if value is not None]
    if len(matches) != 1:
        raise CleanupRefused("cleanup requires exactly one merged release PR")
    return matches[0]


def _associated(repository: str, published: str, end: float) -> list:
    if time.monotonic() >= end:
        raise CleanupRefused("cleanup PR lookup deadline exceeded")
    endpoint = f"repos/{quote(repository, safe='/')}/commits/{published}/pulls?per_page=100"
    result = subprocess.run(["gh", "api", "--hostname", "github.com", "--method", "GET",
                             "--paginate", "--slurp", endpoint],
                            capture_output=True, check=True, timeout=max(0.1, end - time.monotonic()))
    if len(result.stdout) > MAX_RESPONSE_BYTES:
        raise CleanupRefused("cleanup PR response exceeds its bound")
    pages = json.loads(result.stdout)
    if not isinstance(pages, list) or any(not isinstance(page, list) for page in pages):
        raise CleanupRefused("cleanup PR pagination is malformed")
    return [value for page in pages for value in page]


def resolve_target(repository: str, version: str, published: str, number: int | None) -> dict:
    remote_repository("https://github.com/" + repository + ".git")
    if CANONICAL_OID.fullmatch(published) is None:
        raise CleanupRefused("published cleanup source requires a full commit identity")
    end = time.monotonic() + LOOKUP_SECONDS
    metadata = _gh_json("/repos/" + quote(repository, safe="/"), end)
    check_repository(metadata, repository)
    default = metadata.get("default_branch")
    if not isinstance(default, str) or not default:
        raise CleanupRefused("cleanup repository default branch is missing")
    if number is not None:
        if type(number) is not int or number <= 0:
            raise CleanupRefused("cleanup PR number is invalid")
        values = [_gh_json(f"/repos/{quote(repository, safe='/')}/pulls/{number}", end)]
        if values[0].get("number") != number:
            raise CleanupRefused("cleanup PR response does not match the requested number")
    else:
        values = _associated(repository, published, end)
    return select_target(values, repository, default, version)
