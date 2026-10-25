"""Shared syntax extraction; references alone are not genuine Issue proof."""

import re


REPOSITORY_PATTERN = re.compile(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+\Z")
ISSUE_URL_PATTERN = re.compile(
    r"https://github\.com/([A-Za-z0-9_.-]+)/([A-Za-z0-9_.-]+)/issues/([1-9][0-9]*)\Z",
    re.IGNORECASE,
)
URL_PATTERN = re.compile(r"https?://[^\s<>\"']+", re.IGNORECASE)
LOCAL_ISSUE_PATTERN = re.compile(r"(?<![A-Za-z0-9_/])#([1-9][0-9]*)(?![A-Za-z0-9_])")
TRAILING_URL_PUNCTUATION = ".,;:!?)]}"
MAX_ISSUE_NUMBER_DIGITS = 20


def _issue_number(text: str) -> int:
    if len(text) > MAX_ISSUE_NUMBER_DIGITS:
        raise ValueError("Issue reference number exceeds the parsing limit")
    return int(text)


def parse_repository(repository: str) -> tuple[str, str]:
    if not REPOSITORY_PATTERN.fullmatch(repository):
        raise ValueError("repository must be OWNER/NAME")
    owner, name = repository.split("/", maxsplit=1)
    return owner.casefold(), name.casefold()


def issue_references(message: str, repository: str) -> list[int]:
    expected_owner, expected_name = parse_repository(repository)
    references: list[tuple[int, int]] = []
    for candidate in URL_PATTERN.finditer(message):
        url = candidate.group().rstrip(TRAILING_URL_PUNCTUATION)
        match = ISSUE_URL_PATTERN.fullmatch(url)
        if match and (match.group(1).casefold(), match.group(2).casefold()) == (
            expected_owner, expected_name,
        ):
            references.append((candidate.start(), _issue_number(match.group(3))))
    # URLのfragment/query内の番号を、本文のIssue参照として誤受理しない。
    message_without_urls = URL_PATTERN.sub(lambda match: " " * len(match.group()), message)
    references.extend(
        (match.start(), _issue_number(match.group(1)))
        for match in LOCAL_ISSUE_PATTERN.finditer(message_without_urls)
    )
    return list(dict.fromkeys(number for _, number in sorted(references)))


def has_issue_reference(message: str, repository: str) -> bool:
    return bool(issue_references(message, repository))
