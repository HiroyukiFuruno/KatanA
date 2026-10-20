"""Execute isolated diagnostic stages without granting test-reuse authority."""

from datetime import datetime, timezone
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import time

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "ci"))
from cargo_clean_guard import cargo_command
from native_test_run_identity import INPUT_ENVIRONMENT, file_digest

NEXTEST_VERSION = "0.9.148"
LLVM_COV_VERSION = "0.9.1"
READONLY_EXECUTABLE_MODE = 0o555


def snapshot_worker(source: Path, destination: Path) -> str:
    digest = file_digest(source)
    if not os.access(source, os.X_OK):
        raise ValueError("shadow worker is not executable")
    # coverageによる同名binaryの再linkから非instrumented workerを分離する。
    with source.open("rb") as original, destination.open("xb") as snapshot:
        shutil.copyfileobj(original, snapshot, length=65536)
    destination.chmod(READONLY_EXECUTABLE_MODE)
    if file_digest(source) != digest or file_digest(destination) != digest:
        raise ValueError("shadow worker changed while taking snapshot")
    return digest


def verify_tools(cargo: list[str], environment: dict[str, str], repo: Path) -> dict[str, str]:
    versions = {}
    for tool, version in (("nextest", NEXTEST_VERSION), ("llvm-cov", LLVM_COV_VERSION)):
        result = subprocess.run([*cargo, tool, "--version"], cwd=repo, env=environment,
                                capture_output=True, text=True, timeout=10, check=False)
        first = result.stdout.splitlines()[0] if result.stdout else ""
        if result.returncode or first.split()[:2] != [f"cargo-{tool}", version]:
            raise ValueError(f"shadow requires cargo-{tool} {version}: {first}")
        versions[tool] = first
    return versions


def shadow_environment_identity(repo: Path, cargo: list[str], environment: dict[str, str]) -> dict:
    observed = {}
    rustc = environment["RUSTC"] if "RUSTC" in environment else "rustc"
    for name, command in (("cargo", [*cargo, "-V"]), ("rustc", [rustc, "-vV"])):
        try:
            result = subprocess.run(command, cwd=repo, env=environment,
                                    capture_output=True, text=True, timeout=10, check=False)
        except (OSError, subprocess.SubprocessError) as error:
            raise ValueError(f"shadow {name} identity unavailable") from error
        if result.returncode or not result.stdout.strip():
            raise ValueError(f"shadow {name} identity unavailable")
        observed[name] = result.stdout.strip()
    worker = environment.get("KATANA_KDV_OFFICE_WORKER")
    try:
        worker_sha = file_digest(repo / worker) if worker else None
    except (OSError, ValueError) as error:
        raise ValueError("shadow worker identity unavailable") from error
    return {"os": platform.platform(), "machine": platform.machine(),
            "toolchain": observed,
            "environment": {key: environment.get(key) for key in INPUT_ENVIRONMENT},
            "worker_sha256": worker_sha}


def command_context(output: Path) -> tuple[list[str], dict[str, str]]:
    cargo, environment = cargo_command(os.environ.get("CARGO", "cargo"))
    environment.update({"CARGO_TARGET_DIR": str(output / "coverage-target"),
                        "CARGO_LLVM_COV_TARGET_DIR": str(output / "coverage-target"),
                        "CARGO_BUILD_JOBS": "1", "NEXTEST_EXPERIMENTAL_RECORD": "1"})
    for name in ("LLVM_PROFILE_FILE", "LLVM_PROFILE_FILE_NAME"):
        environment.pop(name, None)
    return cargo, environment


def phase_environment(base: dict[str, str], directory: Path) -> dict[str, str]:
    environment = dict(base)
    environment["NEXTEST_STATE_DIR"] = str(directory / "nextest-state")
    environment["LLVM_PROFILE_FILE_NAME"] = f"katana-shadow-{directory.name}-%m-%p.profraw"
    encoded = environment.get("CARGO_ENCODED_RUSTFLAGS")
    if encoded is not None:
        environment["CARGO_ENCODED_RUSTFLAGS"] = (
            encoded + "\x1f" if encoded else "") + "--cfg\x1fcoverage"
    else:
        environment["RUSTFLAGS"] = environment.get("RUSTFLAGS", "") + " --cfg coverage"
    return environment


def execute_stage(repo: Path, directory: Path, command: list[str],
                  environment: dict[str, str]) -> dict:
    directory.mkdir()
    started_at = datetime.now(timezone.utc).isoformat()
    started = time.perf_counter()
    result = subprocess.run(command, cwd=repo, env=environment, check=False)
    payload = {"argv": command, "exit_code": result.returncode,
               "started_at": started_at, "finished_at": datetime.now(timezone.utc).isoformat(),
               "elapsed_seconds": time.perf_counter() - started}
    write_receipt(directory / "command.json", payload)
    return payload


def write_receipt(path: Path, payload: dict) -> None:
    try:
        with path.open("x", encoding="utf-8") as stream:
            json.dump(payload, stream, indent=2, sort_keys=True)
            stream.write("\n")
    except OSError as error:
        if not payload.get("exit_code"):
            raise
        print(f"shadow receipt unavailable; child failure preserved: {error}", file=sys.stderr)


def prepare_output(repo: Path, output: Path) -> None:
    if not repo.is_absolute() or repo.resolve() != repo or not repo.is_dir():
        raise ValueError("shadow repository must be a physical absolute directory")
    if not output.is_absolute() or ".." in output.parts or repo / "tmp" not in output.parents:
        raise ValueError("shadow output must be a new directory below repository tmp")
    for parent in (output, *output.parents):
        if parent.is_symlink() or (parent.exists() and not parent.is_dir()):
            raise ValueError("shadow output has a symlink or non-directory ancestor")
    output.mkdir(parents=True, exist_ok=False)
