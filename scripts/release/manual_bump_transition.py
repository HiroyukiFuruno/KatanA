"""Validate an exact version-only Git transition for a manual release."""
from __future__ import annotations

import re
import argparse
import json
import os
import subprocess
import tomllib
from pathlib import Path, PurePosixPath

REPOSITORY = "HiroyukiFuruno/KatanA"
WORKFLOW = ".github/workflows/build-and-release.yml"
FILES = ("Cargo.toml", "Cargo.lock", "crates/katana-ui/Info.plist")
SCHEMA = "katana-manual-bump-transition-v1"
COMMIT = re.compile(r"[0-9a-f]{40}\Z")
VERSION = re.compile(r"v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\Z")


def git(root: Path, *args: str) -> bytes:
    return subprocess.run(["git", "-C", str(root), *args], check=True,
                          capture_output=True, timeout=60).stdout


def blob(root: Path, commit: str, path: str) -> str:
    entry = git(root, "ls-tree", commit, "--", path).decode().strip()
    if not entry.startswith("100644 blob "):
        raise ValueError("version files and manifests must be regular files")
    return git(root, "show", f"{commit}:{path}").decode("utf-8")


def workspace_names(root: Path, base: str, cargo: dict) -> set[str]:
    members = cargo["workspace"]["members"]
    if not isinstance(members, list) or not members:
        raise ValueError("workspace members missing")
    names = set()
    for member in members:
        path = PurePosixPath(member)
        if path.is_absolute() or ".." in path.parts or str(path) != member:
            raise ValueError("workspace member must be an exact repository path")
        data = tomllib.loads(blob(root, base, f"{member}/Cargo.toml"))
        name = data["package"]["name"]
        if not isinstance(name, str) or not name or name in names:
            raise ValueError("workspace package name missing or duplicated")
        names.add(name)
    return names


def expected_files(root: Path, base: str, version: str) -> dict[str, str]:
    before = {path: blob(root, base, path) for path in FILES}
    cargo = tomllib.loads(before["Cargo.toml"])
    previous = cargo["workspace"]["package"]["version"]
    target = version.removeprefix("v")
    old_match, new_match = VERSION.fullmatch("v" + previous), VERSION.fullmatch(version)
    if not old_match or not new_match or tuple(map(int, new_match.groups())) != (
        int(old_match[1]), int(old_match[2]), int(old_match[3]) + 1
    ):
        raise ValueError("manual version bump must be exactly the next patch")
    section = re.compile(r'(?ms)(^\[workspace\.package\]\s*\n(?:(?!^\[).)*?^version\s*=\s*")' + re.escape(previous) + r'("[^\n]*$)')
    cargo_text, count = section.subn(lambda match: match[1] + target + match[2], before["Cargo.toml"])
    if count != 1:
        raise ValueError("workspace version declaration is not unique")
    lock = before["Cargo.lock"]
    for name in workspace_names(root, base, cargo):
        pattern = re.compile(r'(?m)(^name = "' + re.escape(name) + r'"\nversion = ")' + re.escape(previous) + r'("$)')
        lock, count = pattern.subn(lambda match: match[1] + target + match[2], lock)
        if count != 1:
            raise ValueError("workspace lock entry missing or duplicated")
    plist_pattern = re.compile(r'(<key>CFBundleShortVersionString</key>\s*<string>)v' + re.escape(previous) + r'(</string>)')
    plist, count = plist_pattern.subn(lambda match: match[1] + version + match[2], before[FILES[2]])
    if count != 1:
        raise ValueError("bundle version declaration is not unique")
    return {FILES[0]: cargo_text, FILES[1]: lock, FILES[2]: plist}


def verify_transition(root: Path, record: dict, trigger: str, source: str, version: str,
                      run_id: int, run_attempt: int) -> list[str]:
    expected = {"schema": SCHEMA, "repository": REPOSITORY, "workflow": WORKFLOW,
                "run_id": run_id, "run_attempt": run_attempt, "event": "workflow_dispatch",
                "branch": "master", "trigger": trigger, "source": source, "version": version}
    if (not isinstance(record, dict) or record != expected or type(run_id) is not int or run_id <= 0
            or type(run_attempt) is not int or run_attempt <= 0 or not COMMIT.fullmatch(trigger)
            or not COMMIT.fullmatch(source) or not VERSION.fullmatch(version) or trigger == source):
        raise ValueError("manual transition metadata does not match its actual run")
    expected_blobs = expected_files(root, trigger, version)
    commits = git(root, "rev-list", "--reverse", "--ancestry-path", f"{trigger}..{source}").decode().splitlines()
    if not 1 <= len(commits) <= len(FILES) or commits[-1] != source:
        raise ValueError("manual bump must contain one to three version commits")
    parent = trigger
    changed = set()
    for commit in commits:
        if git(root, "rev-list", "--parents", "-n", "1", commit).decode().strip() != f"{commit} {parent}":
            raise ValueError("manual bump commit chain must be linear and exact")
        paths = git(root, "diff-tree", "--no-commit-id", "--name-only", "-r", "--no-renames", "-z", parent, commit).decode().split("\0")
        paths = [path for path in paths if path]
        if not paths or any(path not in FILES or path in changed for path in paths):
            raise ValueError("manual bump changes an unexpected or duplicate file")
        for path in paths:
            if blob(root, commit, path) != expected_blobs[path]:
                raise ValueError("manual bump changes content beyond the expected version")
            changed.add(path)
        parent = commit
    if changed != set(FILES):
        raise ValueError("manual bump version files missing")
    return commits


def record_transition(root: Path, output: Path, version: str) -> bool:
    trigger = os.environ["GITHUB_SHA"]
    source = git(root, "rev-parse", "HEAD").decode().strip()
    if source == trigger:
        return False
    run_id = int(os.environ["GITHUB_RUN_ID"])
    attempt = int(os.environ["GITHUB_RUN_ATTEMPT"])
    record = {"schema": SCHEMA, "repository": os.environ["GITHUB_REPOSITORY"],
              "workflow": WORKFLOW, "run_id": run_id, "run_attempt": attempt,
              "event": os.environ["GITHUB_EVENT_NAME"],
              "branch": os.environ["GITHUB_REF"].removeprefix("refs/heads/"),
              "trigger": trigger, "source": source, "version": version}
    if os.environ["GITHUB_REF"] != "refs/heads/master":
        raise ValueError("manual bump recording requires master")
    verify_transition(root, record, trigger, source, version, run_id, attempt)
    with output.open("x", encoding="utf-8") as stream:
        json.dump(record, stream, sort_keys=True)
        stream.write("\n")
    return True


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--record", type=Path, required=True)
    parser.add_argument("--version", required=True)
    args = parser.parse_args()
    present = record_transition(Path.cwd(), args.record, args.version)
    with Path(os.environ["GITHUB_OUTPUT"]).open("a", encoding="utf-8") as stream:
        stream.write(f"present={'true' if present else 'false'}\n")


if __name__ == "__main__":
    main()
