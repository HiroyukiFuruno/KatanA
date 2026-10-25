#!/usr/bin/env python3
"""Admit a published release before executing explicit branch cleanup."""

import argparse
from pathlib import Path
import subprocess
import sys

from cleanup_execution import CleanupExecutor
from cleanup_receipt import CleanupReceipt
from cleanup_prune import prune_empty_registry, verify_prune_scope
from cleanup_session import CleanupSession
from release_cleanup_audit import CANONICAL_OID, CleanupRefused, require_repository_scope
from release_cleanup_publication import published_audit


def run_cleanup(root: Path, controller: Path, repository: str, version: str,
                branch: str, expected: str, *, published: str,
                include_remote: bool = False, merge_commit: str | None = None,
                pull_request: int | None = None) -> dict[str, str]:
    require_repository_scope(root, controller)
    if CANONICAL_OID.fullmatch(published) is None:
        raise CleanupRefused("cleanup publication requires a full commit identity")
    request = {"repository": repository, "version": version, "branch": branch,
               "expected": expected, "published": published, "include_remote": include_remote,
               "merge_commit": merge_commit, "pull_request": pull_request}
    with CleanupReceipt(controller, request) as receipt, CleanupSession(controller) as session:
        admitted = published_audit(root, controller, repository, version, branch, expected,
                                   allow_absent=include_remote, publication_oid=published,
                                   merge_commit=merge_commit, pull_request=pull_request)
        receipt.record("publication", "verified", admitted)
        if admitted["state"] != "eligible":
            raise CleanupRefused("cleanup target retained: " + admitted["reason"])
        verify_prune_scope(controller)
        executor = CleanupExecutor(controller, repository, admitted["default_branch"], branch, expected,
                                   merge_commit=merge_commit, pull_request=pull_request)
        executor.prepare_hooks(include_remote=include_remote)
        result = {"local": "not-started", "remote": "not-requested"}
        if include_remote:
            result["remote"] = executor.delete_remote(session)
            receipt.record("remote", result["remote"], branch)
        result["local"] = executor.delete_local(session)
        receipt.record("local", result["local"], branch)
        prune_empty_registry(controller)
        receipt.record("worktree-prune", "completed", "empty linked-worktree registry")
        return result


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--controller", required=True, type=Path)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--branch", required=True)
    parser.add_argument("--expected", required=True)
    parser.add_argument("--published", required=True)
    parser.add_argument("--merge-commit")
    parser.add_argument("--pull-request", type=int)
    parser.add_argument("--remote", action="store_true", help="also delete the explicit remote branch")
    options = parser.parse_args(arguments)
    try:
        result = run_cleanup(options.root, options.controller, options.repository, options.version,
                             options.branch, options.expected, published=options.published,
                             include_remote=options.remote, merge_commit=options.merge_commit,
                             pull_request=options.pull_request)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print(f"release cleanup refused: {error}", file=sys.stderr)
        return 2
    print("cleanup result: " + str(result))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
