"""Shared worktree runtime and common-hook scope checks."""

import os
from pathlib import Path
import subprocess


def clean_environment() -> dict[str, str]:
    return {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}


def git_path(*arguments: str) -> Path:
    result = subprocess.run(["git", *arguments], check=True, capture_output=True,
                            text=True, env=clean_environment())
    path = Path(result.stdout.strip())
    return (Path.cwd() / path).resolve() if not path.is_absolute() else path.resolve()


def validate_checkout_runtimes() -> None:
    source = Path(__file__).resolve().parent
    expected = {path.name: path.read_bytes() for path in source.glob("*.py") if not path.name.startswith("test")}
    result = subprocess.run(["git", "worktree", "list", "--porcelain", "-z"],
                            check=True, capture_output=True, text=True, env=clean_environment())
    roots = [Path(item.removeprefix("worktree ")) for item in result.stdout.split("\0") if item.startswith("worktree ")]
    if not roots:
        raise ValueError("no registered worktree runtime was found")
    for root in roots:
        for name, contents in expected.items():
            candidate = root / "scripts/release" / name
            if candidate.is_symlink() or not candidate.is_file() or candidate.read_bytes() != contents:
                raise ValueError(f"shared hook runtime is incompatible in worktree: {root} ({name})")


def validate_regular(path: Path, label: str) -> None:
    if path.is_symlink() or (path.exists() and not path.is_file()):
        raise ValueError(f"unsafe {label}: {path}")


def validate_hooks_scope(hooks: Path, common: Path) -> None:
    expected = common.resolve() / "hooks"
    if expected.is_symlink() or hooks.is_symlink() or hooks.resolve() != expected:
        raise ValueError(f"hooks path is outside Git common hooks: {hooks}")
