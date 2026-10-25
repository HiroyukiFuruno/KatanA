#!/usr/bin/env python3
"""Install Lefthook and the owner-preserving pre-push dispatcher transactionally."""

from __future__ import annotations

import argparse
import importlib.util
import os
from pathlib import Path
import stat
import subprocess
import tempfile

from hook_installation_scope import clean_environment, git_path, validate_hooks_scope, validate_regular


def installer_module():
    source = Path(__file__).with_name("install-pre-push-dispatcher.py")
    spec = importlib.util.spec_from_file_location("katana_pre_push_installer", source)
    if spec is None or spec.loader is None:
        raise RuntimeError("pre-push installer cannot be loaded")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def hook_paths(repository: str) -> tuple[Path, Path]:
    common = git_path("rev-parse", "--git-common-dir")
    hooks = git_path("rev-parse", "--path-format=absolute", "--git-path", "hooks")
    validate_hooks_scope(hooks, common)
    hook, backup = hooks / "pre-push", hooks / "pre-push.katana-existing"
    validate_regular(hook, "pre-push target")
    validate_regular(backup, "pre-push backup")
    return hook, backup


def snapshot_dispatcher(repository: str, hook: Path, backup: Path) -> tuple[bytes, int] | None:
    installer = installer_module()
    if not installer.REPOSITORY.fullmatch(repository):
        raise ValueError("repository must be OWNER/NAME")
    if installer.is_installed(hook, repository, backup):
        return hook.read_bytes(), stat.S_IMODE(hook.stat().st_mode)
    validate_regular(hook, "pre-push target")
    validate_regular(backup, "pre-push backup")
    if backup.exists() or backup.is_symlink():
        raise FileExistsError(f"refusing unknown backup collision: {backup}")
    if hook.exists() and b"KATANA_PRE_PUSH_DISPATCHER_V1" in hook.read_bytes():
        raise ValueError("refusing unknown or damaged dispatcher wrapper")
    return None


def restore_dispatcher(
    hook: Path, snapshot: tuple[bytes, int], generated: tuple[bytes, int]
) -> None:
    validate_regular(hook, "pre-push target")
    generated_contents, generated_mode = generated
    if (hook.read_bytes() != generated_contents
            or stat.S_IMODE(hook.stat().st_mode) != generated_mode):
        raise RuntimeError("pre-push target changed during setup rollback")
    contents, mode = snapshot
    descriptor, name = tempfile.mkstemp(prefix=".pre-push-restore-", dir=hook.parent)
    temporary = Path(name)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(contents)
        os.chmod(temporary, mode)
        os.replace(temporary, hook)
    finally:
        temporary.unlink(missing_ok=True)


def hook_state(hook: Path) -> tuple[bytes, int]:
    if hook.is_file() and not hook.is_symlink():
        return hook.read_bytes(), stat.S_IMODE(hook.stat().st_mode)
    return b"", -1


def rollback_dispatcher(
    hook: Path, snapshot: tuple[bytes, int] | None, generated: tuple[bytes, int], status: int
) -> int:
    if snapshot is None or status == 0:
        return status
    try:
        restore_dispatcher(hook, snapshot, generated)
    except (OSError, RuntimeError) as error:
        print(f"pre-push dispatcher rollback refused: {error}", file=os.sys.stderr)
        return 2
    print("pre-push dispatcher restored after setup failure", file=os.sys.stderr)
    return status


def run(repository: str) -> int:
    hook, backup = hook_paths(repository)
    snapshot = snapshot_dispatcher(repository, hook, backup)
    environment = clean_environment()
    lefthook = subprocess.run(["lefthook", "install"], env=environment, check=False)
    generated = hook_state(hook)
    if lefthook.returncode != 0:
        return rollback_dispatcher(hook, snapshot, generated, lefthook.returncode)
    installer = Path(__file__).with_name("install-pre-push-dispatcher.py")
    try:
        result = subprocess.run(["python3", "-B", str(installer), "--repository", repository],
                                env=environment, check=False)
    except (OSError, subprocess.SubprocessError) as error:
        print(f"pre-push dispatcher install failed: {error}", file=os.sys.stderr)
        return rollback_dispatcher(hook, snapshot, hook_state(hook), 2)
    return rollback_dispatcher(hook, snapshot, generated, result.returncode)


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", required=True)
    options = parser.parse_args(arguments)
    try:
        return run(options.repository)
    except (OSError, RuntimeError, subprocess.SubprocessError, ValueError) as error:
        print(f"hook setup failed: {error}", file=os.sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
