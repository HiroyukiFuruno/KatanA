"""Synthetic classification contracts do not claim real CI credential access."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from probe_registry_credential import ENDPOINTS, REPOSITORY, NoCredentialRedirects, probe, summarize


class RegistryCredentialProbeTests(unittest.TestCase):
    def test_bad_credentials_are_classified_without_echoing_payload(self):
        secret = "credential-value-must-never-appear"
        result = summarize(ENDPOINTS[1], 401, {"message": "Bad credentials", "token": secret})
        self.assertEqual(result["http_status"], 401)
        self.assertEqual(result["message_class"], "bad_credentials")
        self.assertNotIn(secret, json.dumps(result))

    def test_unknown_error_message_cannot_echo_a_secret(self):
        secret = "credential-value-must-never-appear"
        result = summarize(ENDPOINTS[1], 403, {"message": secret})
        self.assertEqual(result["message_class"], "other")
        self.assertNotIn(secret, json.dumps(result))

    def test_read_metadata_reports_role_without_claiming_write_access(self):
        result = summarize(ENDPOINTS[1], 200, {
            "full_name": REPOSITORY, "permissions": {"push": True}, "unrelated": "private",
        })
        self.assertTrue(result["repository_matches"])
        self.assertTrue(result["repository_role_push"])
        self.assertNotIn("unrelated", result)
        self.assertNotIn("write_access_verified", result)

    def test_foreign_repository_and_identity_are_not_accepted(self):
        self.assertFalse(summarize(ENDPOINTS[1], 200, {"full_name": "other/repo"})["repository_matches"])
        self.assertFalse(summarize("/user", 200, {"login": "other"})["expected_identity_matches"])
        self.assertEqual(summarize(ENDPOINTS[1], 404, {"message": "Not Found"})["message_class"], "not_found")

    def test_arbitrary_endpoint_is_rejected_before_network_access(self):
        with self.assertRaises(ValueError):
            probe("https://other.invalid/token", "must-not-be-sent")

    def test_redirects_cannot_forward_the_credential(self):
        handler = NoCredentialRedirects()
        self.assertIsNone(handler.redirect_request(None, None, 302, "moved", {}, "https://other.invalid"))

    def test_invalid_header_credential_is_rejected_without_network(self):
        for token in ("secret\n", "secret\r", "secret\0", "secret with space", "secret\u00e9"):
            result = probe(ENDPOINTS[1], token)
            self.assertEqual(result["message_class"], "invalid_credential_format")
            self.assertNotIn(token, json.dumps(result))

    def test_real_cli_redacts_a_multiline_environment_credential(self):
        secret = "sensitive-header-value\nsecond-line"
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "report.json"
            command = [sys.executable, "-B", str(Path(__file__).with_name("probe_registry_credential.py")),
                       "--output", str(output)]
            result = subprocess.run(command, env=os.environ | {"HOMEBREW_TAP_TOKEN": secret},
                                    capture_output=True, check=False, timeout=10)
            self.assertEqual(result.returncode, 1)
            recorded = output.read_bytes() + result.stdout + result.stderr
            self.assertNotIn(b"sensitive-header-value", recorded)
            self.assertNotIn(b"second-line", recorded)
            self.assertIn(b"invalid_credential_format", recorded)

    def test_exception_boundary_redacts_backend_value_error_unit_fault(self):
        with patch("urllib.request.OpenerDirector.open", side_effect=ValueError("Bearer sensitive-value")):
            result = probe(ENDPOINTS[1], "valid-fixture-token")
        self.assertEqual(result["message_class"], "request_error")
        self.assertNotIn("sensitive-value", json.dumps(result))

    def test_malformed_response_cannot_escape_safe_classification(self):
        for body in ([], None, {"message": {"token": "secret"}}):
            result = summarize(ENDPOINTS[1], 403, body)
            self.assertEqual(result["message_class"], "other")
            self.assertNotIn("secret", json.dumps(result))

    def test_workflow_is_manual_owner_only_with_read_permissions(self):
        root = Path(__file__).resolve().parents[2]
        workflow = (root / ".github/workflows/registry-credential-diagnostics.yml").read_text()
        self.assertIn("workflow_dispatch:", workflow)
        self.assertNotIn("pull_request:", workflow)
        self.assertNotIn("push:", workflow)
        self.assertIn("contents: read", workflow)
        self.assertIn("github.actor == 'HiroyukiFuruno'", workflow)
        self.assertIn("github.repository == 'HiroyukiFuruno/KatanA'", workflow)
        self.assertIn("github.ref == 'refs/heads/master'", workflow)
        self.assertIn("if: always()", workflow)


if __name__ == "__main__":
    unittest.main()
