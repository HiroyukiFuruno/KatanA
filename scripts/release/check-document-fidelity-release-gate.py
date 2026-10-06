#!/usr/bin/env python3
"""Bind the v0.22.42 release gate to the active document-fidelity change."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path


TARGET_VERSION = "0.22.42"
CHANGE_NAME = "post-v0-22-41-document-fidelity-regressions"
MODES = {
    "source",
    "packaged-host",
    "strict",
    "pr-bootstrap",
    "release-artifact-pending",
    "post-release-evidence",
}
BOOTSTRAP_ALLOWED = {
    "1.2",
    "1.6",
    *{f"3.{number}" for number in range(2, 8)},
    *{f"4.{number}" for number in range(3, 10)},
    *{f"4.{number}" for number in range(11, 18)},
    "4.23",
    "5.1",
    "5.2",
    "5.3",
    "6.2",
    "6.3",
}
SOURCE_UPSTREAM_DEFERRED = {"3.2"}
SOURCE_HOST_IMPLEMENTATION_MARKERS = {
    "3.3": "3.9",
    "3.4": "3.10",
    "3.5": "3.11",
    "3.6": "3.12",
    "3.7": "3.13",
    "5.1": "5.4",
}
SOURCE_EVIDENCE_GENERATION = {
    "1.2",
    "1.6",
    "4.3",
    "4.4",
    "4.5",
    "4.7",
    "4.11",
    "4.12",
    "4.13",
    "4.15",
    "4.23",
    "5.2",
    "6.2",
}
SOURCE_POST_PUBLICATION = {"4.8", "4.9", "5.3", "6.3"}
PACKAGED_HOST_ALLOWED = (
    SOURCE_UPSTREAM_DEFERRED
    | set(SOURCE_HOST_IMPLEMENTATION_MARKERS)
    | SOURCE_POST_PUBLICATION
)
SOURCE_ALLOWED = (
    SOURCE_UPSTREAM_DEFERRED
    | set(SOURCE_HOST_IMPLEMENTATION_MARKERS)
    | SOURCE_EVIDENCE_GENERATION
    | SOURCE_POST_PUBLICATION
)
CRITICAL_REQUIRED = {
    "1.2",
    "1.6",
    *{f"2.{number}" for number in range(1, 6)},
    "2.6",
    *{f"3.{number}" for number in range(2, 9)},
    *{f"4.{number}" for number in (3, 4, 5, 7, 13, 14, 16, 17, 23)},
    *{f"5.{number}" for number in range(1, 4)},
    "6.2",
}
TASK_ID_PATTERN = re.compile(r"^\s*-\s*\[[^\]]\]\s+(\d+\.\d+)\b")
COMPLETED_TASK_ID_PATTERN = re.compile(r"^\s*-\s*\[x\]\s+(\d+\.\d+)\b")


class GateError(RuntimeError):
    """Raised when the release binding cannot be established safely."""


def task_file(repository_root: Path) -> Path:
    active = repository_root / "openspec" / "changes" / CHANGE_NAME
    if active.exists():
        path = active / "tasks.md"
        if not path.is_file():
            raise GateError(f"active change tasks file is missing: {path}")
        return path

    archive_root = repository_root / "openspec" / "changes" / "archive"
    archived = sorted(
        path for path in archive_root.glob(f"*-{CHANGE_NAME}") if path.is_dir()
    )
    if len(archived) != 1:
        raise GateError(
            f"expected exactly one archived {CHANGE_NAME} directory, found {len(archived)}"
        )
    path = archived[0] / "tasks.md"
    if not path.is_file():
        raise GateError(f"archived change tasks file is missing: {path}")
    return path


def declared_task_ids(tasks: Path) -> set[str]:
    return {
        match.group(1)
        for line in tasks.read_text(encoding="utf-8").splitlines()
        if (match := TASK_ID_PATTERN.match(line)) is not None
    }


def completed_task_ids(tasks: Path) -> set[str]:
    return {
        match.group(1)
        for line in tasks.read_text(encoding="utf-8").splitlines()
        if (match := COMPLETED_TASK_ID_PATTERN.match(line)) is not None
    }


def verify_host_implementation_markers(tasks: Path, mode: str) -> None:
    if mode not in {"source", "packaged-host"}:
        return
    declared = declared_task_ids(tasks)
    completed = completed_task_ids(tasks)
    missing = sorted(
        marker
        for marker in SOURCE_HOST_IMPLEMENTATION_MARKERS.values()
        if marker not in declared
    )
    incomplete = sorted(
        marker
        for marker in SOURCE_HOST_IMPLEMENTATION_MARKERS.values()
        if marker in declared and marker not in completed
    )
    if missing or incomplete:
        details = []
        if missing:
            details.append("missing: " + ", ".join(missing))
        if incomplete:
            details.append("incomplete: " + ", ".join(incomplete))
        raise GateError("required host implementation markers are " + "; ".join(details))


def run_task_checker(tasks: Path, mode: str) -> int:
    checker = Path(__file__).with_name("check-openspec-task-completion.py")
    command = [sys.executable, str(checker), str(tasks)]
    allowed = {
        "pr-bootstrap": BOOTSTRAP_ALLOWED,
        "source": SOURCE_ALLOWED,
        "packaged-host": PACKAGED_HOST_ALLOWED,
    }.get(mode)
    if allowed is not None:
        for task_id in sorted(allowed):
            command.extend(("--allow", task_id))
    result = subprocess.run(command, capture_output=True, text=True, check=False)
    if result.stdout:
        sys.stdout.write(result.stdout)
    if result.stderr:
        sys.stderr.write(result.stderr)
    return result.returncode


def run_acceptance_evidence_checker(repository_root: Path, *, host_scope: bool = False) -> int:
    checker = Path(__file__).with_name("check-document-fidelity-acceptance-evidence.py")
    command = [
        sys.executable,
        str(checker),
        "--root",
        str(repository_root),
        "--evidence",
        str(tasks_evidence_path(repository_root)),
    ]
    if host_scope:
        command.extend(("--scope", "katana-host"))
    result = subprocess.run(
        command,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.stdout:
        sys.stdout.write(result.stdout)
    if result.stderr:
        sys.stderr.write(result.stderr)
    return result.returncode


def tasks_evidence_path(repository_root: Path) -> Path:
    tasks = task_file(repository_root)
    return tasks.parent / "evidence" / "document-acceptance-v0.22.42.json"


def verify(version: str, mode: str, repository_root: Path) -> int:
    normalized_version = version.removeprefix("v")
    if normalized_version != TARGET_VERSION:
        return 0
    if mode not in MODES:
        raise GateError(f"unsupported document-fidelity release gate mode: {mode}")
    tasks = task_file(repository_root)
    missing = sorted(CRITICAL_REQUIRED - declared_task_ids(tasks))
    if missing:
        raise GateError(
            "critical release task IDs are missing from the bound change: "
            + ", ".join(missing)
        )
    verify_host_implementation_markers(tasks, mode)
    task_result = run_task_checker(tasks, mode)
    if task_result != 0 or mode in {"pr-bootstrap", "source"}:
        return task_result
    if mode == "packaged-host":
        return run_acceptance_evidence_checker(repository_root, host_scope=True)
    evidence_result = run_acceptance_evidence_checker(repository_root)
    return evidence_result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("version")
    parser.add_argument("mode")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[2])
    args = parser.parse_args()
    try:
        return verify(args.version, args.mode, args.root.resolve())
    except (GateError, OSError) as error:
        print(f"ERROR: document-fidelity release gate: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
