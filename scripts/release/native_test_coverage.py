"""Use cargo-llvm-cov's supported external-test instrumentation, without reuse."""

import hashlib
import json
from pathlib import Path
import subprocess

from native_test_coverage_environment import KNOWN_KEYS, MAX_OUTPUT_BYTES, parse_show_environment

SHOW_ENV = ("cargo", "llvm-cov", "show-env", "--sh", "--no-cfg-coverage")


def coverage_test_command(argv: list[str]) -> list[str]:
    boundary = argv.index("--") if "--" in argv else len(argv)
    arguments = argv[2:boundary]
    if argv[:2] != ["cargo", "llvm-cov"] or any(
            arguments.count(flag) != 1 for flag in ("--no-report", "--no-cfg-coverage")):
        raise ValueError("coverage inventory requires the explicit no-report/no-cfg-coverage phase")
    if not any(flag in arguments for flag in ("--lib", "--bins", "--test")):
        raise ValueError("coverage inventory requires an explicit native test selection")
    return ["cargo", "test", *(argument for argument in arguments
                              if argument not in {"--no-report", "--no-cfg-coverage"}),
            *argv[boundary:]]


def _tool_output(argv: tuple[str, ...], root: Path, environment: dict[str, str],
                 limit: int = MAX_OUTPUT_BYTES) -> bytes:
    try:
        result = subprocess.run(argv, cwd=root, env=environment, check=True,
                                capture_output=True, timeout=30)
    except (OSError, subprocess.SubprocessError) as error:
        raise ValueError("cargo-llvm-cov instrumentation setup failed") from error
    if len(result.stdout) > limit:
        raise ValueError("cargo-llvm-cov setup output exceeds the metadata limit")
    return result.stdout


def _target_environment(root: Path, parent: dict[str, str]) -> dict[str, str]:
    metadata = json.loads(_tool_output(
        ("cargo", "metadata", "--no-deps", "--format-version", "1"), root, parent,
        limit=1024 * 1024))
    target = parent.get("CARGO_LLVM_COV_TARGET_DIR") or str(
        Path(metadata["target_directory"]) / "llvm-cov-target")
    default_build = target if parent.get("CARGO_LLVM_COV_TARGET_DIR") else str(
        Path(metadata.get("build_directory", metadata["target_directory"])) / "llvm-cov-target")
    configured_build = (metadata.get("build_directory", metadata["target_directory"])
                        != metadata["target_directory"] or bool(parent.get("CARGO_BUILD_BUILD_DIR")))
    build = (parent.get("CARGO_LLVM_COV_BUILD_DIR") or default_build) if configured_build else target
    # WHY: show-envが返すmetadataとprofile/reportの保存先を同一にする。
    return parent | {"CARGO_TARGET_DIR": str((root / target).resolve()),
                     "CARGO_BUILD_BUILD_DIR": str((root / build).resolve()),
                     "CARGO_LLVM_COV_TARGET_DIR": str((root / target).resolve()),
                     "CARGO_LLVM_COV_BUILD_DIR": str((root / build).resolve())}


def coverage_environment(root: Path, parent: dict[str, str]) -> tuple[dict[str, str], dict]:
    prepared = _target_environment(root, parent)
    raw = _tool_output(SHOW_ENV, root, prepared)
    environment = parse_show_environment(raw, prepared)
    version = _tool_output(("cargo", "llvm-cov", "--version"), root, parent).decode("utf-8").strip()
    return environment, {
        "show_env_argv": list(SHOW_ENV), "show_env_sha256": hashlib.sha256(raw).hexdigest(),
        "version": version,
        "environment": {key: environment[key] for key in sorted(
            KNOWN_KEYS | {"CARGO_TARGET_DIR", "CARGO_BUILD_BUILD_DIR"}) if key in environment},
    }
