"""Pure response contracts; live GitHub acceptance is performed separately."""

from __future__ import annotations

import json
import unittest
from unittest.mock import ANY, patch

from issue_reference_live import (
    LOOKUP_DEADLINE_SECONDS, MAX_CANDIDATES, MAX_RESPONSE_CHARACTERS,
    IssueLookupError, LiveIssueClient, candidate_numbers, is_genuine_issue,
    remaining_time, response_payload,
)

REPOSITORY = "HiroyukiFuruno/KatanA"
ISSUE_NUMBER = 335


def issue_payload(**changes: object) -> dict[str, object]:
    payload: dict[str, object] = {
        "number": ISSUE_NUMBER,
        "html_url": f"https://github.com/{REPOSITORY}/issues/{ISSUE_NUMBER}",
        "repository_url": f"https://api.github.com/repos/{REPOSITORY}",
        "state": "open",
    }
    return payload | changes


class GenuineIssueContractTests(unittest.TestCase):
    def test_accepts_open_and_closed_issues(self) -> None:
        for state in ("open", "closed"):
            with self.subTest(state=state):
                self.assertTrue(is_genuine_issue(issue_payload(state=state), REPOSITORY, ISSUE_NUMBER))

    def test_rejects_pull_request_before_issue_url_validation(self) -> None:
        payload = issue_payload(html_url=f"https://github.com/{REPOSITORY}/pull/{ISSUE_NUMBER}", pull_request={})
        self.assertFalse(is_genuine_issue(payload, REPOSITORY, ISSUE_NUMBER))

    def test_rejects_null_pull_request_marker(self) -> None:
        self.assertFalse(is_genuine_issue(issue_payload(pull_request=None), REPOSITORY, ISSUE_NUMBER))

    def test_pull_request_candidate_does_not_hide_later_genuine_issue(self) -> None:
        payloads = [issue_payload(pull_request={}), issue_payload()]
        self.assertEqual([is_genuine_issue(p, REPOSITORY, ISSUE_NUMBER) for p in payloads], [False, True])

    def test_accepts_case_insensitive_repository_urls(self) -> None:
        payload = issue_payload(
            html_url=f"https://github.com/{REPOSITORY.lower()}/issues/{ISSUE_NUMBER}",
            repository_url=f"https://api.github.com/repos/{REPOSITORY.lower()}",
        )
        self.assertTrue(is_genuine_issue(payload, REPOSITORY, ISSUE_NUMBER))

    def test_rejects_boolean_or_mismatched_number(self) -> None:
        for number in (True, "335", 336, None):
            with self.subTest(number=number), self.assertRaises(IssueLookupError):
                is_genuine_issue(issue_payload(number=number), REPOSITORY, ISSUE_NUMBER)

    def test_rejects_repository_issue_url_and_state_mismatches(self) -> None:
        for change in (
            {"repository_url": "https://api.github.com/repos/other/repository"},
            {"html_url": f"https://github.com/{REPOSITORY}/issues/336"},
            {"html_url": f"https://github.com/{REPOSITORY}/issues/335?untrusted=1"},
            {"state": "unknown"},
        ):
            with self.subTest(change=change), self.assertRaises(IssueLookupError):
                is_genuine_issue(issue_payload(**change), REPOSITORY, ISSUE_NUMBER)


class HttpResponseContractTests(unittest.TestCase):
    def test_parses_successful_http_response_with_crlf(self) -> None:
        response = "HTTP/2.0 200 OK\r\nContent-Type: application/json\r\n\r\n" + json.dumps(issue_payload())
        self.assertEqual(response_payload(0, response), issue_payload())

    def test_not_found_is_a_negative_candidate(self) -> None:
        self.assertIsNone(response_payload(1, "HTTP/2.0 404 Not Found\n\n{}"))

    def test_not_found_does_not_hide_a_later_successful_response(self) -> None:
        negative = response_payload(1, "HTTP/2.0 404 Not Found\n\n{}")
        positive = response_payload(0, "HTTP/2.0 200 OK\n\n" + json.dumps(issue_payload()))
        self.assertIsNone(negative)
        self.assertTrue(is_genuine_issue(positive, REPOSITORY, ISSUE_NUMBER))

    def test_auth_rate_limit_and_server_failures_are_fatal(self) -> None:
        for status in (401, 403, 429, 500, 503):
            with self.subTest(status=status), self.assertRaises(IssueLookupError):
                response_payload(1, f"HTTP/2.0 {status} failure\n\n{{}}")

    def test_missing_headers_invalid_json_and_non_object_are_fatal(self) -> None:
        for output in ("", "raw secret diagnostic", "HTTP/2.0 200 OK\n\ninvalid", "HTTP/2.0 200 OK\n\n[]"):
            with self.subTest(output=output), self.assertRaises(IssueLookupError):
                response_payload(0, output)

    def test_unsuccessful_cli_exit_does_not_accept_http_success(self) -> None:
        with self.assertRaises(IssueLookupError):
            response_payload(1, "HTTP/2.0 200 OK\n\n" + json.dumps(issue_payload()))

    def test_oversized_response_is_rejected(self) -> None:
        with self.assertRaises(IssueLookupError):
            response_payload(0, "x" * (MAX_RESPONSE_CHARACTERS + 1))


