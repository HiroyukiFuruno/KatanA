#!/usr/bin/env python3
"""Bind versioned OpenSpec task completion to the current delivery phase."""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


PHASE_ALLOWLISTS = {
    ("0.22.43", "v0-22-43-document-ui-regressions"): {
        "strict": set(),
        # WHY: 実行中のpush結果はhook終了前には存在しないため、完了証跡だけを後段へ残す。
        "pr-bootstrap": {"8.2", "8.3", "8.4", "8.5", "8.6"},
        # WHY: 公開成果物が存在する前に、CI・merge・公開・後処理の集約結果を完了扱いしない。
        "release-artifact-pending": {"8.4", "8.5", "8.6"},
        "post-release-evidence": {"8.5", "8.6"},
    },
    ("0.22.44", "v0-22-44-document-loading-input-regressions"): {
        "strict": set(),
        # WHY: push終了証跡、Draft review、required CI、公開後証跡は各後続段階で回収する。
        "pr-bootstrap": {"1.3", "5.3", "5.4", "5.5", "5.6"},
        "release-artifact-pending": {"1.3", "5.5", "5.6"},
        "post-release-evidence": {"5.6"},
    },
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("version")
    parser.add_argument("change_name")
    parser.add_argument("mode", choices=sorted(next(iter(PHASE_ALLOWLISTS.values()))))
    parser.add_argument("tasks", type=Path)
    args = parser.parse_args()

    tasks_path = args.tasks
    repository_root = Path(__file__).resolve().parents[2]
    binding = (args.version, args.change_name)
    phase_allowlists = PHASE_ALLOWLISTS.get(binding)
    if (
        phase_allowlists is None
        or args.mode not in phase_allowlists
    ):
        parser.error("phase exceptions are restricted to exact supported version and OpenSpec change pairs")
    expected_tasks = repository_root / "openspec/changes" / args.change_name / "tasks.md"
    if tasks_path.resolve() != expected_tasks.resolve():
        parser.error("phase exceptions are restricted to the canonical tasks.md path")

    checker = Path(__file__).with_name("check-openspec-task-completion.py")
    command = [sys.executable, str(checker), str(tasks_path)]
    for task_id in sorted(phase_allowlists[args.mode]):
        command.extend(("--allow", task_id))
    return subprocess.run(command, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
