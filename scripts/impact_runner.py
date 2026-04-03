#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from collections import deque
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
WORKSPACE_CRATES = {
    "katana-core": "crates/katana-core",
    "katana-linter": "crates/katana-linter",
    "katana-platform": "crates/katana-platform",
    "katana-ui": "crates/katana-ui",
}
SERIAL_UI_MODULES = {
    "diagram_rendering.rs",
    "integration.rs",
    "tree_layout.rs",
}
PARALLEL_UI_MODULES = {
    "app_state.rs",
    "command_palette.rs",
    "font_bridge.rs",
    "font_realtime.rs",
    "i18n.rs",
    "overlap_checker.rs",
    "preview_pane.rs",
    "settings_window.rs",
    "shell_logic.rs",
    "theme.rs",
    "theme_bridge.rs",
    "theme_rendering_sync.rs",
    "underline_rendering.rs",
}
PARALLEL_UI_TARGET = "ui_integration_parallel"
SERIAL_UI_TARGET = "ui_integration_serial"
FIXTURE_UI_TARGET = "ui_integration_fixture"


@dataclass
class UiScope:
    lib_bins: bool = False
    parallel: bool = False
    serial: bool = False
    fixture: bool = False

    def any(self) -> bool:
        return self.lib_bins or self.parallel or self.serial or self.fixture


def git(*args: str, check: bool = True) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if check and result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or result.stdout.strip() or "git command failed")
    return result.stdout.strip()

