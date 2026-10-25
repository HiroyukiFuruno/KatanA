"""Observed inputs for diagnostics, never authorization to reuse CI results."""

import hashlib
import os
from pathlib import Path
import platform
import subprocess

from bounded_git_output import bounded_git
from downstream_git_binding import GitBindingError

LOCK_PATHS = ("Cargo.lock", "scripts/screenshot/Cargo.lock")
INPUT_ENVIRONMENT = (
    "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTDOCFLAGS", "RUSTC", "RUSTDOC",
    "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "RUSTUP_TOOLCHAIN",
    "CARGO_BUILD_TARGET", "CARGO_TARGET_DIR", "CARGO_BUILD_BUILD_DIR", "CARGO_LLVM_COV_TARGET_DIR",
    "CARGO_BUILD_JOBS", "JOBS", "RUST_TEST_THREADS", "DEBUG", "CARGO_INCREMENTAL",
    "CARGO_PROFILE_DEV_DEBUG", "CARGO_PROFILE_TEST_DEBUG",
    "CARGO_PROFILE_DEV_STRIP", "CARGO_PROFILE_TEST_STRIP",
    "KATANA_KDV_OFFICE_WORKER", "KATANA_HTML_FAILURE_SAMPLE",
    "LLVM_PROFILE_FILE", "CARGO_LLVM_COV", "CARGO_LLVM_COV_SHOW_ENV",
    "CARGO_LLVM_COV_BUILD_DIR", "__CARGO_LLVM_COV_RUSTC_WRAPPER",
    "__CARGO_LLVM_COV_RUSTC_WRAPPER_RUSTFLAGS", "__CARGO_LLVM_COV_RUSTC_WRAPPER_CRATE_NAMES",
    "__CARGO_LLVM_COV_RUSTC_WRAPPER_COVERAGE_TARGET", "__CARGO_LLVM_COV_RUSTC_WRAPPER_HOST",
    "__CARGO_LLVM_COV_RUSTC_WRAPPER_PRE_EXISTING",
)


def file_digest(path: Path) -> str:
    if path.is_symlink() or not path.is_file():
        raise ValueError("inventory input is not a regular non-symlink file")
    result = hashlib.sha256()
    with path.open("rb") as stream:
        while chunk := stream.read(65536):
            result.update(chunk)
    return result.hexdigest()


def _version(command: list[str], cwd: Path, environment: dict[str, str]) -> str | None:
    try:
        result = subprocess.run(command, cwd=cwd, env=environment, check=True,
                                capture_output=True, timeout=10)
        return result.stdout.decode("utf-8").strip()
    except (OSError, UnicodeError, subprocess.SubprocessError):
        return None


def source_identity(root: Path) -> dict:
    try:
        head = bounded_git(root, ["rev-parse", "HEAD"]).decode().strip()
        tree = bounded_git(root, ["rev-parse", "HEAD^{tree}"]).decode().strip()
        status = bounded_git(root, ["status", "--porcelain=v1", "-z", "--untracked-files=all"])
        locks = {name: file_digest(root / name) for name in LOCK_PATHS}
        return {"head": head, "tree": tree, "clean": not status,
                "status_sha256": hashlib.sha256(status).hexdigest(), "locks": locks}
    except (OSError, ValueError, UnicodeError, GitBindingError) as error:
        return {"clean": False, "unavailable": str(error)}


def environment_identity(root: Path, execution_environment: dict[str, str] | None = None) -> dict:
    inherited = os.environ if execution_environment is None else execution_environment
    environment = {key: inherited.get(key) for key in INPUT_ENVIRONMENT}
    worker = environment["KATANA_KDV_OFFICE_WORKER"]
    try:
        worker_sha = file_digest(root / worker) if worker else None
    except (OSError, ValueError):
        worker_sha = None
    return {"os": platform.platform(), "machine": platform.machine(),
            "rustc": _version([inherited["RUSTC"] if "RUSTC" in inherited else "rustc", "-vV"],
                               root, inherited),
            "cargo": _version(["cargo", "-V"], root, inherited),
            "environment": environment, "worker_sha256": worker_sha}
