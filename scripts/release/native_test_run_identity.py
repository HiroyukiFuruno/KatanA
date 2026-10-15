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
    "CARGO_BUILD_TARGET", "CARGO_TARGET_DIR", "CARGO_LLVM_COV_TARGET_DIR",
    "CARGO_BUILD_JOBS", "JOBS", "RUST_TEST_THREADS", "DEBUG",
    "KATANA_KDV_OFFICE_WORKER", "KATANA_HTML_FAILURE_SAMPLE",
)


def file_digest(path: Path) -> str:
    if path.is_symlink() or not path.is_file():
        raise ValueError("inventory input is not a regular non-symlink file")
    result = hashlib.sha256()
    with path.open("rb") as stream:
        while chunk := stream.read(65536):
            result.update(chunk)
    return result.hexdigest()


def _version(command: list[str]) -> str | None:
    try:
        result = subprocess.run(command, check=True, capture_output=True, timeout=10)
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


def environment_identity(root: Path) -> dict:
    environment = {key: os.environ.get(key) for key in INPUT_ENVIRONMENT}
    worker = environment["KATANA_KDV_OFFICE_WORKER"]
    try:
        worker_sha = file_digest(root / worker) if worker else None
    except (OSError, ValueError):
        worker_sha = None
    return {"os": platform.platform(), "machine": platform.machine(),
            "rustc": _version(["rustc", "-vV"]), "cargo": _version(["cargo", "-V"]),
            "environment": environment, "worker_sha256": worker_sha}
