"""Install a reference dispatcher while preserving its existing owner."""

from contextlib import contextmanager
import os
from pathlib import Path
import tempfile

from hook_installation_scope import validate_regular

BACKUP_NAME = "reference-transaction.katana-existing"
MARKER = "KATANA_REFERENCE_DISPATCHER_V1"
MARKER_PREFIX = "KATANA_REFERENCE_DISPATCHER_"
LOCK_NAME = ".katana-reference-install.lock"


def wrapper(has_owner: bool) -> bytes:
    owner = ' --existing-hook "$hooks/' + BACKUP_NAME + '"' if has_owner else ""
    return ("#!/bin/sh\n# " + MARKER + "\nset -eu\n"
            'root=$(git rev-parse --show-toplevel) || exit 2\n'
            'hooks=$(git rev-parse --git-path hooks) || exit 2\n'
            'exec python3 "$root/scripts/release/cleanup_reference_dispatcher.py"'
            + owner + ' "$@"\n').encode()


@contextmanager
def installation_lock(hooks: Path):
    lock = hooks / LOCK_NAME
    lock.mkdir(mode=0o700)
    try:
        yield
    finally:
        lock.rmdir()


def installed(hook: Path, backup: Path) -> bool:
    if not hook.exists():
        return False
    payload = hook.read_bytes()
    for has_owner in (True, False):
        if payload != wrapper(has_owner):
            continue
        if has_owner != backup.is_file() or not os.access(hook, os.X_OK):
            raise ValueError("reference dispatcher owner state changed")
        if has_owner and not os.access(backup, os.X_OK):
            raise ValueError("retained reference owner is not executable")
        return True
    if MARKER_PREFIX.encode() in payload:
        raise ValueError("refusing unknown or damaged reference wrapper")
    return False


def temporary_wrapper(hooks: Path, has_owner: bool) -> Path:
    descriptor, name = tempfile.mkstemp(prefix=".reference-wrapper-", dir=hooks)
    temporary = Path(name)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(wrapper(has_owner))
        temporary.chmod(0o755)
    except BaseException:
        temporary.unlink(missing_ok=True)
        raise
    return temporary


def activate(hook: Path, backup: Path, temporary: Path, has_owner: bool) -> None:
    if not has_owner:
        os.link(temporary, hook, follow_symlinks=False)
        return
    # 既存backupを上書きせずowner inodeを保持してからwrapperを活性化する。
    os.link(hook, backup, follow_symlinks=False)
    try:
        validate_regular(backup, "retained reference owner")
        validate_regular(hook, "reference target")
        if not os.path.samefile(hook, backup):
            raise ValueError("reference owner changed during installation")
        os.replace(temporary, hook)
    except BaseException:
        if hook.exists() and not hook.is_symlink() and os.path.samefile(hook, backup):
            backup.unlink()
        raise


def install(hooks: Path) -> bool:
    hooks.mkdir(exist_ok=True)
    with installation_lock(hooks):
        hook, backup = hooks / "reference-transaction", hooks / BACKUP_NAME
        validate_regular(hook, "reference target")
        validate_regular(backup, "reference backup")
        if installed(hook, backup):
            return False
        if backup.exists():
            raise FileExistsError("refusing unknown reference backup collision")
        has_owner = hook.exists()
        if has_owner and not os.access(hook, os.X_OK):
            raise PermissionError("existing reference owner is not executable")
        temporary = temporary_wrapper(hooks, has_owner)
        try:
            activate(hook, backup, temporary, has_owner)
        finally:
            temporary.unlink(missing_ok=True)
    return True
