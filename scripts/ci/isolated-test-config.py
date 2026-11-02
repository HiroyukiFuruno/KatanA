#!/usr/bin/env python3
"""Run a test command with a fresh, process-scoped KatanA configuration."""

import os
from pathlib import Path
import shutil
import sys
import tempfile

from cargo_target_lease import run_process_group


def main() -> int:
    command = sys.argv[1:]
    if command and command[0] == "--":
        command = command[1:]
    if not command:
        print("test command is required", file=sys.stderr)
        return 2
    directory = tempfile.mkdtemp(prefix="katana-test-config-")
    environment = os.environ.copy()
    environment["KATANA_CONFIG_DIR"] = str(Path(directory).resolve())
    settled = False
    try:
        try:
            result = run_process_group(command, Path.cwd(), environment)
        except SystemExit as error:
            result = int(error.code)
        except FileNotFoundError as error:
            print(f"Could not start test command: {error}", file=sys.stderr)
            result = 127
        settled = True
        return 128 - result if result < 0 else result
    finally:
        if settled and os.name == "posix":
            shutil.rmtree(directory)
        else:
            print(f"Test configuration retained because descendant exit is unverified: {directory}", file=sys.stderr)


if __name__ == "__main__":
    raise SystemExit(main())
