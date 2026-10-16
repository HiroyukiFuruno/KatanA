"""Build a small real Cargo workspace for inventory comparison tests."""

from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[2]
RECORDER = ROOT / "scripts/release/record-test-invocation.py"
PHASES = {
    "workspace-lib-bin": ["--workspace", "--lib", "--bins"],
    "workspace-integration": ["--workspace", "--test", "*", "--exclude", "katana-ui"],
    "ui-parallel": ["-p", "katana-ui", "--test", "ui_integration_parallel"],
    "ui-serial": ["-p", "katana-ui", "--test", "ui_integration_serial"],
}


def _write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def _write_core(root: Path) -> None:
    _write(root / "crates/katana-core/Cargo.toml", """[package]
name = "katana-core"
version = "0.1.0"
edition = "2021"
""")
    _write(root / "crates/katana-core/src/lib.rs", "pub fn answer() -> u8 { 42 }\n")
    _write(root / "crates/katana-core/tests/core_integration.rs", """#[test]
fn fixture() { assert_eq!(katana_core::answer(), 42); }

#[test]
fn core_stable() { assert_eq!(katana_core::answer(), 42); }
""")


def _write_ui(root: Path) -> None:
    _write(root / "crates/katana-ui/Cargo.toml", """[package]
name = "katana-ui"
version = "0.1.0"
edition = "2021"

[dependencies]
katana-core = { path = "../katana-core" }
""")
    _write(root / "crates/katana-ui/src/lib.rs", "pub fn ui_answer() -> u8 { katana_core::answer() }\n")
    _write(root / "crates/katana-ui/tests/ui_integration_parallel.rs", """#[test]
fn parallel_case() { assert_eq!(katana_ui::ui_answer(), 42); }
""")
    _write(root / "crates/katana-ui/tests/ui_integration_serial.rs",
           "#[test]\nfn serial_case() { assert_eq!(katana_ui::ui_answer(), 42); }\n")


def _write_packages(root: Path) -> None:
    _write(root / "Cargo.toml", """[workspace]
members = ["crates/katana-core", "crates/katana-ui"]
resolver = "2"
""")
    _write(root / ".gitignore", "/target\n/inventory\n")
    _write_core(root)
    _write_ui(root)


def _commit_fixture(root: Path) -> None:
    env = _environment(root)
    subprocess.run(["cargo", "generate-lockfile", "--offline"], cwd=root, env=env,
                   check=True, capture_output=True, text=True)
    _write(root / "scripts/screenshot/Cargo.lock", (root / "Cargo.lock").read_text())
    subprocess.run(["git", "init", "-q"], cwd=root, env=env, check=True)
    subprocess.run(["git", "config", "user.name", "Inventory Fixture"], cwd=root, env=env,
                   check=True)
    subprocess.run(["git", "config", "user.email", "inventory@example.invalid"], cwd=root,
                   env=env, check=True)
    subprocess.run(["git", "add", "."], cwd=root, env=env, check=True)
    subprocess.run(["git", "-c", "commit.gpgsign=false", "commit", "-qm", "fixture"],
                   cwd=root, env=env, check=True)


def create_fixture(root: Path) -> Path:
    """Create a clean, committed miniature workspace and return its root."""
    root = root.resolve()
    _write_packages(root)
    _commit_fixture(root)
    return root


def _recording_commands() -> dict[str, tuple[str, list[str]]]:
    commands = {"normal": ("test", ["cargo", "test", "--workspace", "--offline"])}
    for phase, selection in PHASES.items():
        if phase == "workspace-integration":
            test_args = [*selection, "--offline", "--", "--skip", "fixture",
                         "--test-threads=1"]
        else:
            test_args = [*selection, "--offline", "--", "--test-threads=1"]
        commands[phase] = ("coverage", ["cargo", "llvm-cov", "--no-report",
                                         "--no-cfg-coverage", *test_args])
    return commands


def _record_one(root: Path, destination: Path, kind: str, command: list[str]) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    result = subprocess.run([sys.executable, "-B", str(RECORDER), "--output",
                             str(destination), "--kind", kind, "--", *command],
                            cwd=root, env=_environment(root), capture_output=True, text=True,
                            timeout=60)
    if not destination.is_file() or result.returncode:
        raise RuntimeError(f"recording failed for {destination}: {result.stderr}")


def record_fixture(root: Path, output: Path) -> dict[str, Path]:
    """Run the real normal command and all four real coverage selections."""
    paths = {}
    for name, (kind, command) in _recording_commands().items():
        destination = output / ("normal.json" if name == "normal" else f"coverage/{name}.json")
        _record_one(root, destination, kind, command)
        paths[name] = destination
    return paths


def _environment(root: Path) -> dict[str, str]:
    environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
    environment.update({"CARGO_TARGET_DIR": str(root / "target"), "CARGO_BUILD_JOBS": "1"})
    return environment
