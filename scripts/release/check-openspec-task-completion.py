#!/usr/bin/env python3

from __future__ import annotations

import argparse
import re
import tempfile
from pathlib import Path


TASK_PATTERN = re.compile(r"^\s*-\s*\[([^xX])\]\s+(\d+\.\d+)\b")


def incomplete_task_ids(path: Path) -> set[str]:
    return {
        match.group(2)
        for line in path.read_text(encoding="utf-8").splitlines()
        if (match := TASK_PATTERN.match(line)) is not None
    }


def verify(path: Path, allowed: set[str]) -> None:
    incomplete = incomplete_task_ids(path)
    unexpected = incomplete - allowed
    if unexpected:
        names = ", ".join(sorted(unexpected))
        raise SystemExit(f"unexpected incomplete OpenSpec tasks: {names}")
    if not allowed and incomplete:
        names = ", ".join(sorted(incomplete))
        raise SystemExit(f"incomplete OpenSpec tasks: {names}")
    if incomplete:
        names = ", ".join(sorted(incomplete))
        print(f"OK: only post-PR evidence tasks remain incomplete: {names}")
    else:
        print("OK: all OpenSpec tasks are complete")


def self_test() -> None:
    with tempfile.TemporaryDirectory() as directory:
        tasks = Path(directory) / "tasks.md"
        tasks.write_text(
            "- [x] 1.1 implemented\n- [ ] 2.1 CI evidence\n- [/] 2.2 review\n",
            encoding="utf-8",
        )
        assert incomplete_task_ids(tasks) == {"2.1", "2.2"}
        verify(tasks, {"2.1", "2.2"})
        try:
            verify(tasks, {"2.1"})
        except SystemExit as error:
            assert "2.2" in str(error)
        else:
            raise AssertionError("unexpected incomplete task was accepted")
    print("OK: OpenSpec task completion contract self-test passed")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("tasks", nargs="?", type=Path)
    parser.add_argument("--allow", action="append", default=[])
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return
    if args.tasks is None:
        parser.error("tasks path is required")
    verify(args.tasks, set(args.allow))


if __name__ == "__main__":
    main()
