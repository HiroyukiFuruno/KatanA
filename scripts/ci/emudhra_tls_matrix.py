"""pip標準のTLS検証でeMudhra正常対照と失効経路を検査する。"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import hashlib
import json
import os
from pathlib import Path
import socket
import ssl
import sys
from typing import Any
from urllib.error import HTTPError, URLError
from urllib.request import HTTPSHandler, HTTPRedirectHandler, build_opener


@dataclass(frozen=True)
class Endpoint:
    name: str
    url: str
    expectation: str


EXPECTED_ENDPOINTS = (
    Endpoint("control", "https://emudhra.we-have-a-problem.com/", "http_response"),
    Endpoint("g3", "https://dv-g3b-emudhra.we-have-a-problem.com/",
             "certificate_rejection"),
    Endpoint("aaa", "https://dv-g3b-aaa-emudhra.we-have-a-problem.com/",
             "certificate_rejection"),
    Endpoint("noct", "https://dv-g3b-noct-emudhra.we-have-a-problem.com/",
             "certificate_rejection"),
    Endpoint("aaa_noct", "https://dv-g3b-aaa-noct-emudhra.we-have-a-problem.com/",
             "certificate_rejection"),
)

PIP_TLS_CONFIG_KEYS = {
    "global.cert", "install.cert", "global.trusted-host", "install.trusted-host",
    "global.use-deprecated", "install.use-deprecated",
}
PIP_TLS_ENV_KEYS = ("PIP_CERT", "PIP_TRUSTED_HOST", "PIP_USE_DEPRECATED")


class NoRedirectHandler(HTTPRedirectHandler):
    def redirect_request(self, req: Any, fp: Any, code: int, msg: str,
                         headers: Any, newurl: str) -> None:
        return None


def pip_tls_overrides() -> tuple[list[str], list[str]]:
    from pip._internal.configuration import Configuration

    configuration = Configuration(isolated=False)
    configuration.load()
    config_keys = sorted(key for key, value in configuration.items()
                         if key.lower() in PIP_TLS_CONFIG_KEYS and value)
    environment_keys = sorted(key for key in PIP_TLS_ENV_KEYS if os.environ.get(key))
    return config_keys, environment_keys


def create_pip_default_context() -> ssl.SSLContext:
    from pip._internal.cli.index_command import _create_truststore_ssl_context

    config_keys, environment_keys = pip_tls_overrides()
    if config_keys or environment_keys:
        raise RuntimeError(
            "pip TLS overrides prevent a default-context check: "
            f"config_keys={config_keys}; environment_keys={environment_keys}")
    context = _create_truststore_ssl_context()
    if context is None:
        raise RuntimeError("pip did not provide its default truststore TLS context")
    if context.verify_mode != ssl.CERT_REQUIRED or not context.check_hostname:
        raise RuntimeError("pip TLS context does not require certificate and hostname verification")
    return context


def classify_transport_error(error: BaseException) -> str:
    if isinstance(error, HTTPError):
        return "http_response"
    reason = error.reason if isinstance(error, URLError) else error
    if isinstance(reason, ssl.SSLCertVerificationError):
        return "certificate_verification_failure"
    if isinstance(reason, ssl.SSLError):
        return "other_tls_failure"
    if isinstance(reason, (socket.gaierror, socket.timeout, TimeoutError,
                           ConnectionError, OSError)):
        return "network_failure"
    return "unexpected_failure"


def outcome_matches_expectation(expectation: str, outcome: str) -> bool:
    expected_outcomes = {
        "http_response": "http_response",
        "certificate_rejection": "certificate_verification_failure",
    }
    if expectation not in expected_outcomes:
        raise ValueError(f"unsupported endpoint expectation: {expectation}")
    return outcome == expected_outcomes[expectation]


def exception_detail(error: BaseException) -> dict[str, Any]:
    reason = error.reason if isinstance(error, URLError) else error
    detail: dict[str, Any] = {
        "error_type": type(reason).__name__,
        "error": str(reason),
    }
    if isinstance(reason, ssl.SSLCertVerificationError):
        detail["verify_code"] = reason.verify_code
        detail["verify_message"] = reason.verify_message
    return detail


def check_endpoint(opener: Any, endpoint: Endpoint, timeout: float) -> dict[str, Any]:
    try:
        with opener.open(endpoint.url, timeout=timeout) as response:
            outcome = "http_response"
            result = {"http_status": response.status}
    except HTTPError as error:
        error.close()
        outcome = "http_response"
        result = {"http_status": error.code}
    except (URLError, OSError, ssl.SSLError) as error:
        outcome = classify_transport_error(error)
        result = exception_detail(error)
    result.update({
        "name": endpoint.name,
        "url": endpoint.url,
        "expected": endpoint.expectation,
        "outcome": outcome,
        "passed": outcome_matches_expectation(endpoint.expectation, outcome),
    })
    return result


def build_report(timeout: float) -> dict[str, Any]:
    context = create_pip_default_context()
    opener = build_opener(HTTPSHandler(context=context), NoRedirectHandler())
    results = [check_endpoint(opener, endpoint, timeout) for endpoint in EXPECTED_ENDPOINTS]
    from pip import __version__ as pip_version
    from pip._vendor import certifi

    bundle = Path(certifi.where()).resolve(strict=True)
    proxy_keys = ("HTTPS_PROXY", "https_proxy", "ALL_PROXY", "all_proxy")
    return {
        "client": "stdlib-urllib with pip default truststore SSLContext",
        "pip_version": pip_version,
        "python_version": sys.version.split()[0],
        "openssl_version": ssl.OPENSSL_VERSION,
        "trust_context": type(context).__module__ + "." + type(context).__name__,
        "verification_required": context.verify_mode == ssl.CERT_REQUIRED,
        "hostname_check_required": context.check_hostname,
        "pip_certifi_bundle": str(bundle),
        "pip_certifi_bundle_sha256": hashlib.sha256(bundle.read_bytes()).hexdigest(),
        "proxy_environment_present": any(os.environ.get(key) for key in proxy_keys),
        "results": results,
        "passed": all(result["passed"] for result in results),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--timeout", type=float, default=15.0)
    options = parser.parse_args()
    if options.timeout <= 0:
        parser.error("--timeout must be positive")
    try:
        report = build_report(options.timeout)
    except Exception as error:
        print(json.dumps({"passed": False, "error_type": type(error).__name__,
                          "error": str(error)}, indent=2, sort_keys=True))
        return 2
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
