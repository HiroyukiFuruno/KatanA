"""Bounded, read-only verification of genuine same-repository GitHub Issues."""

from __future__ import annotations

import json
import re
import subprocess
import time

MAX_CANDIDATES = 32
LOOKUP_DEADLINE_SECONDS = 30.0
MAX_RESPONSE_CHARACTERS = 1_048_576
HTTP_STATUS = re.compile(r"HTTP/\S+ ([0-9]{3})(?:\s|$)")
REPOSITORY = re.compile(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+\Z")


class IssueLookupError(ValueError):
    """Lookup failed without evidence sufficient to accept a commit reference."""


def remaining_time(deadline: float, now: float) -> float:
    remaining = deadline - now
    if remaining <= 0:
        raise IssueLookupError("GitHub Issue lookup deadline exceeded")
    return remaining


def candidate_numbers(numbers: list[int]) -> list[int]:
    if any(type(number) is not int or number <= 0 for number in numbers):
        raise IssueLookupError("Issue numbers must be positive integers")
    unique = list(dict.fromkeys(numbers))
    if len(unique) > MAX_CANDIDATES:
        raise IssueLookupError("Too many Issue lookup candidates")
    return unique


def response_payload(returncode: int, output: str) -> dict[str, object] | None:
    if len(output) > MAX_RESPONSE_CHARACTERS:
        raise IssueLookupError("GitHub Issue response exceeds the size limit")
    normalized = output.replace("\r\n", "\n")
    headers, separator, body = normalized.partition("\n\n")
    status = HTTP_STATUS.match(headers.split("\n", maxsplit=1)[0])
    if status is None or not separator:
        raise IssueLookupError("GitHub Issue lookup did not return an HTTP response")
    if status.group(1) == "404":
        return None
    if status.group(1) != "200" or returncode != 0:
        raise IssueLookupError("GitHub Issue lookup was not successful")
    try:
        payload = json.loads(body)
    except json.JSONDecodeError as error:
        raise IssueLookupError("GitHub Issue response is not valid JSON") from error
    if not isinstance(payload, dict):
        raise IssueLookupError("GitHub Issue response must be an object")
    return payload


def is_genuine_issue(payload: dict[str, object], repository: str, number: int) -> bool:
    expected = repository.casefold()
    if type(payload.get("number")) is not int or payload["number"] != number:
        raise IssueLookupError("GitHub Issue number does not match the request")
    repository_url = payload.get("repository_url")
    if not isinstance(repository_url, str) or repository_url.casefold() != (
        f"https://api.github.com/repos/{expected}"
    ):
        raise IssueLookupError("GitHub Issue repository does not match the request")
    # WHY: PRもissues endpointで返るため、正規pull URLをIssueとして誤受理しない。
    if "pull_request" in payload:
        return False
    html_url = payload.get("html_url")
    if not isinstance(html_url, str) or html_url.casefold() != (
        f"https://github.com/{expected}/issues/{number}"
    ):
        raise IssueLookupError("GitHub Issue URL does not match the request")
    if payload.get("state") not in ("open", "closed"):
        raise IssueLookupError("GitHub Issue state is not recognized")
    return True


class LiveIssueClient:
    def __init__(self, repository: str) -> None:
        if not REPOSITORY.fullmatch(repository):
            raise IssueLookupError("repository must be OWNER/NAME")
        self.repository = repository

    def validate(self, numbers: list[int]) -> bool:
        candidates = candidate_numbers(numbers)
        deadline = time.monotonic() + LOOKUP_DEADLINE_SECONDS
        for number in candidates:
            timeout = remaining_time(deadline, time.monotonic())
            payload = self._lookup(number, timeout)
            if payload is not None and is_genuine_issue(payload, self.repository, number):
                return True
        return False

    def genuine_payloads(self, numbers: list[int]) -> dict[int, dict[str, object]]:
        """Return canonical genuine Issue payloads, omitting missing/non-Issues."""
        candidates = candidate_numbers(numbers)
        deadline = time.monotonic() + LOOKUP_DEADLINE_SECONDS
        payloads: dict[int, dict[str, object]] = {}
        for number in candidates:
            timeout = remaining_time(deadline, time.monotonic())
            payload = self._lookup(number, timeout)
            if payload is not None and is_genuine_issue(payload, self.repository, number):
                # WHY: PR/404は候補から除外し、identity不一致は呼び出し元へ伝播する。
                payloads[number] = payload
        return payloads

    def _lookup(self, number: int, timeout: float) -> dict[str, object] | None:
        # WHY: 環境のGH_HOSTで取得先を変更せず、Issue本文や認証診断を実行・表示しない。
        command = [
            "gh", "api", "--hostname", "github.com", "--method", "GET", "--include",
            f"repos/{self.repository}/issues/{number}",
        ]
        try:
            result = subprocess.run(
                command, capture_output=True, text=True, check=False, timeout=timeout,
            )
        except (OSError, subprocess.TimeoutExpired) as error:
            raise IssueLookupError("GitHub Issue lookup could not complete") from error
        return response_payload(result.returncode, result.stdout)
