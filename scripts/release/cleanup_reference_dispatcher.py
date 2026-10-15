"""Preserve the existing reference hook before checking a scoped cleanup intent."""

import argparse
import os
from pathlib import Path
import subprocess
import sys

from cleanup_existing_hook import DIGEST_ENV, INTENT_ENV, capture_owner_input, exit_like_owner
from cleanup_existing_hook import owner_command, owner_environment
from cleanup_intent import read_intent
from release_cleanup_audit import CleanupRefused, common_directory
from release_cleanup_transaction import MAX_TRANSACTION_BYTES, verify_prepared_deletion


def run_existing(path: Path | None, phase: str, payload: bytes) -> int:
    if path is None:
        return 0
    return subprocess.run(owner_command(path, phase), input=payload,
                          env=owner_environment(), check=False).returncode


def dispatch(intent: Path | None, digest: str | None, existing: Path | None, phase: str, payload: bytes) -> int:
    status = run_existing(existing, phase, payload)
    if status != 0:
        return status
    return verify_intent(intent, digest, existing is not None, phase, payload)


def verify_intent(intent: Path | None, digest: str | None, has_owner: bool, phase: str, payload: bytes) -> int:
    if intent is None and digest is None:
        return 0
    if intent is None or digest is None:
        raise CleanupRefused("cleanup intent requires both path and digest")
    if len(payload) > MAX_TRANSACTION_BYTES:
        raise CleanupRefused("reference transaction exceeds the input bound")
    if has_owner and os.environ.get("LEFTHOOK") == "0":
        raise CleanupRefused("disabled existing owner cannot admit cleanup")
    if phase != "prepared":
        return 0
    # 既存hookの成否を保った後で、この操作専用の入力と実repoを照合する。
    value = read_intent(intent, digest)
    if str(common_directory(Path.cwd())) != value["common_git_dir"]:
        raise CleanupRefused("cleanup intent belongs to another transaction repository")
    verify_prepared_deletion(Path(value["root"]), value["branch"], value["expected"], payload)
    return 0


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--intent", type=Path, default=os.environ.get(INTENT_ENV))
    parser.add_argument("--intent-sha256", default=os.environ.get(DIGEST_ENV))
    parser.add_argument("--existing-hook", type=Path)
    parser.add_argument("phase", choices=("preparing", "prepared", "committed", "aborted"))
    options = parser.parse_args(arguments)
    try:
        payload, status = capture_owner_input(sys.stdin.buffer, options.existing_hook, options.phase)
        if status != 0:
            return status
        return verify_intent(options.intent, options.intent_sha256,
                             options.existing_hook is not None, options.phase, payload)
    except (CleanupRefused, OSError, ValueError, subprocess.SubprocessError) as error:
        print(f"cleanup reference dispatcher refused: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    exit_like_owner(main())
