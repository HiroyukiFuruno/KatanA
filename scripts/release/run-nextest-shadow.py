#!/usr/bin/env python3
"""Run an opt-in comparison candidate; never replace required release checks."""

import argparse
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import uuid

from native_test_run_identity import file_digest, source_identity
from nextest_shadow_cache import validate_build_cache
from nextest_shadow_execution import (
    command_context, execute_stage, phase_environment, prepare_output, snapshot_worker,
    shadow_environment_identity, verify_tools, write_receipt,
)
from nextest_shadow_plan import phase_plan, shadow_contract
from nextest_shadow_junit import inspect_report, record_run_uuid, write_phase_config
from nextest_shadow_records import collect_records


def worker_command(cargo: list[str]) -> list[str]:
    return [*cargo, "build", "--locked", "-p", "katana-ui", "--bin", "kdv-office-worker"]


def prepare_phase_junit(output: Path, base: dict[str, str], phase: dict) -> dict:
    junit = write_phase_config(output, phase["phase"], base["KATANA_SHADOW_PROFILE_ID"],
                               Path(phase["config_template"]))
    if (junit["config_path"] != phase["junit_config"]
            or junit["report_path"] != phase["junit_report"]):
        raise ValueError("shadow JUnit phase paths disagree with the plan")
    argv = phase["argv"]
    if ("--config-file" not in argv or argv.index("--config-file") + 1 >= len(argv)
            or argv[argv.index("--config-file") + 1] != junit["config_path"]):
        raise ValueError("shadow JUnit config is not selected by the phase")
    return junit


def run_phase(repo: Path, output: Path, cargo: list[str], base: dict[str, str],
              phase: dict) -> dict:
    directory = output / phase["phase"]
    environment = phase_environment(base, directory)
    phase_identity = shadow_environment_identity(repo, cargo, environment)
    junit = prepare_phase_junit(output, base, phase)
    argv = phase["argv"]
    result = execute_stage(repo, directory, [*cargo, *argv], environment)
    try:
        result["record_files"] = collect_records(directory / "nextest-state")
        result["junit"] = {**inspect_report(Path(junit["report_path"]),
                                             record_run_uuid(result["record_files"])),
                           "config_path": junit["config_path"],
                           "config_sha256": junit["config_sha256"]}
        if result["exit_code"] == 0 and any(result["junit"][key] for key in
                                             ("failure_count", "error_count")):
            raise ValueError("shadow JUnit failures contradict the successful child exit")
    except (OSError, ValueError) as error:
        if "record_files" not in result:
            result["record_unavailable"] = str(error)
        else:
            result["junit_unavailable"] = str(error)
    result["environment_identity"] = phase_identity
    result["environment_identity"]["explicit"] = {
        key: environment.get(key) for key in
        ("NEXTEST_STATE_DIR", "LLVM_PROFILE_FILE_NAME", "KATANA_KDV_OFFICE_WORKER")}
    write_receipt(directory / "phase.json", result)
    return {"phase": phase["phase"], **result}


def initial_receipt(repo: Path, output: Path, cargo: list[str], environment: dict[str, str],
                    tool_versions: dict[str, str], identity: dict) -> dict:
    contract = shadow_contract(repo)
    config_hashes = {key: file_digest(Path(contract[key]))
                     for key in ("config_file", "user_config_file")}
    return {"schema": 1, "diagnostic_only": True, **contract,
            "producer_completed": False, "report_checked": False,
            "source_before": source_identity(repo), "output": str(output),
            "cargo_argv": cargo, "tool_versions": tool_versions,
            "environment_identity": identity,
            "build_cache_target": environment["CARGO_TARGET_DIR"],
            "profile_namespace": environment["KATANA_SHADOW_PROFILE_ID"],
            "producer_sha256": {name: file_digest(Path(__file__).parent / name) for name in
                                ("run-nextest-shadow.py", "nextest_shadow_execution.py",
                                 "nextest_shadow_plan.py", "nextest_shadow_records.py",
                                 "nextest_shadow_cache.py", "nextest_shadow_junit.py",
                                 "nextest_shadow_junit_io.py")},
            "config_sha256": config_hashes, "stages": [],
            "limitations": ["nextest test-per-process differs from libtest",
                            "normal/doctest/fixture checks remain required",
                            "record files are not protected-CI evidence",
                            "serialized shadow is not a timing improvement claim"]}


def collect_shadow(repo: Path, output: Path, cargo: list[str],
                   environment: dict[str, str], receipt: dict) -> int:
    worker = execute_stage(repo, output / "office-worker", worker_command(cargo), environment)
    receipt["stages"].append({"phase": "office-worker", **worker})
    if worker["exit_code"]:
        return worker["exit_code"]
    worker_path = Path(environment["CARGO_TARGET_DIR"]) / "debug/kdv-office-worker"
    if not worker_path.is_file() or not os.access(worker_path, os.X_OK):
        raise ValueError("shadow Office worker is missing or not executable")
    snapshot_path = output / "office-worker/worker-snapshot"
    snapshot_sha256 = snapshot_worker(worker_path, snapshot_path)
    receipt.update({"worker_sha256_at_build": file_digest(worker_path),
                    "worker_path": str(snapshot_path),
                    "worker_snapshot_sha256": snapshot_sha256})
    environment["KATANA_KDV_OFFICE_WORKER"] = str(snapshot_path)
    for phase in phase_plan(repo, output, Path(environment["CARGO_TARGET_DIR"])):
        result = run_phase(repo, output, cargo, environment, phase)
        receipt["stages"].append(result)
        if result["exit_code"]:
            return result["exit_code"]
        if result.get("record_unavailable") or result.get("junit_unavailable"):
            return 2
    receipt["producer_completed"] = True
    return 0


def run_shadow(repo: Path, output: Path, build_cache_target: Path | None = None) -> int:
    if platform.system() != "Darwin":
        raise ValueError("this shadow candidate is macOS-only; required OS checks remain unchanged")
    target = validate_build_cache(repo, build_cache_target) if build_cache_target else None
    cargo, environment = command_context(output, target)
    tool_versions = verify_tools(cargo, environment, repo)
    identity = shadow_environment_identity(repo, cargo, environment)
    receipt = initial_receipt(repo, output, cargo, environment, tool_versions, identity)
    if receipt["source_before"].get("clean") is not True:
        raise ValueError("shadow requires a clean source tree; no build was started")
    prepare_output(repo, output)
    status = 2
    try:
        status = collect_shadow(repo, output, cargo, environment, receipt)
    finally:
        after = source_identity(repo)
        stable = after == receipt["source_before"] and after.get("clean") is True
        if not stable:
            receipt["producer_completed"] = False
            status = status or 2
        receipt.update({"exit_code": status, "source_after": after,
                        "source_stable": stable})
        write_receipt(output / "shadow.json", receipt)
    return 128 - status if status < 0 else status


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--output", type=Path)
    parser.add_argument("--build-cache-target", type=Path,
                        help="reuse only a validated earlier shadow Cargo build cache; rerun all tests")
    parser.add_argument("--plan", action="store_true")
    args = parser.parse_args()
    repo = args.repo
    output = args.output or repo / "tmp/nextest-shadow" / str(uuid.uuid4())
    try:
        if args.plan:
            print(json.dumps({"phases": phase_plan(repo, output, args.build_cache_target),
                              **shadow_contract(repo)}, indent=2))
            return 0
        return run_shadow(repo, output, args.build_cache_target)
    except (OSError, ValueError, TypeError, subprocess.SubprocessError) as error:
        print(f"nextest shadow failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
