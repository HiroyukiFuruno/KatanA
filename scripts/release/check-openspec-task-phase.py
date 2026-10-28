#!/usr/bin/env python3
"""Bind v0.22.43 task completion to the current delivery phase."""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


EXPECTED_VERSION = "0.22.43"
EXPECTED_CHANGE = "v0-22-43-document-ui-regressions"
PHASE_ALLOWLISTS = {
    "strict": set(),
    # WHY: 実行中のpush結果はhook終了前には存在しないため、完了証跡だけを後段へ残す。
    "pr-bootstrap": {"8.2", "8.3", "8.4", "8.5", "8.6"},
    # WHY: 公開成果物が存在する前に、CI・merge・公開・後処理の集約結果を完了扱いしない。
    "release-artifact-pending": {"8.4", "8.5", "8.6"},
    "post-release-evidence": {"8.5", "8.6"},
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("version")
    parser.add_argument("change_name")
    parser.add_argument("mode", choices=sorted(PHASE_ALLOWLISTS))
    parser.add_argument("tasks", type=Path)
    args = parser.parse_args()

    tasks_path = args.tasks
    repository_root = Path(__file__).resolve().parents[2]
    expected_tasks = repository_root / "openspec/changes" / EXPECTED_CHANGE / "tasks.md"
    if (
        args.version != EXPECTED_VERSION
        or args.change_name != EXPECTED_CHANGE
        or tasks_path.resolve() != expected_tasks.resolve()
    ):
        parser.error("phase exceptions are restricted to the exact v0.22.43 OpenSpec change")

    checker = Path(__file__).with_name("check-openspec-task-completion.py")
    command = [sys.executable, str(checker), str(tasks_path)]
    for task_id in sorted(PHASE_ALLOWLISTS[args.mode]):
        command.extend(("--allow", task_id))
    return subprocess.run(command, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
