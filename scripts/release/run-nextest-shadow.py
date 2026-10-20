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
from nextest_shadow_execution import (
    command_context, execute_stage, phase_environment, prepare_output, snapshot_worker,
    verify_tools, write_receipt,
)
from nextest_shadow_plan import phase_plan, shadow_contract
from nextest_shadow_records import collect_records


def worker_command(cargo: list[str]) -> list[str]:
    return [*cargo, "build", "--locked", "-p", "katana-ui", "--bin", "kdv-office-worker"]


def run_phase(repo: Path, output: Path, cargo: list[str], base: dict[str, str],
              phase: dict) -> dict:
    directory = output / phase["phase"]
    environment = phase_environment(base, directory)
    result = execute_stage(repo, directory, [*cargo, *phase["argv"]], environment)
    try:
        result["record_files"] = collect_records(directory / "nextest-state")
    except ValueError as error:
        result["record_unavailable"] = str(error)
    write_receipt(directory / "phase.json", result)
    return {"phase": phase["phase"], **result}


def initial_receipt(repo: Path, output: Path) -> dict:
    contract = shadow_contract(repo)
    config_hashes = {key: file_digest(Path(contract[key]))
                     for key in ("config_file", "user_config_file")}
    return {"schema": 1, "diagnostic_only": True, **contract,
            "producer_completed": False, "report_checked": False,
            "source_before": source_identity(repo), "output": str(output),
            "producer_sha256": {name: file_digest(Path(__file__).parent / name) for name in
                                ("run-nextest-shadow.py", "nextest_shadow_execution.py",
                                 "nextest_shadow_plan.py", "nextest_shadow_records.py")},
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
    worker_path = output / "coverage-target/debug/kdv-office-worker"
    if not worker_path.is_file() or not os.access(worker_path, os.X_OK):
        raise ValueError("shadow Office worker is missing or not executable")
    snapshot_path = output / "office-worker/worker-snapshot"
    snapshot_sha256 = snapshot_worker(worker_path, snapshot_path)
    receipt.update({"worker_sha256_at_build": file_digest(worker_path),
                    "worker_path": str(snapshot_path),
                    "worker_snapshot_sha256": snapshot_sha256})
    environment["KATANA_KDV_OFFICE_WORKER"] = str(snapshot_path)
    for phase in phase_plan(repo, output):
        result = run_phase(repo, output, cargo, environment, phase)
        receipt["stages"].append(result)
        if result["exit_code"]:
            return result["exit_code"]
        if result.get("record_unavailable"):
            return 2
    receipt["producer_completed"] = True
    return 0


def run_shadow(repo: Path, output: Path) -> int:
    if platform.system() != "Darwin":
        raise ValueError("this shadow candidate is macOS-only; required OS checks remain unchanged")
    cargo, environment = command_context(output)
    verify_tools(cargo, environment, repo)
    receipt = initial_receipt(repo, output)
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
    parser.add_argument("--plan", action="store_true")
    args = parser.parse_args()
    repo = args.repo
    output = args.output or repo / "tmp/nextest-shadow" / str(uuid.uuid4())
    try:
        if args.plan:
            print(json.dumps({"phases": phase_plan(repo, output), **shadow_contract(repo)}, indent=2))
            return 0
        return run_shadow(repo, output)
    except (OSError, ValueError, TypeError, subprocess.SubprocessError) as error:
        print(f"nextest shadow failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
