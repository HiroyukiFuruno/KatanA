"""Discover Cargo profiles and verify local lock semantics."""

from __future__ import annotations

import os
from pathlib import Path
import re
import subprocess
import sys


class CleanupUnavailable(Exception):
    pass


TRIPLE = re.compile(r"[A-Za-z0-9_]+(?:-[A-Za-z0-9_]+)+\Z")
ROOT_ARTIFACTS = {"doc", "examples", "package", "tmp", "incremental"}
LOCAL_FILESYSTEMS = {"apfs", "hfs", "ext2", "ext3", "ext4", "xfs", "btrfs", "overlay", "tmpfs"}


def profile_directories(target: Path) -> list[Path]:
    if target.is_symlink():
        raise CleanupUnavailable("Cargo target directory is a symlink")
    if not target.exists():
        return []
    if not target.is_dir():
        raise CleanupUnavailable("Cargo target directory is not a real directory")
    profiles = {target / name for name in ("debug", "release") if (target / name).exists()}
    try:
        with os.scandir(target) as entries:
            for entry in entries:
                profiles.update(_profiles_below_target_entry(entry))
    except OSError as error:
        raise CleanupUnavailable(f"could not inspect Cargo target profiles: {error}")
    for profile in profiles:
        _validate_profile(profile)
    return sorted(profiles)


def _profiles_below_target_entry(entry: os.DirEntry) -> set[Path]:
    path = Path(entry.path)
    if entry.is_symlink():
        raise CleanupUnavailable(f"symlink in Cargo target directory: {path}")
    if not entry.is_dir(follow_symlinks=False) or entry.name in {"debug", "release"}:
        return set()
    if entry.name in ROOT_ARTIFACTS:
        return set()
    if not TRIPLE.fullmatch(entry.name):
        raise CleanupUnavailable(f"unknown directory in Cargo target: {path}")
    with os.scandir(path) as entries:
        nested = [(item.name, item.path, item.is_symlink(), item.is_dir(follow_symlinks=False))
                  for item in entries]
    if any(link for _, _, link, _ in nested):
        raise CleanupUnavailable(f"symlink in target triple directory: {path}")
    if {name for name, _, _, is_dir in nested if is_dir} - {"debug", "release"}:
        raise CleanupUnavailable(f"ambiguous target profile directory: {path}")
    return {path / name for name in ("debug", "release") if (path / name).exists()}


def _validate_profile(profile: Path) -> None:
    if profile.is_symlink() or not profile.is_dir():
        raise CleanupUnavailable(f"ambiguous Cargo profile path: {profile}")
    fingerprint = profile / ".fingerprint"
    if fingerprint.is_symlink() or (fingerprint.exists() and not fingerprint.is_dir()):
        raise CleanupUnavailable(f"ambiguous fingerprint path: {fingerprint}")
    if (profile / ".cargo-lock").is_symlink():
        raise CleanupUnavailable(f"symlink Cargo lock: {profile / '.cargo-lock'}")


def require_local_lock_filesystem(target: Path) -> None:
    mounts = _local_mounts(str(target.resolve()))
    if not mounts:
        raise CleanupUnavailable("Cargo target mount type is unknown; cleanup skipped")
    _, filesystem, options = max(mounts)
    local = filesystem in LOCAL_FILESYSTEMS
    if sys.platform == "darwin":
        local = local and "local" in {option.strip() for option in options.split(",")}
    if not local:
        raise CleanupUnavailable(f"flock semantics are not verified for {filesystem}")


def _local_mounts(path: str) -> list[tuple[int, str, str]]:
    if sys.platform.startswith("linux"):
        try:
            lines = Path("/proc/self/mountinfo").read_text().splitlines()
        except OSError as error:
            raise CleanupUnavailable(f"could not read mount table: {error}")
        mounts = []
        for line in lines:
            before, separator, after = line.partition(" - ")
            left, right = before.split(), after.split()
            if separator and len(left) > 4 and right:
                mount = _unescape_mount(left[4])
                if _contains_path(path, mount):
                    mounts.append((len(mount), right[0], ",".join(left[5:])))
        return mounts
    if sys.platform != "darwin":
        raise CleanupUnavailable(f"unsupported lock filesystem platform: {sys.platform}")
    try:
        output = subprocess.run(["mount"], capture_output=True, text=True, check=True).stdout
    except (OSError, subprocess.CalledProcessError) as error:
        raise CleanupUnavailable(f"could not establish local lock filesystem: {error}")
    matches = (re.search(r" on (.+?) \(([^,]+),([^)]*)\)$", line) for line in output.splitlines())
    return [(len(match.group(1)), match.group(2), match.group(3)) for match in matches
            if match and _contains_path(path, match.group(1))]


def _contains_path(path: str, mount: str) -> bool:
    return path == mount or (mount == "/" and path.startswith("/")) or path.startswith(mount.rstrip("/") + "/")


def _unescape_mount(path: str) -> str:
    for escaped, character in (("\\040", " "), ("\\011", "\t"), ("\\012", "\n"), ("\\134", "\\")):
        path = path.replace(escaped, character)
    return path
