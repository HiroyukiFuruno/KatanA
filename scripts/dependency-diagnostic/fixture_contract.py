#!/usr/bin/env python3
"""Exercise the cargo-outdated overlay against real temporary fixtures."""

from __future__ import annotations

import argparse
import contextlib
import io
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

import real_contract


def _write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def _write_fixture_sources(root: Path) -> None:
    _write(root / "app/src/main.rs", "fn main() {}\n")
    _write(root / "helper/src/lib.rs", "pub fn helper() {}\n")
    _write(root / "vendor/fixture/src/lib.rs", "pub fn fixture() {}\n")
    _write(root / ".cargo/config.toml", "[net]\noffline = true\n")


def _make_fixture(root: Path) -> None:
    _write(root / "Cargo.toml", """[workspace]
members = [\"app\", \"helper\"]
exclude = [\"vendor/fixture\"]
resolver = \"2\"

[patch.crates-io]
fixture-dependency = { path = \"vendor/fixture\" }
""")
    _write(root / "app/Cargo.toml", """[package]
name = \"fixture-app\"
version = \"0.1.0\"
edition = \"2021\"

[dependencies]
helper = { path = \"../helper\" }
fixture-dependency = \"0.1\"
""")
    _write(root / "helper/Cargo.toml", """[package]
name = \"helper\"
version = \"0.1.0\"
edition = \"2021\"
""")
    _write(root / "helper/Cargo.lock", "# member lock sentinel\n")
    _write(root / "vendor/fixture/Cargo.toml", """[package]
name = \"fixture-dependency\"
version = \"0.1.0\"
edition = \"2021\"
""")
    _write_fixture_sources(root)


def _cargo_command(value: str) -> list[str]:
    return real_contract.cargo_argv(value)[0]


def _generate_lock(root: Path, cargo: str) -> None:
    result = subprocess.run(_cargo_command(cargo) + ["generate-lockfile", "--offline"],
                            cwd=root, capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(result.stderr or result.stdout)


def _snapshot(root: Path) -> dict[str, bytes]:
    paths = list(root.rglob("Cargo.toml")) + list(root.rglob("Cargo.lock"))
    paths += list(root.rglob(".cargo/config.toml"))
    return {str(path.relative_to(root)): path.read_bytes() for path in paths}


def _metadata(root: Path, cargo: str) -> dict:
    with tempfile.TemporaryDirectory(prefix="dependency-metadata-") as name:
        manifest = real_contract.create_overlay(root, Path(name))
        assert (manifest.parent / "helper/Cargo.lock").read_text() == "# member lock sentinel\n"
        result = subprocess.run(_cargo_command(cargo) + ["metadata", "--format-version", "1",
                                    "--locked", "--manifest-path", str(manifest)],
                                cwd=root.resolve(), capture_output=True, text=True, check=True)
        return json.loads(result.stdout)


def _tool_version(cargo: str) -> str:
    result = subprocess.run(_cargo_command(cargo) + ["outdated", "--version"], capture_output=True,
                            text=True, check=True)
    return result.stdout.strip()


def _assert_metadata(data: dict, root: Path) -> None:
    packages = {item["name"]: item for item in data["packages"]}
    overlay = Path(packages["fixture-app"]["manifest_path"]).parents[1]
    assert packages["helper"]["manifest_path"] == str(overlay / "helper/Cargo.toml")
    assert packages["fixture-dependency"]["manifest_path"] == str(root.resolve() / "vendor/fixture/Cargo.toml")
    assert not any(item.endswith("fixture-dependency@0.1.0")
                   for item in data["workspace_members"])
    dependency = next(item for item in packages["fixture-app"]["dependencies"]
                      if item["name"] == "helper")
    assert dependency["path"] == str(overlay / "helper")


def _run_capture(root: Path, cargo: str) -> tuple[int, str]:
    output = io.StringIO()
    with contextlib.redirect_stdout(output), contextlib.redirect_stderr(output):
        code = real_contract.run(root, cargo)
    return code, output.getvalue()


def _success_case(root: Path, cargo: str) -> None:
    _make_fixture(root)
    _generate_lock(root, cargo)
    _assert_metadata(_metadata(root, cargo), root)
    before = _snapshot(root)
    code, output = _run_capture(root, cargo)
    assert code == 0 and output
    assert _snapshot(root) == before
    assert not list(root.rglob("*.bak"))


def _failure_case(root: Path, cargo: str) -> tuple[int, str]:
    _make_fixture(root)
    _generate_lock(root, cargo)
    _write(root / "app/Cargo.toml", (root / "app/Cargo.toml").read_text(encoding="utf-8")
           + 'missing-dependency = "0.1"\n')
    before = _snapshot(root)
    code, output = _run_capture(root, cargo)
    assert code != 0 and output
    assert _snapshot(root) == before
    assert not list(root.rglob("*.bak"))
    return code, output


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--cargo", required=True)
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="dependency-diagnostic-fixture-") as name:
        success_root = Path(name) / "success"
        failure_root = Path(name) / "failure"
        _success_case(success_root, args.cargo)
        code, output = _failure_case(failure_root, args.cargo)
    print(f"tool={_tool_version(args.cargo)}; metadata=verified; success_exit=0; "
          f"failure_exit={code}; output=preserved; backups=0")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (AssertionError, OSError, subprocess.SubprocessError) as error:
        print(f"dependency diagnostic fixture failed: {error!r}", file=sys.stderr)
        raise SystemExit(2)
