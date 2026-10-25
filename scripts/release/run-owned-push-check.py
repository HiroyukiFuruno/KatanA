#!/usr/bin/env python3
"""Execute the existing fixed quality command once and issue a fresh receipt."""

from pathlib import Path
import subprocess
import sys

from owner_verification import run_owned_check


def main() -> int:
    try:
        return run_owned_check(Path.cwd())
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(f"owned push check failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
