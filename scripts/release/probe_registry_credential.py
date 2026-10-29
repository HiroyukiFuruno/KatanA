"""既存CI credentialを固定GitHub APIで読取り、機密を含まない診断だけを残す。"""
import argparse
import http.client
import json
import os
from pathlib import Path
import urllib.error
import urllib.request

REPOSITORY = "HiroyukiFuruno/homebrew-katana"
API_ROOT = "https://api.github.com"
MAX_RESPONSE_BYTES = 65536
REQUEST_TIMEOUT_SECONDS = 30
ENDPOINTS = ("/user", f"/repos/{REPOSITORY}")
ASCII_VISIBLE_START = 33
ASCII_VISIBLE_END = 126
MESSAGE_CLASSES = {
    "Bad credentials": "bad_credentials",
    "Resource not accessible by personal access token": "token_access_denied",
    "Resource not accessible by integration": "integration_access_denied",
    "Not Found": "not_found",
    "Requires authentication": "authentication_required",
}


class NoCredentialRedirects(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, response, code, message, headers, new_url):
        return None


def valid_credential(token):
    return bool(token) and token.isascii() and all(
        ASCII_VISIBLE_START <= ord(character) <= ASCII_VISIBLE_END for character in token
    )


def summarize(endpoint, status, body):
    body = body if isinstance(body, dict) else {}
    report = {"endpoint": endpoint, "http_status": status}
    message = body.get("message")
    message = message if isinstance(message, str) else None
    report["message_class"] = MESSAGE_CLASSES.get(message, "none" if status == 200 else "other")
    if status == 200 and endpoint == "/user":
        report["expected_identity_matches"] = body.get("login") == "HiroyukiFuruno"
    if status == 200 and endpoint == ENDPOINTS[1]:
        report["repository_matches"] = body.get("full_name") == REPOSITORY
        permissions = body.get("permissions", {})
        if isinstance(permissions, dict):
            report["repository_role_push"] = permissions.get("push") is True
    return report


def read_report(endpoint, response):
    with response:
        try:
            raw = response.read(MAX_RESPONSE_BYTES + 1)
        except (urllib.error.URLError, TimeoutError, OSError, http.client.HTTPException):
            return {"endpoint": endpoint, "http_status": response.status,
                    "message_class": "transport_error"}
        if len(raw) > MAX_RESPONSE_BYTES:
            return {"endpoint": endpoint, "http_status": response.status,
                    "message_class": "response_too_large"}
        try:
            body = json.loads(raw)
        except (ValueError, UnicodeError):
            body = {}
        return summarize(endpoint, response.status, body)


def probe(endpoint, token):
    if endpoint not in ENDPOINTS:
        raise ValueError("registry probe endpoint is not allowed")
    if not valid_credential(token):
        return {"endpoint": endpoint, "http_status": None, "message_class": "invalid_credential_format"}
    try:
        request = urllib.request.Request(API_ROOT + endpoint, headers={
            "Authorization": "Bearer " + token,
            "Accept": "application/vnd.github+json",
            "User-Agent": "KatanA-registry-credential-diagnostic",
        })
        opener = urllib.request.build_opener(NoCredentialRedirects)
        response = opener.open(request, timeout=REQUEST_TIMEOUT_SECONDS)
    except urllib.error.HTTPError as error:
        response = error
    except ValueError:
        return {"endpoint": endpoint, "http_status": None, "message_class": "request_error"}
    except (urllib.error.URLError, TimeoutError, OSError, http.client.HTTPException):
        return {"endpoint": endpoint, "http_status": None, "message_class": "transport_error"}
    return read_report(endpoint, response)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    token = os.environ.get("HOMEBREW_TAP_TOKEN", "")
    # WHY: 公開repoのGET成功だけでは、同じcredentialのPUT権限を証明できない。
    report = {"repository": REPOSITORY, "write_access_verified": False,
              "credential_configured": bool(token), "probes": []}
    if token:
        report["probes"] = [probe(endpoint, token) for endpoint in ENDPOINTS]
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))
    return 0 if token and report["probes"][1].get("repository_matches") else 1


if __name__ == "__main__":
    raise SystemExit(main())
