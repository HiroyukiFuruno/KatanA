#!/usr/bin/env python3
"""Require a commit message to reference an Issue in the selected repository."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


REPOSITORY_PATTERN = re.compile(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+\Z")
ISSUE_URL_PATTERN = re.compile(
    r"https://github\.com/([A-Za-z0-9_.-]+)/([A-Za-z0-9_.-]+)/issues/([1-9][0-9]*)\Z",
    re.IGNORECASE,
)
URL_PATTERN = re.compile(r"https?://[^\s<>\"']+", re.IGNORECASE)
LOCAL_ISSUE_PATTERN = re.compile(r"(?<![A-Za-z0-9_/])#([1-9][0-9]*)(?![A-Za-z0-9_])")
TRAILING_URL_PUNCTUATION = ".,;:!?)]}"


def parse_repository(repository: str) -> tuple[str, str]:
    if not REPOSITORY_PATTERN.fullmatch(repository):
        raise ValueError("repository must be OWNER/NAME")
    owner, name = repository.split("/", maxsplit=1)
    return owner.casefold(), name.casefold()


def has_issue_reference(message: str, repository: str) -> bool:
    expected_owner, expected_name = parse_repository(repository)

    for candidate in URL_PATTERN.findall(message):
        url = candidate.rstrip(TRAILING_URL_PUNCTUATION)
        match = ISSUE_URL_PATTERN.fullmatch(url)
        if match and (match.group(1).casefold(), match.group(2).casefold()) == (
            expected_owner,
            expected_name,
        ):
            return True

    # URL内のfragment/queryに含まれる #番号を、本文のIssue参照と誤認しない。
    message_without_urls = URL_PATTERN.sub(" ", message)
    return LOCAL_ISSUE_PATTERN.search(message_without_urls) is not None


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", required=True, help="対象repositoryのOWNER/NAME")
    parser.add_argument("commit_message_file", type=Path, help="git commit-msg hookから渡されるファイル")
    options = parser.parse_args(arguments)

    try:
        message = options.commit_message_file.read_text(encoding="utf-8")
        valid = has_issue_reference(message, options.repository)
    except (OSError, UnicodeError, ValueError) as error:
        print(f"commit Issue reference check failed: {error}", file=sys.stderr)
        return 2

    if not valid:
        print(
            f"Commit message must reference an Issue in {options.repository} "
            "using #<number> or its GitHub /issues/<number> URL.",
            file=sys.stderr,
        )
        return 1

    print(f"Commit message references an Issue in {options.repository}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
