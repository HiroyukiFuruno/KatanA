#!/usr/bin/env python3
"""Read live run history before accepting the latest three-OS results."""

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile

from ci_run_admission import verify_identity, verify_run

MAX_RESPONSE_BYTES = 8 * 1024 * 1024


def _unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("GitHub response has duplicate JSON keys")
        result[key] = value
    return result


def _read(command: list[str]):
    with tempfile.TemporaryFile() as output:
        subprocess.run(command, stdout=output, check=True, timeout=30)
        output.seek(0)
        raw = output.read(MAX_RESPONSE_BYTES + 1)
    if len(raw) > MAX_RESPONSE_BYTES:
        raise ValueError("CI response exceeds the evidence limit")
    return json.loads(raw, object_pairs_hook=_unique_object)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sha", required=True)
    parser.add_argument("--run-id", required=True, type=int)
    parser.add_argument("--repo", default=os.environ.get("GITHUB_REPOSITORY"))
    args = parser.parse_args()
    try:
        repository = args.repo or _read(["gh", "repo", "view", "--json", "nameWithOwner"])["nameWithOwner"]
        if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository):
            raise ValueError("CI repository name is invalid")
        endpoint = f"repos/{repository}/actions/runs/{args.run_id}"
        before = _read(["gh", "api", endpoint])
        attempt = verify_identity(before, args.sha, args.run_id, repository)
        pages = _read(["gh", "api", "--paginate", "--slurp",
                       f"{endpoint}/jobs?filter=all&per_page=100"])
        after = _read(["gh", "api", endpoint])
        verify_run(before, pages, after, args.sha, args.run_id, repository)
        print(f"CI run {args.run_id} snapshot attempt {attempt}: latest three-OS jobs succeeded for {args.sha}")
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        print(f"CI run admission failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
