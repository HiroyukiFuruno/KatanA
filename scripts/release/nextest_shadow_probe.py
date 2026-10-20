"""Create a committed, real Cargo workspace for the nextest shadow runner."""

import argparse
import os
from pathlib import Path
import shutil
import subprocess
import uuid


def _write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def _workspace(root: Path) -> None:
    _write(root / "Cargo.toml", """[workspace]
members = ["crates/katana-core", "crates/katana-ui"]
resolver = "2"
""")
    _write(root / "crates/katana-core/Cargo.toml", """[package]
name = "katana-core"
version = "0.1.0"
edition = "2021"
""")
    _write(root / "crates/katana-core/src/lib.rs", """/// ```
/// assert_eq!(katana_core::double(2), 4);
/// ```
pub fn double(value: u32) -> u32 { value * 2 }

#[cfg(test)]
mod tests {
    #[test]
    fn core_calculation_is_nonempty() { assert_eq!(super::double(3), 6); }
}
""")
    _write(root / "crates/katana-core/tests/core_integration.rs", """#[test]
fn normal_case() { assert_eq!(katana_core::double(3), 6); }

#[test]
fn fixture_case() { assert_eq!(katana_core::double(4), 8); }
""")


def _ui(root: Path) -> None:
    _write(root / "crates/katana-ui/Cargo.toml", """[package]
name = "katana-ui"
version = "0.1.0"
edition = "2021"

[dependencies]
katana-core = { path = "../katana-core" }
""")
    _write(root / "crates/katana-ui/src/lib.rs", """pub fn ui_value(value: u32) -> u32 { katana_core::double(value) + 1 }

#[cfg(test)]
mod tests {
    #[test]
    fn ui_calculation_is_nonempty() { assert_eq!(super::ui_value(2), 5); }
}
""")
    for name, function in (("ui_integration_parallel", "parallel_case"),
                           ("ui_integration_serial", "serial_case")):
        _write(root / f"crates/katana-ui/tests/{name}.rs",
               f"#[test]\nfn {function}() {{ assert_eq!(katana_ui::ui_value(2), 5); }}\n")


def _worker(root: Path) -> None:
    _write(root / "crates/katana-ui/src/bin/kdv-office-worker.rs",
           "fn main() { assert_eq!(katana_ui::ui_value(2), 5); }\n")


def _configs(repo: Path, root: Path) -> None:
    destination = root / "scripts/release"
    destination.mkdir(parents=True, exist_ok=True)
    (root / "scripts/screenshot").mkdir(parents=True, exist_ok=True)
    for name in ("nextest-shadow.toml", "nextest-shadow-user.toml"):
        shutil.copy2(repo / "scripts/release" / name, destination / name)


def _git_environment() -> dict[str, str]:
    environment = {key: value for key, value in os.environ.items()
                   if not key.startswith("GIT_")}
    environment.update({"GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull,
                        "GIT_AUTHOR_NAME": "Nextest Shadow Fixture",
                        "GIT_AUTHOR_EMAIL": "fixture@example.invalid",
                        "GIT_COMMITTER_NAME": "Nextest Shadow Fixture",
                        "GIT_COMMITTER_EMAIL": "fixture@example.invalid"})
    return environment


def _commit(root: Path) -> None:
    environment = _git_environment()
    for command in (("git", "-c", "init.defaultBranch=fixture", "init", "-q"),
                    ("git", "config", "core.hooksPath", ".empty-hooks"),
                    ("git", "add", "."),
                    ("git", "-c", "commit.gpgsign=false", "commit", "-qm", "fixture")):
        subprocess.run(command, cwd=root, env=environment, check=True)


def _validate_output(repo: Path, output: Path) -> None:
    if (not output.is_absolute() or output != output.resolve()
            or repo / "tmp" not in output.parents or output.exists()):
        raise ValueError("probe output must be a new physical directory under repo tmp")
    for parent in (output, *output.parents):
        if parent.is_symlink() or (parent.exists() and not parent.is_dir()):
            raise ValueError("probe output has a symlink or non-directory ancestor")


def create_probe(repo: Path, output: Path) -> Path:
    """Create the fixture and return its root without running tests."""
    if not repo.is_absolute() or repo.resolve() != repo or not repo.is_dir():
        raise ValueError("probe repo must be a physical absolute directory")
    _validate_output(repo, output)
    output.mkdir(parents=True)
    (output / ".empty-hooks").mkdir()
    _workspace(output)
    _ui(output)
    _worker(output)
    subprocess.run(("cargo", "generate-lockfile", "--offline"), cwd=output, check=True)
    _configs(repo, output)
    shutil.copy2(output / "Cargo.lock", output / "scripts/screenshot/Cargo.lock")
    _write(output / ".gitignore", "/target\n/tmp\n")
    _commit(output)
    return output


def create_failure(root: Path) -> Path:
    """Commit one intentional fixture failure without touching the real repository."""
    source = root / "crates/katana-core/src/lib.rs"
    content = source.read_text(encoding="utf-8")
    marker = "assert_eq!(super::double(3), 6);"
    if content.count(marker) != 1:
        raise ValueError("fixture failure marker is not unique")
    source.write_text(content.replace(marker, marker.replace(", 6", ", 7")), encoding="utf-8")
    _commit(root)
    return root


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    output = args.output or args.repo / "tmp/nextest-shadow-probe" / uuid.uuid4().hex
    create_probe(args.repo.resolve(), output.resolve())
    print(output.resolve())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
