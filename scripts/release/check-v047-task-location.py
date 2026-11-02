"""Require the v47 task file before evaluating its phase gate."""

from datetime import date
from pathlib import Path
import re
import subprocess
import sys

CHANGE = "v0-22-47-katana-contained-fixes"


def main(mode: str) -> None:
    root = Path.cwd()
    if (root / "openspec").is_symlink() or (root / "openspec/changes").is_symlink():
        raise ValueError("v47 change roots must not be symlinks")
    active = root / "openspec/changes" / CHANGE
    archive = root / "openspec/changes/archive"
    if archive.is_symlink():
        raise ValueError("v47 archive must be a regular directory")
    candidates = list(archive.glob(f"????-??-??-{CHANGE}")) if archive.is_dir() else []
    if mode == "post-release-evidence" and not active.exists():
        if len(candidates) != 1 or candidates[0].is_symlink():
            raise ValueError("exactly one v47 archived change is required")
        date.fromisoformat(candidates[0].name[:10])
        tasks = candidates[0] / "tasks.md"
        if tasks.is_symlink() or not tasks.is_file():
            raise ValueError("v47 archived tasks are missing")
        if any(re.match(r"^\s*-\s*\[([^xX/])\]\s+", line)
               and not re.match(r"^\s*-\s*\[([^xX/])\]\s+\d+\.\d+\b", line)
               for line in tasks.read_text(encoding="utf-8").splitlines()):
            raise ValueError("v47 archived user feedback remains incomplete")
        subprocess.run([sys.executable, "-B", "scripts/release/check-openspec-task-completion.py", str(tasks)],
                       check=True, timeout=60)
        return
    if candidates or active.is_symlink() or not active.is_dir():
        raise ValueError("v47 canonical active change is required")
    tasks = active / "tasks.md"
    if tasks.is_symlink() or not tasks.is_file():
        raise ValueError("v47 canonical tasks are missing")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("task gate mode is required")
    try:
        main(sys.argv[1])
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        raise SystemExit(str(error))
