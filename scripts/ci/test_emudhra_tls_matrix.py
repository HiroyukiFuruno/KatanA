"""eMudhra TLS 判定が通信障害を証明書拒否と誤分類しないことを確認する。"""

import socket
import ssl
import unittest
from urllib.error import HTTPError, URLError

from emudhra_tls_matrix import (
    EXPECTED_ENDPOINTS,
    classify_transport_error,
    outcome_matches_expectation,
)


class EmudhraTlsMatrixTests(unittest.TestCase):
    def test_endpoint_matrix_contains_control_and_four_revoked_paths(self) -> None:
        self.assertEqual(len(EXPECTED_ENDPOINTS), 5)
        self.assertEqual(EXPECTED_ENDPOINTS[0].expectation, "http_response")
        self.assertTrue(all(endpoint.expectation == "certificate_rejection"
                            for endpoint in EXPECTED_ENDPOINTS[1:]))
        self.assertEqual(
            [endpoint.url for endpoint in EXPECTED_ENDPOINTS[1:]],
            [
                "https://dv-g3b-emudhra.we-have-a-problem.com/",
                "https://dv-g3b-aaa-emudhra.we-have-a-problem.com/",
                "https://dv-g3b-noct-emudhra.we-have-a-problem.com/",
                "https://dv-g3b-aaa-noct-emudhra.we-have-a-problem.com/",
            ],
        )

    def test_only_certificate_verification_error_is_a_negative_endpoint_pass(self) -> None:
        certificate_error = ssl.SSLCertVerificationError(
            1, "certificate verify failed: unable to get local issuer certificate")
        self.assertEqual(classify_transport_error(certificate_error),
                         "certificate_verification_failure")
        self.assertEqual(classify_transport_error(URLError(certificate_error)),
                         "certificate_verification_failure")
        self.assertTrue(outcome_matches_expectation("certificate_rejection",
                                                    "certificate_verification_failure"))
        for error in (
            URLError(socket.gaierror(socket.EAI_AGAIN, "temporary DNS failure")),
            URLError(TimeoutError("request timed out")),
            URLError(ConnectionRefusedError("connection refused")),
            ssl.SSLError("TLS handshake failed"),
        ):
            with self.subTest(error=type(error).__name__):
                outcome = classify_transport_error(error)
                self.assertFalse(outcome_matches_expectation("certificate_rejection", outcome))

    def test_http_status_is_not_misclassified_as_certificate_rejection(self) -> None:
        response = HTTPError("https://example.invalid/", 403, "forbidden", {}, None)
        outcome = classify_transport_error(response)
        response.close()
        self.assertEqual(outcome, "http_response")
        self.assertFalse(outcome_matches_expectation("certificate_rejection", outcome))
        self.assertTrue(outcome_matches_expectation("http_response", outcome))

    def test_control_requires_an_http_response(self) -> None:
        self.assertTrue(outcome_matches_expectation("http_response", "http_response"))
        self.assertFalse(outcome_matches_expectation("http_response",
                                                     "certificate_verification_failure"))
        self.assertFalse(outcome_matches_expectation("http_response", "network_failure"))


if __name__ == "__main__":
    unittest.main()
