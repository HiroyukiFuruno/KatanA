#!/usr/bin/env python3
"""Run cargo clean only while known Cargo profiles are exclusively locked."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys

from cargo_sweep_guard import (
    CleanupUnavailable,
    acquire_profile_locks,
    cargo_target_directory,
    profile_directories,
    release_profile_locks,
    require_local_lock_filesystem,
)

REFUSED = 75


def validate_target(target: Path, manifest: Path) -> Path:
    if target.is_symlink():
        raise CleanupUnavailable("Cargo target directory is not a safe real path")
    resolved = target.resolve()
    manifest_root = manifest.resolve().parent
    if not resolved.is_absolute() or resolved.is_symlink():
        raise CleanupUnavailable("Cargo target directory is not a safe real path")
    if resolved in {Path("/"), Path.home()} or resolved == manifest_root:
        raise CleanupUnavailable("Cargo target directory is a protected path")
    if resolved in manifest_root.parents:
        raise CleanupUnavailable("Cargo target directory is an ancestor of the manifest")
    return resolved


def run_clean(manifest: Path, target: Path, cargo_spec: str) -> int:
    cargo, environment = cargo_command(cargo_spec)
    command = [
        *cargo,
        "clean",
        "--manifest-path",
        str(manifest.resolve()),
        "--target-dir",
        str(target),
    ]
    try:
        result = subprocess.run(command, env=environment, text=True, capture_output=True)
    except OSError as error:
        print(f"cargo clean could not run: {error}", file=sys.stderr)
        return 1
    if result.stdout:
        print(result.stdout, end="")
    if result.stderr:
        print(result.stderr, end="", file=sys.stderr)
    return result.returncode


def cargo_command(spec: str) -> tuple[list[str], dict[str, str]]:
    try:
        tokens = shlex.split(spec)
    except ValueError as error:
        raise CleanupUnavailable(f"invalid CARGO command: {error}")
    environment = os.environ.copy()
    while tokens and "=" in tokens[0] and tokens[0].split("=", 1)[0].isidentifier():
        name, value = tokens.pop(0).split("=", 1)
        environment[name] = value
    if not tokens:
        raise CleanupUnavailable("CARGO command is empty")
    return tokens, environment


def target_directory(manifest: Path, spec: str) -> Path:
    if spec == "cargo":
        return cargo_target_directory(manifest)
    command, environment = cargo_command(spec)
    for name in ("CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR"):
        configured = environment.get(name)
        if configured and Path(configured).is_symlink():
            raise CleanupUnavailable(f"{name} points to a symlink; cleanup skipped")
    result = subprocess.run(
        [*command, "metadata", "--no-deps", "--format-version", "1", "--locked",
         "--manifest-path", str(manifest)],
        env=environment, capture_output=True, text=True,
    )
    if result.returncode:
        raise CleanupUnavailable(f"cargo metadata failed: {result.stderr.strip()}")
    try:
        directory = Path(json.loads(result.stdout)["target_directory"])
    except (KeyError, TypeError, json.JSONDecodeError) as error:
        raise CleanupUnavailable(f"cargo metadata has no usable target_directory: {error}")
    if not directory.is_absolute() or directory.is_symlink():
        raise CleanupUnavailable("Cargo target directory is a symlink; cleanup skipped")
    return directory


def run_guard(manifest: Path, cargo_spec: str) -> int:
    # WHY: sweepの安全なスキップをclean成功へ読み替えず、削除対象を保持する。
    try:
        target = validate_target(target_directory(manifest, cargo_spec), manifest)
        if not target.exists():
            print("Cargo target is absent; clean skipped")
            return 0
        profiles = profile_directories(target)
        require_local_lock_filesystem(target)
        descriptors = acquire_profile_locks(profiles)
    except (CleanupUnavailable, OSError) as error:
        print(f"Cargo clean refused safely: {error}", file=sys.stderr)
        return REFUSED
    try:
        return run_clean(manifest, target, cargo_spec)
    finally:
        release_profile_locks(descriptors)


def main() -> int:
    repository = Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest-path", type=Path, default=repository / "Cargo.toml")
    parser.add_argument("--cargo", default="cargo")
    arguments = parser.parse_args()
    return run_guard(arguments.manifest_path, arguments.cargo)


if __name__ == "__main__":
    raise SystemExit(main())
