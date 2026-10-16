#!/usr/bin/env python3
"""Validate the published renderer dependency graph used by release builds."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Any


RENDER_PACKAGES = {"katana-document-viewer", "katana-render-runtime"}
CRATES_IO_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"


def load_metadata(path: Path | None) -> dict[str, Any]:
    if path is not None:
        return json.loads(path.read_text(encoding="utf-8"))

    result = subprocess.run(
        ["cargo", "metadata", "--locked", "--format-version=1"],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip()
        raise RuntimeError(f"cargo metadata failed: {detail}")
    return json.loads(result.stdout)


def validate(metadata: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    packages = metadata.get("packages", [])

    for package_name in sorted(RENDER_PACKAGES):
        resolved = [
            package for package in packages if package.get("name") == package_name
        ]
        if not resolved:
            errors.append(f"required package is not resolved: {package_name}")
            continue
        for package in resolved:
            source = package.get("source")
            if source != CRATES_IO_SOURCE:
                errors.append(
                    f"{package_name} must resolve from crates.io, "
                    f"got source={source!r} for {package.get('id')!r}"
                )

    workspace_members = set(metadata.get("workspace_members", []))
    dependency_edges: dict[str, list[dict[str, Any]]] = {
        package_name: [] for package_name in RENDER_PACKAGES
    }
    for package in packages:
        if package.get("id") not in workspace_members:
            continue
        for dependency in package.get("dependencies", []):
            name = dependency.get("name")
            if name in dependency_edges:
                dependency_edges[name].append(dependency)

    for package_name, dependencies in sorted(dependency_edges.items()):
        if not dependencies:
            errors.append(f"workspace has no dependency edge for {package_name}")
            continue
        for dependency in dependencies:
            source = dependency.get("source")
            path = dependency.get("path")
            requirement = dependency.get("req")
            if path is not None:
                errors.append(f"{package_name} dependency contains path override: {path}")
            if source != CRATES_IO_SOURCE:
                errors.append(
                    f"{package_name} dependency must use the crates.io registry, "
                    f"got {source!r}"
                )
            if not isinstance(requirement, str) or not requirement.startswith("="):
                errors.append(
                    f"{package_name} dependency must be exact, got req={requirement!r}"
                )

    v8_packages = [package for package in packages if package.get("name") == "v8"]
    if len(v8_packages) != 1:
        versions = sorted(
            str(package.get("version", "<unknown>")) for package in v8_packages
        )
        errors.append(
            "release graph must resolve exactly one v8 package; "
            f"found {len(v8_packages)} ({', '.join(versions) or 'none'})"
        )

    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--metadata",
        type=Path,
        help="read cargo metadata JSON from a fixture instead of invoking Cargo",
    )
    args = parser.parse_args()

    try:
        metadata = load_metadata(args.metadata)
    except (OSError, ValueError, RuntimeError) as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 2

    errors = validate(metadata)
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1

    versions = {
        package["name"]: package["version"]
        for package in metadata["packages"]
        if package.get("name") in RENDER_PACKAGES | {"v8"}
    }
    print(
        "OK: published renderer dependency contract satisfied "
        + ", ".join(f"{name}={version}" for name, version in sorted(versions.items()))
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
