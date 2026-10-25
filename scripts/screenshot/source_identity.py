#!/usr/bin/env python3
"""Fail-closed Git source identity guard for canonical screenshot capture."""

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path


SCHEMA_VERSION = 1


def local_git_environment() -> dict[str, str]:
    # hook環境の別repo指定より、引数で指定したrepositoryを優先する。
    return {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}


def git(repo: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", "-C", str(repo), *args],
        check=True,
        capture_output=True,
        text=True,
        env=local_git_environment(),
    )
    return result.stdout


def output_root_allowed(repo: Path, identity: Path) -> None:
    root = identity.parent.resolve()
    try:
        relative = root.relative_to(repo)
    except ValueError:
        return
    if not relative.parts or relative.parts[0] not in {"tmp", "target"}:
        raise ValueError("repository output must be under tmp/ or target/")
    subprocess.run(
        ["git", "-C", str(repo), "check-ignore", "--quiet", "--", str(root)],
        check=True,
        capture_output=True,
        text=True,
        env=local_git_environment(),
    )


def clean_status(repo: Path) -> list[str]:
    status = git(repo, "status", "--porcelain=v1", "--untracked-files=normal")
    entries = status.splitlines()
    if entries:
        raise ValueError("Git status is not clean before or after capture")
    return entries


def begin(repo: Path, identity_path: Path) -> None:
    repo = repo.resolve(strict=True)
    identity = identity_path.resolve()
    if identity.exists():
        raise ValueError("source identity must be a new file")
    output_root_allowed(repo, identity)
    head = git(repo, "rev-parse", "HEAD").strip()
    clean_status(repo)
    identity.write_text(
        json.dumps({"schema_version": SCHEMA_VERSION, "head": head}, indent=2) + "\n",
        encoding="utf-8",
    )


def load_identity(path: Path) -> dict[str, object]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict) or set(value) != {"schema_version", "head"}:
        raise ValueError("invalid source identity schema")
    if type(value["schema_version"]) is not int or value["schema_version"] != SCHEMA_VERSION:
        raise ValueError("invalid source identity schema version")
    head = value["head"]
    if not isinstance(head, str) or len(head) not in {40, 64} or not all(
        character in "0123456789abcdef" for character in head
    ):
        raise ValueError("source identity HEAD must be a full lowercase Git object ID")
    return value


def verify(repo: Path, identity_path: Path) -> None:
    repo = repo.resolve(strict=True)
    identity = load_identity(identity_path.resolve(strict=True))
    head = git(repo, "rev-parse", "HEAD").strip()
    clean_status(repo)
    if head != identity["head"]:
        raise ValueError("Git HEAD changed during capture")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("begin", "verify"))
    parser.add_argument("--repository", required=True, type=Path)
    parser.add_argument("--identity", required=True, type=Path)
    args = parser.parse_args()
    try:
        if args.action == "begin":
            begin(args.repository, args.identity)
        else:
            verify(args.repository, args.identity)
    except (OSError, ValueError, json.JSONDecodeError, subprocess.CalledProcessError) as error:
        print(f"ERROR: source identity {args.action} failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