class LookupBoundsContractTests(unittest.TestCase):
    def test_deduplicates_candidates_in_order(self) -> None:
        self.assertEqual(candidate_numbers([335, 312, 335]), [335, 312])

    def test_rejects_more_than_the_bounded_candidate_count(self) -> None:
        with self.assertRaises(IssueLookupError):
            candidate_numbers(list(range(1, MAX_CANDIDATES + 2)))

    def test_rejects_invalid_number_types_and_values(self) -> None:
        for value in (True, 0, -1, "335"):
            with self.subTest(value=value), self.assertRaises(IssueLookupError):
                candidate_numbers([value])

    def test_each_lookup_uses_only_remaining_deadline(self) -> None:
        self.assertEqual(remaining_time(30.0, 29.5), 0.5)

    def test_expired_deadline_is_not_a_pass(self) -> None:
        for now in (30.0, 31.0):
            with self.subTest(now=now), self.assertRaises(IssueLookupError):
                remaining_time(30.0, now)


class GenuinePayloadContractTests(unittest.TestCase):
    def client(self) -> LiveIssueClient:
        return LiveIssueClient(REPOSITORY)

    def test_returns_payload_and_preserves_body(self) -> None:
        client = self.client()
        payload = issue_payload(body="dependency evidence")
        with patch.object(client, "_lookup", return_value=payload):
            result = client.genuine_payloads([ISSUE_NUMBER])
        self.assertEqual(result[ISSUE_NUMBER]["body"], "dependency evidence")

    def test_omits_missing_and_pull_request_payloads(self) -> None:
        client = self.client()
        pull_request = issue_payload(
            number=336,
            html_url=f"https://github.com/{REPOSITORY}/pull/336",
            pull_request={},
        )
        with patch.object(client, "_lookup", side_effect=[None, pull_request]):
            self.assertEqual(client.genuine_payloads([335, 336]), {})

    def test_rejects_mismatched_identity(self) -> None:
        client = self.client()
        with patch.object(client, "_lookup", return_value=issue_payload(number=336)), \
                self.assertRaises(IssueLookupError):
            client.genuine_payloads([ISSUE_NUMBER])

    def test_deduplicates_numbers_before_lookup(self) -> None:
        client = self.client()
        with patch.object(client, "_lookup", return_value=issue_payload()) as lookup:
            client.genuine_payloads([335, 335])
        lookup.assert_called_once_with(335, ANY)

    def test_uses_one_shared_deadline_for_all_candidates(self) -> None:
        client = self.client()
        with patch("issue_reference_live.time.monotonic", side_effect=[100.0, 101.0, 102.0]), patch.object(
            client, "_lookup", side_effect=[None, None]
        ) as lookup:
            client.genuine_payloads([335, 336])
        timeouts = [call.args[1] for call in lookup.call_args_list]
        self.assertEqual(timeouts, [LOOKUP_DEADLINE_SECONDS - 1.0,
                                    LOOKUP_DEADLINE_SECONDS - 2.0])

    def test_validate_keeps_early_success_short_circuit(self) -> None:
        client = self.client()
        with patch.object(
            client, "_lookup", side_effect=[issue_payload(), IssueLookupError("must not run")]
        ) as lookup:
            self.assertTrue(client.validate([335, 336]))
        lookup.assert_called_once()

    def test_invalid_candidates_never_reach_transport(self) -> None:
        for numbers in ([True], [0], list(range(1, MAX_CANDIDATES + 2))):
            client = self.client()
            with patch.object(client, "_lookup") as lookup, self.assertRaises(IssueLookupError):
                client.genuine_payloads(numbers)
            lookup.assert_not_called()

    def test_exhausted_shared_deadline_stops_before_next_lookup(self) -> None:
        client = self.client()
        with patch("issue_reference_live.time.monotonic", side_effect=[100.0, 101.0, 130.0]), \
                patch.object(client, "_lookup", return_value=None) as lookup, \
                self.assertRaises(IssueLookupError):
            client.genuine_payloads([335, 336])
        lookup.assert_called_once()


if __name__ == "__main__":
    unittest.main()
