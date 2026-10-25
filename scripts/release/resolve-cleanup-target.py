#!/usr/bin/env python3
"""Print the live unique merged PR identity used by post-publication cleanup."""

import argparse
import json
import subprocess
import sys

from cleanup_target import resolve_target


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--published", required=True)
    parser.add_argument("--pull-request", type=int)
    options = parser.parse_args(arguments)
    try:
        target = resolve_target(options.repository, options.version, options.published, options.pull_request)
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(f"cleanup target refused: {error}", file=sys.stderr)
        return 2
    print(json.dumps(target, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