def auto_base() -> str:
    upstream = subprocess.run(
        ["git", "rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{upstream}"],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if upstream.returncode == 0:
        upstream_ref = upstream.stdout.strip()
        return git("merge-base", "HEAD", upstream_ref)

    head_parent = subprocess.run(
        ["git", "rev-parse", "--verify", "HEAD~1"],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if head_parent.returncode == 0:
        return head_parent.stdout.strip()

    return "4b825dc642cb6eb9a060e54bf8d69288fbee4904"


def changed_files(base: str) -> list[str]:
    changed: set[str] = set()
    commands = [
        ["diff", "--name-only", "--diff-filter=ACMR", f"{base}...HEAD"],
        ["diff", "--name-only", "--diff-filter=ACMR", "--cached"],
        ["diff", "--name-only", "--diff-filter=ACMR"],
        ["ls-files", "--others", "--exclude-standard"],
    ]
    for args in commands:
        out = git(*args, check=False)
        for line in out.splitlines():
            line = line.strip()
            if line:
                changed.add(line)
    return sorted(changed)


def load_workspace_graph() -> tuple[list[str], dict[str, set[str]]]:
    metadata = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    data = json.loads(metadata.stdout)
    names = [pkg["name"] for pkg in data["packages"] if pkg["name"] in WORKSPACE_CRATES]
    reverse: dict[str, set[str]] = {name: set() for name in names}

    for pkg in data["packages"]:
        name = pkg["name"]
        if name not in reverse:
            continue
        for dep in pkg.get("dependencies", []):
            dep_name = dep["name"]
            if dep_name in reverse:
                reverse[dep_name].add(name)
    return sorted(names), reverse


def closure(initial: set[str], reverse: dict[str, set[str]]) -> list[str]:
    seen = set(initial)
    queue = deque(initial)
    while queue:
        current = queue.popleft()
        for dependent in reverse.get(current, set()):
            if dependent not in seen:
                seen.add(dependent)
                queue.append(dependent)
    return sorted(seen)


def is_non_verification_change(path: str) -> bool:
    return (
        path.startswith("docs/")
        or path.startswith("openspec/")
        or path.endswith(".md")
        or path.endswith(".txt")
        or path == "LICENSE"
        or path == ".gitignore"
        or path == ".DS_Store"
    )


def classify(
    files: list[str], include_fixture: bool, workspace_packages: list[str], reverse: dict[str, set[str]]
) -> tuple[list[str], UiScope]:
    direct_packages: set[str] = set()
    force_all = False
    ui_scope = UiScope()
    relevant = False

    for path in files:
        if path in {"Cargo.toml", "Cargo.lock", ".clippy.toml", "rustfmt.toml", "Makefile", "makefile", "lefthook.yml"}:
            force_all = True
            relevant = True
            continue
        if path.startswith(".github/") or path.startswith("scripts/"):
            force_all = True
            relevant = True
            continue
        if path.startswith("assets/fixtures/"):
            direct_packages.add("katana-ui")
            ui_scope.lib_bins = True
            ui_scope.parallel = True
            ui_scope.serial = True
            ui_scope.fixture = True
            relevant = True
            continue
        if path.startswith("assets/"):
            direct_packages.add("katana-ui")
            ui_scope.lib_bins = True
            ui_scope.parallel = True
            ui_scope.serial = True
            relevant = True
            continue
        if path.startswith("crates/"):
            parts = Path(path).parts
            if len(parts) < 2:
                continue
            crate_dir = "/".join(parts[:2])
            package = next((name for name, rel in WORKSPACE_CRATES.items() if rel == crate_dir), None)
            if package is None:
                force_all = True
                relevant = True
                continue
            direct_packages.add(package)
            relevant = True
            if package == "katana-ui":
                if path.startswith("crates/katana-ui/tests/integration/"):
                    name = Path(path).name
                    if name == "sample_fixture_tests.rs":
                        ui_scope.fixture = True
                    elif name in SERIAL_UI_MODULES:
                        ui_scope.serial = True
                    elif name in PARALLEL_UI_MODULES:
                        ui_scope.parallel = True
                elif path.endswith(f"{PARALLEL_UI_TARGET}.rs"):
                    ui_scope.parallel = True
                elif path.endswith(f"{SERIAL_UI_TARGET}.rs"):
                    ui_scope.serial = True
                elif path.endswith(f"{FIXTURE_UI_TARGET}.rs"):
                    ui_scope.fixture = True
                else:
                    ui_scope.lib_bins = True
                    ui_scope.parallel = True
                    ui_scope.serial = True
            continue
        if not is_non_verification_change(path):
            force_all = True
            relevant = True

    if not relevant:
        return [], UiScope()

    impacted = closure(set(workspace_packages) if force_all else direct_packages, reverse)

    if "katana-ui" in impacted and not ui_scope.any():
        ui_scope.lib_bins = True
        ui_scope.parallel = True
        ui_scope.serial = True

    if not include_fixture:
        ui_scope.fixture = False

    return impacted, ui_scope


def run(cmd: list[str]) -> None:
    print(f"[impact] {' '.join(cmd)}")
    subprocess.run(cmd, cwd=ROOT, check=True)


def run_clippy(packages: list[str]) -> int:
    if not packages:
        print("[impact] no impacted Rust packages, skipping clippy")
        return 0
    cmd = ["cargo", "clippy"]
    for package in packages:
        cmd.extend(["-p", package])
    cmd.extend(["--", "-D", "warnings"])
    run(cmd)
    return 0


def run_tests(packages: list[str], ui_scope: UiScope) -> int:
    other_packages = [pkg for pkg in packages if pkg != "katana-ui"]
    if not other_packages and not ui_scope.any():
        print("[impact] no impacted Rust test targets, skipping tests")
        return 0

    for package in other_packages:
        run(["cargo", "test", "-p", package])

    if ui_scope.lib_bins:
        run(["cargo", "test", "-p", "katana-ui", "--lib", "--bins"])
    if ui_scope.parallel:
        run(
            [
                "cargo",
                "test",
                "-p",
                "katana-ui",
                "--test",
                PARALLEL_UI_TARGET,
            ]
        )
    if ui_scope.serial:
        run(
            [
                "cargo",
                "test",
                "-p",
                "katana-ui",
                "--test",
                SERIAL_UI_TARGET,
                "--",
                "--test-threads=1",
            ]
        )
    if ui_scope.fixture:
        run(
            [
                "cargo",
                "test",
                "-p",
                "katana-ui",
                "--test",
                FIXTURE_UI_TARGET,
                "--",
                "--test-threads=1",
            ]
        )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Run impacted Rust verification commands.")
    parser.add_argument("command", choices=["summary", "clippy", "test"])
    parser.add_argument("--base", default="auto", help="git base commit/range start (default: auto)")
    parser.add_argument(
        "--include-fixture",
        action="store_true",
        help="include the heavy katana-ui fixture integration bucket",
    )
    args = parser.parse_args()

    base = auto_base() if args.base == "auto" else args.base
    files = changed_files(base)
    workspace_packages, reverse = load_workspace_graph()
    packages, ui_scope = classify(files, args.include_fixture, workspace_packages, reverse)

    if args.command == "summary":
        print(f"base: {base}")
        print("changed_files:")
        for path in files:
            print(f"  - {path}")
        print(f"impacted_packages: {packages}")
        print(
            "ui_scope:"
            f" lib_bins={ui_scope.lib_bins}"
            f" parallel={ui_scope.parallel}"
            f" serial={ui_scope.serial}"
            f" fixture={ui_scope.fixture}"
        )
        return 0

    print(f"[impact] base {base}")
    if files:
        print(f"[impact] changed files: {len(files)}")
    else:
        print("[impact] changed files: 0")
    print(f"[impact] impacted packages: {', '.join(packages) if packages else '(none)'}")
    print(
        "[impact] ui buckets:"
        f" lib/bins={ui_scope.lib_bins}"
        f" parallel={ui_scope.parallel}"
        f" serial={ui_scope.serial}"
        f" fixture={ui_scope.fixture}"
    )

    if args.command == "clippy":
        return run_clippy(packages)
    return run_tests(packages, ui_scope)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        raise SystemExit(130)
    except subprocess.CalledProcessError as exc:
        raise SystemExit(exc.returncode)
    except Exception as exc:  # pragma: no cover - CLI error path
        print(f"[impact] error: {exc}", file=sys.stderr)
        raise SystemExit(1)
