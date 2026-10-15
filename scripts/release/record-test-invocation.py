#!/usr/bin/env python3
"""Record real test execution without granting skip/reuse authority."""

import argparse
from pathlib import Path
import sys

from native_test_run_recording import run_recording


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--kind", choices=("test", "coverage"), required=True)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    arguments = parser.parse_args()
    command = arguments.command
    if command[:1] == ["--"]:
        command = command[1:]
    try:
        status = run_recording(Path.cwd().resolve(), arguments.output, command, arguments.kind)
    except (OSError, ValueError) as error:
        print(f"inventory recording failed: {error}", file=sys.stderr)
        return 2
    return status if status >= 0 else 128 - status


if __name__ == "__main__":
    sys.exit(main())
