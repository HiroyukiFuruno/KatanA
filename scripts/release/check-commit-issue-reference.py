#!/usr/bin/env python3
"""Require a commit message to reference an Issue in the selected repository."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from issue_reference_syntax import (
    ISSUE_URL_PATTERN, LOCAL_ISSUE_PATTERN, REPOSITORY_PATTERN,
    TRAILING_URL_PUNCTUATION, URL_PATTERN, has_issue_reference,
    issue_references, parse_repository,
)


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", required=True, help="対象repositoryのOWNER/NAME")
    parser.add_argument("--verify-live", action="store_true", help="GitHub上の真正Issueを実照合する")
    parser.add_argument("commit_message_file", type=Path, help="git commit-msg hookから渡されるファイル")
    options = parser.parse_args(arguments)

    try:
        message = options.commit_message_file.read_text(encoding="utf-8")
        references = issue_references(message, options.repository)
        valid = bool(references)
        if valid and options.verify_live:
            from issue_reference_live import LiveIssueClient

            valid = LiveIssueClient(options.repository).validate(references)
    except (OSError, UnicodeError, ValueError) as error:
        print(f"commit Issue reference check failed: {error}", file=sys.stderr)
        return 2

    if not valid:
        if options.verify_live:
            print(f"No genuine GitHub Issue was verified in {options.repository}.", file=sys.stderr)
            return 1
        print(
            f"Commit message must reference an Issue in {options.repository} "
            "using #<number> or its GitHub /issues/<number> URL.",
            file=sys.stderr,
        )
        return 1

    if options.verify_live:
        print(f"Verified a genuine GitHub Issue in {options.repository}.")
    else:
        print(f"Commit message references an Issue in {options.repository}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
