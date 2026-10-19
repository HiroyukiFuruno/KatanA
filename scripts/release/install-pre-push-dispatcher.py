#!/usr/bin/env python3
"""Install the existing-hook-first pre-push dispatcher safely."""

from __future__ import annotations

import argparse
import os
import re
import shlex
import stat
import subprocess
import sys
import tempfile
from pathlib import Path

from hook_installation_scope import clean_environment, git_path, validate_checkout_runtimes
from hook_installation_scope import validate_hooks_scope, validate_regular

BACKUP_NAME = "pre-push.katana-existing"
MARKER = "KATANA_PRE_PUSH_DISPATCHER_V1"
REPOSITORY = re.compile(r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$")


def wrapper(repository: str) -> str:
    # WHY: 共通hooksはlinked worktree間で共有されるため、実行時のcheckoutを解決する。
    return """#!/bin/sh
# %s
set -eu
root=$(git rev-parse --show-toplevel) || exit 2
hooks=$(git rev-parse --git-path hooks) || exit 2
exec python3 "$root/scripts/release/pre_push_dispatcher.py" --repository %s --existing-hook "$hooks/%s" "$@"
""" % (MARKER, shlex.quote(repository), BACKUP_NAME)


def is_installed(path: Path, repository: str, backup: Path) -> bool:
    if path.is_symlink() or not path.is_file():
        return False
    try:
        if path.read_bytes() != wrapper(repository).encode("utf-8"):
            return False
        if not backup.is_file() or backup.is_symlink() or not os.access(backup, os.X_OK):
            return False
        return True
    except (OSError, UnicodeError):
        return False


def validate_install_target(repository: str, hook: Path, backup: Path) -> None:
    if not REPOSITORY.fullmatch(repository):
        raise ValueError("repository must be OWNER/NAME")
    validate_regular(hook, "pre-push target")
    validate_regular(backup, "pre-push backup")
    if hook.exists() and MARKER in hook.read_text(encoding="utf-8"):
        raise ValueError("refusing unknown or damaged dispatcher wrapper")
    if not hook.exists():
        raise FileNotFoundError("existing pre-push hook is required")
    if backup.exists() or backup.is_symlink():
        if (backup.is_symlink() or not backup.is_file()
                or backup.read_bytes() != hook.read_bytes()
                or stat.S_IMODE(backup.stat().st_mode) != stat.S_IMODE(hook.stat().st_mode)):
            raise FileExistsError(f"refusing unknown backup collision: {backup}")
    if not hook.stat().st_mode & stat.S_IXUSR:
        raise PermissionError(f"existing pre-push hook is not executable: {hook}")


def write_temporary_wrapper(repository: str, hook: Path) -> Path:
    descriptor, filename = tempfile.mkstemp(prefix=".pre-push-wrapper-", dir=hook.parent)
    temporary = Path(filename)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as stream:
            stream.write(wrapper(repository))
        os.chmod(temporary, 0o755)
    except BaseException:
        temporary.unlink(missing_ok=True)
        raise
    return temporary


def activate_wrapper(hook: Path, backup: Path, temporary: Path, original_mode: int) -> None:
    # 置換前にownerを移動すると、割込後の通常pushが既存検査を迂回するためlinkで保持する。
    created_backup = False
    if not backup.exists():
        os.link(hook, backup, follow_symlinks=False)
        created_backup = True
    try:
        validate_regular(backup, "retained pre-push owner")
        validate_regular(hook, "pre-push target")
        if (stat.S_IMODE(hook.stat().st_mode) != original_mode
                or stat.S_IMODE(backup.stat().st_mode) != original_mode
                or backup.read_bytes() != hook.read_bytes()):
            raise ValueError("pre-push owner changed during installation")
        os.replace(temporary, hook)
    except BaseException:
        if (created_backup and hook.exists() and not hook.is_symlink()
                and os.path.samefile(hook, backup)):
            backup.unlink()
        raise


def install(repository: str, hook: Path, backup: Path) -> bool:
    if not REPOSITORY.fullmatch(repository):
        raise ValueError("repository must be OWNER/NAME")
    validate_regular(hook, "pre-push target")
    validate_regular(backup, "pre-push backup")
    if is_installed(hook, repository, backup):
        return False
    validate_install_target(repository, hook, backup)
    original_mode = stat.S_IMODE(hook.stat().st_mode)
    temporary = write_temporary_wrapper(repository, hook)
    try:
        activate_wrapper(hook, backup, temporary, original_mode)
    finally:
        if temporary.exists():
            temporary.unlink()
    return True


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", required=True, help="OWNER/NAME")
    options = parser.parse_args(arguments)
    try:
        common = git_path("rev-parse", "--git-common-dir")
        hooks = git_path("rev-parse", "--git-path", "hooks")
        validate_hooks_scope(hooks, common)
        validate_checkout_runtimes()
        hook = hooks / "pre-push"
        changed = install(options.repository, hook, hook.with_name(BACKUP_NAME))
    except (OSError, subprocess.SubprocessError, ValueError) as error:
        print(f"pre-push dispatcher install failed: {error}", file=sys.stderr)
        return 2
    print("pre-push dispatcher already installed" if not changed else f"installed pre-push dispatcher at {hook}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
