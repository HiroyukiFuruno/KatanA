#!/usr/bin/env python3
"""Run one CI command and record bounded, non-authoritative timing diagnostics."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
from html import escape
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time


def cache_value(raw: str) -> bool | None:
    normalized = raw.strip().lower()
    if normalized == "true":
        return True
    if normalized == "false":
        return False
    return None


def lock_digest(root: Path) -> dict[str, str | None]:
    digests: dict[str, str | None] = {}
    for relative in (Path("Cargo.lock"), Path("scripts/screenshot/Cargo.lock")):
        path = root / relative
        key = relative.as_posix()
        if not path.exists():
            digests[key] = None
            continue
        if not path.is_file():
            raise OSError(f"lockfile is not a regular file: {path}")
        digests[key] = hashlib.sha256(path.read_bytes()).hexdigest()
    return digests


def source_metadata() -> dict[str, str | None]:
    names = ("GITHUB_SHA", "RUNNER_OS", "RUNNER_ARCH", "GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT")
    return {name: os.environ.get(name) for name in names}


def report(raw_cache: str, command: list[str], elapsed: float, raw_status: int,
           normalized_status: int, spawned: bool, root: Path) -> dict:
    return {
        "schema": 1,
        "diagnostic_only": True,
        "reuse_authorized": False,
        "cache_scope": "cargo_registry_sources_only",
        "build_cache_state": "not_measured",
        "cold_warm_equivalence": "not_evaluated",
        "cache_hit_raw": raw_cache,
        "cache_hit": cache_value(raw_cache),
        "command_argv": command,
        "executed_once": spawned,
        "observed_at": datetime.now(timezone.utc).isoformat(),
        "elapsed_seconds": elapsed,
        "exit": raw_status,
        "exit_code": normalized_status,
        "cwd": str(root),
        "source": source_metadata(),
        "cargo_lock_sha256": lock_digest(root),
        "lock_observation_phase": "after_command",
    }


def save_json(output: Path, payload: dict) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary = tempfile.mkstemp(prefix=f".{output.name}.", dir=output.parent)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as stream:
            json.dump(payload, stream, sort_keys=True, indent=2)
            stream.write("\n")
        os.replace(temporary, output)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def summary_identity(payload: dict) -> str:
    identity = {key: payload[key] for key in (
        "source", "cargo_lock_sha256", "lock_observation_phase",
    )}
    return f"<pre>{escape(json.dumps(identity, sort_keys=True), quote=False)}</pre>"


def append_summary(path: Path, payload: dict) -> None:
    lines = [
        "### Windows test command timing (diagnostic only)",
        f"- Exit: `{payload['exit_code']}`",
        f"- Elapsed seconds: `{payload['elapsed_seconds']:.6f}`",
        f"- Cargo registry cache hit: `{payload['cache_hit']}`",
        "- Build cache state: `not_measured`",
        "- Cold/warm equivalence: `not_evaluated`",
        "- Test reuse authorized: `false`",
        "",
        summary_identity(payload),
        "",
    ]
    with path.open("a", encoding="utf-8") as stream:
        stream.write("\n".join(lines))


def execute(command: list[str]) -> tuple[int, int, float, bool]:
    started = time.perf_counter()
    spawned = True
    try:
        completed = subprocess.run(command, check=False)
        status = completed.returncode
    except OSError as error:
        print(f"could not execute command: {error}", file=sys.stderr)
        status = 127
        spawned = False
    normalized = 128 - status if status < 0 else status
    return status, normalized, time.perf_counter() - started, spawned


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cache-hit", required=True)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    arguments = parser.parse_args(argv)
    command = arguments.command[1:] if arguments.command[:1] == ["--"] else arguments.command
    if not command:
        parser.error("a command is required after --")
    status, normalized, elapsed, spawned = execute(command)
    recording_failed = False
    try:
        root = Path.cwd().resolve()
        payload = report(arguments.cache_hit, command, elapsed, status, normalized, spawned, root)
        save_json(arguments.output, payload)
        summary = os.environ.get("GITHUB_STEP_SUMMARY")
        if summary:
            append_summary(Path(summary), payload)
    except (OSError, TypeError, ValueError) as error:
        recording_failed = True
        print(f"could not record diagnostic: {error}", file=sys.stderr)
    if status != 0:
        return normalized
    return 2 if recording_failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
