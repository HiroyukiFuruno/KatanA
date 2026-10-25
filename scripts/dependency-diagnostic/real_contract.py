#!/usr/bin/env python3
"""Run cargo-outdated against a read-only manifest overlay."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import tempfile
import tomllib


def toml_value(value: object) -> str:
    if isinstance(value, dict):
        items = ", ".join(f"{json.dumps(str(k))} = {toml_value(v)}" for k, v in value.items())
        return "{ " + items + " }"
    if isinstance(value, list):
        return "[" + ", ".join(toml_value(item) for item in value) + "]"
    if isinstance(value, (bool, int, float, str)):
        return json.dumps(value, allow_nan=False)
    raise TypeError(f"unsupported TOML value: {type(value).__name__}")


def serialize_manifest(data: dict) -> str:
    rendered = "\n".join(f"{json.dumps(str(k))} = {toml_value(v)}" for k, v in data.items()) + "\n"
    if tomllib.loads(rendered) != data:
        raise ValueError("temporary manifest TOML roundtrip changed dependency metadata")
    return rendered


def parse_manifest(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def _workspace_members(root: Path, manifest: dict) -> set[Path]:
    workspace = manifest.get("workspace", {})
    members = {member.resolve() for pattern in workspace.get("members", [])
               for member in root.glob(pattern)}
    excluded = {member.resolve() for pattern in workspace.get("exclude", [])
                for member in root.glob(pattern)}
    return members - excluded


def _rewrite_paths(value: object, source: Path, members: set[Path]) -> object:
    if isinstance(value, dict):
        rewritten = {key: _rewrite_paths(item, source, members) for key, item in value.items()}
        if isinstance(value.get("path"), str):
            target = (source.parent / value["path"]).resolve()
            if not any(target == member or member in target.parents for member in members):
                rewritten["path"] = str(target)
        return rewritten
    if isinstance(value, list):
        return [_rewrite_paths(item, source, members) for item in value]
    return value


def _copy_member(source: Path, overlay: Path, members: set[Path]) -> None:
    overlay.mkdir(parents=True, exist_ok=True)
    manifest = source / "Cargo.toml"
    data = _rewrite_paths(parse_manifest(manifest), manifest, members)
    (overlay / "Cargo.toml").write_text(serialize_manifest(data), encoding="utf-8")
    lock = source / "Cargo.lock"
    if lock.is_file():
        shutil.copy2(lock, overlay / "Cargo.lock")
    for child in source.iterdir():
        if child.name in {"Cargo.toml", "Cargo.lock", "target", ".git"}:
            continue
        (overlay / child.name).symlink_to(child, target_is_directory=child.is_dir())


def create_overlay(root: Path, temporary: Path) -> Path:
    root_manifest = root / "Cargo.toml"
    data = parse_manifest(root_manifest)
    members = _workspace_members(root, data)
    temporary.mkdir(parents=True, exist_ok=True)
    rewritten = _rewrite_paths(data, root_manifest, members)
    (temporary / "Cargo.toml").write_text(serialize_manifest(rewritten), encoding="utf-8")
    lock = root / "Cargo.lock"
    if lock.is_file():
        shutil.copy2(lock, temporary / "Cargo.lock")
    config = root / ".cargo"
    if config.is_dir():
        (temporary / ".cargo").symlink_to(config, target_is_directory=True)
    for member in members:
        _copy_member(member, temporary / member.relative_to(root.resolve()), members)
    workspace = data.get("workspace", {})
    for pattern in workspace.get("exclude", []):
        for excluded in root.glob(pattern):
            relative = excluded.resolve().relative_to(root.resolve())
            _copy_member(excluded, temporary / relative, members)
    return temporary / "Cargo.toml"


def cargo_argv(value: str) -> tuple[list[str], dict[str, str]]:
    parts = shlex.split(value)
    assignments: dict[str, str] = {}
    while parts and "=" in parts[0] and not parts[0].startswith("="):
        key, item = parts.pop(0).split("=", 1)
        assignments[key] = item
    if not parts:
        raise ValueError("cargo command is empty")
    return parts, assignments


def run(root: Path, cargo: str) -> int:
    command, assignments = cargo_argv(cargo)
    environment = dict(os.environ)
    environment.update(assignments)
    with tempfile.TemporaryDirectory(prefix="cargo-outdated-overlay-") as name:
        manifest = create_overlay(root.resolve(), Path(name))
        result = subprocess.run(command + ["outdated", "--workspace", "--manifest-path", str(manifest)],
                                cwd=root.resolve(), env=environment,
                                capture_output=True, text=True)
        print(result.stdout, end="")
        print(result.stderr, end="", file=sys.stderr)
        return result.returncode


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--cargo", required=True)
    args = parser.parse_args()
    try:
        return run(args.root, args.cargo)
    except (OSError, TypeError, ValueError, tomllib.TOMLDecodeError) as error:
        print(f"dependency diagnostic failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
