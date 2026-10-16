#!/usr/bin/env python3
"""Run cargo-sweep only while Cargo's profile locks are exclusively held."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
from cargo_sweep_target import (
    CleanupUnavailable,
    _validate_profile,
    profile_directories,
    require_local_lock_filesystem,
)

try:
    import fcntl
except ImportError:  # Windows: do not clean without a proven compatible lock.
    fcntl = None


def cargo_target_directory(manifest: Path) -> Path:
    command = ["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"]
    try:
        result = subprocess.run(
            [*command, "--manifest-path", str(manifest)], capture_output=True, text=True
        )
    except OSError as error:
        raise CleanupUnavailable(f"could not run cargo metadata: {error}")
    if result.returncode:
        raise CleanupUnavailable(f"cargo metadata failed: {result.stderr.strip()}")
    for name in ("CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR"):
        configured = os.environ.get(name)
        if configured and Path(configured).is_symlink():
            raise CleanupUnavailable(f"{name} points to a symlink; cleanup skipped")
    try:
        directory = Path(json.loads(result.stdout)["target_directory"])
    except (KeyError, TypeError, json.JSONDecodeError) as error:
        raise CleanupUnavailable(f"cargo metadata has no usable target_directory: {error}")
    if not directory.is_absolute() or directory.is_symlink():
        raise CleanupUnavailable("Cargo target directory is a symlink; cleanup skipped")
    return directory


def acquire_profile_locks(profiles: list[Path]) -> list[int]:
    if fcntl is None or os.name != "posix":
        raise CleanupUnavailable("compatible flock is unavailable; cleanup skipped")
    descriptors: list[int] = []
    try:
        for profile in profiles:
            descriptors.append(_open_and_lock(profile))
    except Exception:
        release_profile_locks(descriptors)
        raise
    return descriptors


def _open_and_lock(profile: Path) -> int:
    try:
        _validate_profile(profile)
        flags = os.O_CREAT | os.O_RDWR | getattr(os, "O_NOFOLLOW", 0)
        descriptor = os.open(profile / ".cargo-lock", flags, 0o666)
    except OSError as error:
        raise CleanupUnavailable(f"could not open profile lock ({profile}): {error}")
    try:
        fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except OSError as error:
        os.close(descriptor)
        raise CleanupUnavailable(f"profile lock unavailable ({profile}): {error}")
    return descriptor


def release_profile_locks(descriptors: list[int]) -> None:
    for descriptor in reversed(descriptors):
        if fcntl is not None:
            fcntl.flock(descriptor, fcntl.LOCK_UN)
        os.close(descriptor)


def run_guard(manifest: Path, dry_run: bool) -> int:
    try:
        target = cargo_target_directory(manifest)
        profiles = profile_directories(target)
        if not profiles:
            print("Cargo target is absent; cleanup skipped")
            return 0
        require_local_lock_filesystem(target)
        descriptors = acquire_profile_locks(profiles)
    except (CleanupUnavailable, OSError) as error:
        print(f"Cargo sweep skipped safely: {error}")
        return 0
    try:
        return run_sweep(manifest, dry_run)
    finally:
        release_profile_locks(descriptors)


def run_sweep(manifest: Path, dry_run: bool) -> int:
    command = ["cargo", "sweep", "--time", "7"]
    if dry_run:
        command.append("--dry-run")
    command.append(str(manifest.parent))
    print("Running cargo-sweep with all discovered profile locks held")
    try:
        result = subprocess.run(command, text=True, capture_output=True)
    except OSError as error:
        print(f"cargo-sweep could not run: {error}", file=sys.stderr)
        return 1
    if result.stdout:
        print(result.stdout, end="")
    if result.stderr:
        print(result.stderr, end="", file=sys.stderr)
    if result.returncode:
        print(f"cargo-sweep failed with exit code {result.returncode}", file=sys.stderr)
    return result.returncode


def main() -> int:
    repository = Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest-path", type=Path, default=repository / "Cargo.toml")
    parser.add_argument("--dry-run", action="store_true")
    arguments = parser.parse_args()
    return run_guard(arguments.manifest_path, arguments.dry_run)


if __name__ == "__main__":
    raise SystemExit(main())
