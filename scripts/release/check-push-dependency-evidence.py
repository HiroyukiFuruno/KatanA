#!/usr/bin/env python3
"""Check real pushed commits, Issue bodies, dependency trees, and publication."""

import argparse
import os
from pathlib import Path
import subprocess
import sys

from downstream_git_binding import _git
from cleanup_existing_hook import REMOTE_DIGEST_ENV, REMOTE_INTENT_ENV
from cleanup_remote_push import RemoteCleanupPush
from downstream_push_admission import DownstreamPushAdmission
from push_issue_selection import lookup_selected_issues, select_commit_issue_references
from push_ref_updates import MAX_PUSH_INPUT_BYTES, parse_push_updates
from push_repository_context import PushRepositoryContext


def _arguments(arguments):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("remote")
    parser.add_argument("remote_url")
    return parser.parse_args(arguments)


def _admit(root, context, updates, verified_source):
    selection = select_commit_issue_references(root, updates, context.repository,
                                              default_ref=context.default_ref, new_ref_base=context.new_ref_base)
    issues = lookup_selected_issues(selection, context.repository)
    by_branch = dict(selection.by_branch)
    for update in updates:
        if update.deleted or not update.remote_ref.startswith("refs/heads/") or update.remote_ref == context.default_ref:
            continue
        if verified_source is not None and update.local_oid != verified_source:
            raise ValueError("existing-hook source does not match the pushed branch source")
        baseline = context.new_ref_base if update.created else update.remote_oid
        payloads = {item["number"]: item for commit in by_branch[update.remote_ref] for item in issues[commit]}
        DownstreamPushAdmission(root, context.repository, update, baseline, list(payloads.values()), verified_source).verify()


def _verify_source(root, verified_source):
    if verified_source is None:
        return
    if _git(root, ["rev-parse", "HEAD"]).decode().strip() != verified_source:
        raise ValueError("existing-hook verification source changed before admission")
    if _git(root, ["status", "--porcelain", "--untracked-files=no"]).strip():
        raise ValueError("tracked source does not match existing-hook verification")


def _empty_push():
    if any(os.environ.get(key) is not None for key in (REMOTE_INTENT_ENV, REMOTE_DIGEST_ENV)):
        raise ValueError("remote cleanup requires a native deletion")
    return 0


def _verify_remote_cleanup(options, context, raw):
    intent = os.environ.get(REMOTE_INTENT_ENV)
    RemoteCleanupPush(context.root, context.repository, context.default_ref,
                      options.remote, options.remote_url).verify(
                          raw, Path(intent) if intent is not None else None,
                          os.environ.get(REMOTE_DIGEST_ENV),
                      )


def main(arguments=None, *, verified_source=None, payload=None):
    options = _arguments(arguments)
    try:
        raw = sys.stdin.buffer.read(MAX_PUSH_INPUT_BYTES + 1) if payload is None else payload
        updates = parse_push_updates(raw)
        if not updates:
            return _empty_push()
        root = options.root.resolve()
        context = PushRepositoryContext(root, options.repository, options.remote_url,
                                        created_refs=tuple(update.remote_ref for update in updates
                                                           if update.created and update.remote_ref.startswith("refs/heads/")))
        _verify_source(root, verified_source)
        _admit(root, context, updates, verified_source)
        _verify_remote_cleanup(options, context, raw)
        _verify_source(root, verified_source)
        return 0
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(f"push dependency evidence rejected: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
