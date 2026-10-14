"""Batch Cargo's native semver checks; never approximate requirements in Python."""

import json
from pathlib import Path
import subprocess

from downstream_dependency_records import DependencySnapshotError

TOOL_ROOT = Path(__file__).resolve().parents[2]
MAX_VERSION_REQUEST_BYTES = 1_048_576
MAX_VERSION_REQUESTS = 4096
VERSION_CONTRACT_SECONDS = 30


def validate_versions(requests):
    if not requests:
        return
    payload = json.dumps(requests).encode("utf-8")
    if len(payload) > MAX_VERSION_REQUEST_BYTES or len(requests) > MAX_VERSION_REQUESTS:
        raise DependencySnapshotError("version requests exceed the native contract limit")
    try:
        result = subprocess.run(
            ["cargo", "run", "--locked", "--offline", "--quiet", "-p", "katana-linter",
             "--bin", "dependency-contract"], cwd=TOOL_ROOT, input=payload,
            capture_output=True, timeout=VERSION_CONTRACT_SECONDS,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise DependencySnapshotError("native Cargo version contract could not complete") from error
    if result.returncode != 0:
        raise DependencySnapshotError("native Cargo version contract rejected its input")
    try:
        matches = json.loads(result.stdout)
    except (ValueError, UnicodeError) as error:
        raise DependencySnapshotError("native Cargo version contract response is invalid") from error
    if not isinstance(matches, list) or len(matches) != len(requests) or any(type(item) is not bool for item in matches):
        raise DependencySnapshotError("native Cargo version contract response does not match its requests")
    if not all(matches):
        raise DependencySnapshotError("path dependency version does not match the registered package")
