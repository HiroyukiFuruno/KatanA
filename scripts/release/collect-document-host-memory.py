#!/usr/bin/env python3
"""実receiptを全件外部観測する。readonly recorder以外のinspectorや操作入力を追加しない。"""

from __future__ import annotations

import argparse
from contextlib import closing
from datetime import datetime, timezone
import hashlib
import os
from pathlib import Path
import platform
import select
import sys
import time

from document_host_io import decode, require
from process_memory_sampler import process_identity, sample
from document_host_processes import ProcessObserver
from document_host_heartbeat import HeartbeatObserver


def collect(run: Path, executable: Path, worker: Path, seconds: int) -> dict:
    require(platform.system() == "Darwin", "actual macOS process observation required")
    require(run.is_absolute() and run.is_dir() and not run.is_symlink(), "actual non-symlink run directory required")
    require(executable.is_absolute() and executable.is_file() and not executable.is_symlink(), "actual packaged main required")
    pid = int((run / "app.pid").read_text().strip())
    expected = process_identity(pid)
    require(Path(expected["executable"]).resolve() == executable.resolve(), "process executable differs from selected packaged main")
    observer = ProcessObserver(pid, executable, worker)
    launch_raw = (run / "launch.json").read_bytes()
    launch = decode(launch_raw)
    require(isinstance(launch, dict) and launch.get("target") in {"macos-arm64", "macos-x86_64"}, "actual explicit launch record required")
    requested = "-arm64" if launch["target"] == "macos-arm64" else "-x86_64"
    require(launch.get("argv") == ["/usr/bin/arch", requested, str(executable)], "actual launch record differs from observed process")
    launch_sha = hashlib.sha256(launch_raw).hexdigest()
    heartbeat = HeartbeatObserver(run / "heartbeat.txt", run / "heartbeat-observations.jsonl")
    heartbeat.start()
    receipts = run / "resource-receipts"
    require(receipts.is_dir() and not receipts.is_symlink(), "actual readonly receipt directory required")
    latest = receipts / "latest.json"
    output = run / "memory-receipt-observations.jsonl"
    seen = set()
    observations = 0
    deadline = time.monotonic() + seconds
    with closing(select.kqueue()) as queue:
        descriptor = os.open(receipts, os.O_RDONLY)
        try:
            queue.control([select.kevent(descriptor, filter=select.KQ_FILTER_VNODE,
                           flags=select.KQ_EV_ADD | select.KQ_EV_CLEAR, fflags=select.KQ_NOTE_WRITE),
                           select.kevent(1, filter=select.KQ_FILTER_TIMER, flags=select.KQ_EV_ADD, data=100)], 0, 0)
            with output.open("x") as destination:
                while time.monotonic() < deadline:
                    if (run / "app.exit").exists():
                        require((run / "app.exit").read_text().strip() == "0" and bool(seen), "abnormal app exit or missing samples")
                        import json
                        terminal = observer.finish()
                        require((run / "launch.json").read_bytes() == launch_raw, "actual launch record changed during observation")
                        heartbeat.finish()
                        with (run / "kernel-processes.after-exit.json").open("x") as handle:
                            json.dump(terminal, handle, allow_nan=False)
                        with (run / "app-exit-observed.json").open("x") as handle:
                            json.dump({"pid": pid, "exit_code": 0, "observed_at": terminal["observed_at"]}, handle)
                        return {"status": "collected", "observations": observations, "receipt_sequences": len(seen), "app_exit": 0}
                    try:
                        raw = latest.read_bytes()
                    except FileNotFoundError:
                        queue.control(None, 8, 1)
                        continue
                    receipt = decode(raw)
                    require(isinstance(receipt, dict) and receipt.get("pid") == pid and receipt.get("run_id") == run.name,
                            "receipt process/run differs")
                    sequence = receipt.get("seq")
                    require(type(sequence) is int and sequence > 0, "invalid receipt sequence")
                    measurement = sample(pid, expected)
                    require(measurement.get("status") == "ok", "external sampler failed; no zero/fallback measurement")
                    processes = observer.snapshot()
                    finished = datetime.now(timezone.utc).isoformat()
                    confirmed = latest.read_bytes() == raw
                    import json
                    destination.write(json.dumps({"receipt_seq": sequence, "receipt_sha256": hashlib.sha256(raw).hexdigest(),
                                      "receipt_stable_during_sample": confirmed, "sample_finished_at": finished,
                                      "host": receipt["host"], "kdv": receipt["kdv"],
                                      "document_workers_live": receipt["document_workers_live"], "memory": measurement,
                                      "kernel_processes": processes, "launch_record_sha256": launch_sha}, allow_nan=False)+"\n")
                    destination.flush()
                    seen.add(sequence)
                    observations += 1
                    queue.control(None, 8, 1)
                require(False, "observation deadline expired")
        finally:
            heartbeat.finish()
            os.close(descriptor)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-directory", type=Path, required=True)
    parser.add_argument("--executable", type=Path, required=True)
    parser.add_argument("--sidecar-executable", type=Path, required=True)
    parser.add_argument("--timeout-seconds", type=int, default=1200)
    args = parser.parse_args()
    try:
        require(1 <= args.timeout_seconds <= 7200, "invalid observation timeout")
        result = collect(args.run_directory, args.executable, args.sidecar_executable, args.timeout_seconds)
    except (ValueError, RuntimeError, OSError, KeyError) as error:
        print(f"ERROR: actual host memory collection: {error}", file=sys.stderr)
        return 1
    import json
    print(json.dumps(result))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
