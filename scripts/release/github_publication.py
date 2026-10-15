"""Small, fixed-host GitHub publication checks."""

from __future__ import annotations

import json
import subprocess
import time
from urllib.parse import quote, unquote, urlsplit

MAX_RESPONSE_BYTES = 8 * 1024 * 1024


def _gh_json(path: str, end: float) -> dict[str, object]:
    if time.monotonic() >= end:
        raise ValueError("publication deadline exceeded")
    try:
        result = subprocess.run(
            ["gh", "api", "--hostname", "github.com", "--include", "--method", "GET", path],
            check=True, capture_output=True, timeout=max(0.1, end - time.monotonic()),
        )
    except (OSError, subprocess.SubprocessError) as error:
        raise ValueError("GitHub API lookup failed") from error
    raw = result.stdout
    if len(raw) > MAX_RESPONSE_BYTES:
        raise ValueError("GitHub response exceeds size limit")
    separator = raw.find(b"\r\n\r\n")
    separator = separator if separator >= 0 else raw.find(b"\n\n")
    if separator < 0:
        raise ValueError("GitHub response has no body")
    try:
        status = int(raw.splitlines()[0].split()[1])
        offset = 4 if raw[separator:separator + 4] == b"\r\n\r\n" else 2
        body = json.loads(raw[separator + offset:])
    except (ValueError, IndexError, json.JSONDecodeError) as error:
        raise ValueError("GitHub response is malformed") from error
    if status != 200 or not isinstance(body, dict):
        raise ValueError("GitHub publication is not public")
    return body


def check_repository(metadata: dict[str, object], repository: str) -> None:
    name = metadata.get("full_name")
    if not isinstance(name, str) or name.casefold() != repository.casefold() or metadata.get("private") is not False or metadata.get("visibility") != "public":
        raise ValueError("GitHub repository is not public or has mismatched identity")


def check_release(release: dict[str, object], repository: str, tag: str, version: str, release_url: str) -> None:
    if not isinstance(release.get("published_at"), str) or not release["published_at"].strip() or release.get("draft") is not False or release.get("prerelease") is not False:
        raise ValueError("GitHub release is not published")
    final_component = tag.rsplit("/", 1)[-1]
    if release.get("tag_name") != tag or final_component not in {version, f"v{version}"}:
        raise ValueError("GitHub release tag mismatch")
    actual = urlsplit(str(release.get("html_url", "")))
    expected = urlsplit(f"https://github.com/{repository}/releases/tag/{quote(tag, safe='')}")
    source = urlsplit(release_url)
    if actual.scheme != "https" or actual.netloc.casefold() != "github.com" or unquote(actual.path).casefold() != unquote(expected.path).casefold() or source.scheme != "https" or source.netloc.casefold() != "github.com" or unquote(source.path).casefold() != unquote(expected.path).casefold() or source.query or source.fragment:
        raise ValueError("GitHub release identity mismatch")


def fetch_and_check(repository: str, tag: str, version: str, release_url: str, end: float) -> None:
    encoded_repo = quote(repository, safe="/")
    metadata = _gh_json(f"/repos/{encoded_repo}", end)
    check_repository(metadata, repository)
    release = _gh_json(f"/repos/{encoded_repo}/releases/tags/{quote(tag, safe='')}", end)
    check_release(release, repository, tag, version, release_url)
