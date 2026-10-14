#!/usr/bin/env python3
"""Run the installed pre-push owner before the Issue 335 checker."""

from __future__ import annotations

import argparse
import importlib.util
import os
import subprocess
import sys
from pathlib import Path

from owner_verification import VerificationSession

MAX_STDIN_BYTES = 1024 * 1024
BACKUP_NAME = "pre-push.katana-existing"
MARKER = "KATANA_PRE_PUSH_DISPATCHER_V1"


def active_root() -> Path:
    result = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        check=True,
        capture_output=True,
        text=True,
    )
    return Path(result.stdout.strip()).resolve()


def hooks_dir() -> Path:
    result = subprocess.run(
        ["git", "rev-parse", "--git-path", "hooks"],
        check=True,
        capture_output=True,
        text=True,
    )
    path = Path(result.stdout.strip())
    return path.resolve() if path.is_absolute() else (Path.cwd() / path).resolve()


def read_bounded_stdin() -> bytes:
    payload = sys.stdin.buffer.read(MAX_STDIN_BYTES + 1)
    if len(payload) > MAX_STDIN_BYTES:
        raise ValueError(f"pre-push input exceeds {MAX_STDIN_BYTES} bytes")
    return payload


def run_hook(command: list[str], payload: bytes, cwd: Path, environment=None) -> int:
    completed = subprocess.run(command, input=payload, check=False, cwd=cwd, env=environment)
    return completed.returncode


def head_revision(root: Path) -> str:
    result = subprocess.run(
        ["git", "rev-parse", "--verify", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
        cwd=root,
    )
    return result.stdout.strip()


def run_checker(root: Path, repository: str, argv: list[str], revision: str | None, payload: bytes) -> int:
    path = root / "scripts/release/check-push-dependency-evidence.py"
    spec = importlib.util.spec_from_file_location("katana_push_dependency_checker", path)
    if spec is None or spec.loader is None:
        raise ImportError(f"cannot load checker: {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    arguments = ["--repository", repository, "--root", str(root), *argv]
    result = module.main(arguments, verified_source=revision, payload=payload)
    return int(result or 0)


def checked_owner(root: Path, argv: list[str], payload: bytes, backup_path: Path):
    before = head_revision(root)
    if backup_path.exists() or backup_path.is_symlink():
        if backup_path.is_symlink() or not backup_path.is_file():
            raise ValueError(f"unsafe existing hook backup: {backup_path}")
        if not os.access(backup_path, os.X_OK):
            raise ValueError(f"existing hook backup is not executable: {backup_path}")
    else:
        raise ValueError(f"required existing hook backup is missing: {backup_path}")

    with VerificationSession(root, before) as session:
        status = run_hook([str(backup_path), *argv], payload, root, session.environment())
        if status != 0:
            return status, None
        after = head_revision(root)
        if before != after:
            raise ValueError(f"existing pre-push hook changed HEAD: {before} -> {after}")
        return 0, session.verified_source()


def dispatch(repository: str, argv: list[str], backup: Path | None = None) -> int:
    if os.environ.get("LEFTHOOK") == "0":
        raise ValueError("disabled existing owner cannot provide fresh verification")
    root = active_root()
    payload = read_bounded_stdin()
    status, revision = checked_owner(root, argv, payload, backup or hooks_dir() / BACKUP_NAME)
    if status != 0:
        return status
    return run_checker(root, repository, argv, revision, payload)


def parse_args(arguments: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", required=True, help="OWNER/NAME")
    parser.add_argument("--existing-hook", type=Path)
    parser.add_argument("hook_args", nargs=argparse.REMAINDER)
    options = parser.parse_args(arguments)
    if options.hook_args[:1] == ["--"]:
        options.hook_args = options.hook_args[1:]
    return options


def main(arguments: list[str] | None = None) -> int:
    options = parse_args(arguments)
    try:
        return dispatch(options.repository, options.hook_args, options.existing_hook)
    except (OSError, subprocess.SubprocessError, ValueError) as error:
        print(f"pre-push dispatcher failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
