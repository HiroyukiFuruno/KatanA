#!/usr/bin/env python3
"""Run a command with a disposable directory in KATANA_SCRATCH_DIR."""

import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

from cargo_target_lease import run_process_group


def remove_scratch(directory: Path, device: int, inode: int) -> None:
    entry = directory.lstat()
    if directory.resolve() != directory or (entry.st_dev, entry.st_ino) != (device, inode):
        raise RuntimeError("scratch directory identity changed")
    # WHY: 同じグループに属さないプロセスからの利用も確認してから削除する。
    lsof = "/usr/sbin/lsof" if sys.platform == "darwin" else shutil.which("lsof")
    if lsof is None:
        raise RuntimeError("lsof is required to verify scratch usage")
    check = subprocess.run([lsof, "-t", "+D", str(directory)], capture_output=True, text=True)
    if check.returncode not in {0, 1} or check.stderr or check.stdout:
        raise RuntimeError("scratch directory may still be in use")
    shutil.rmtree(directory)


def run(command: list[str]) -> int:
    directory = Path(tempfile.mkdtemp(prefix="katana-scratch-")).resolve()
    entry = directory.stat()
    environment = os.environ.copy() | {"KATANA_SCRATCH_DIR": str(directory)}
    try:
        status = run_process_group(command, Path.cwd(), environment)
    except SystemExit as error:
        status = int(error.code)
    except FileNotFoundError as error:
        print(f"Could not start scratch command: {error}", file=sys.stderr)
        status = 127
    except OSError as error:
        print(f"Could not verify scratch command exit, retained {directory}: {error}", file=sys.stderr)
        return 1

    status = 128 - status if status < 0 else status
    if os.name != "posix":
        print(f"Scratch retained because descendant exit is unverified: {directory}", file=sys.stderr)
        return status
    try:
        remove_scratch(directory, entry.st_dev, entry.st_ino)
    except (OSError, RuntimeError) as error:
        print(f"Scratch cleanup refused for {directory}: {error}", file=sys.stderr)
        return status or 1
    return status


if __name__ == "__main__":
    arguments = sys.argv[1:]
    if arguments[:1] == ["--"]:
        arguments = arguments[1:]
    if not arguments:
        raise SystemExit("usage: with-scratch.py -- command [args ...]")
    raise SystemExit(run(arguments))
