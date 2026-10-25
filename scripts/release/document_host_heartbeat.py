"""既存100ms heartbeatを独立readし、既存200ms advance/3x50ms read契約を保存する。"""

from __future__ import annotations

from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import threading
import time

from document_host_io import fields, integer, moment, require

PROGRESS_INTERVAL_NS = 200_000_000
READ_RETRIES = 3
READ_RETRY_SECONDS = 0.05


def observe(path: Path, stop: threading.Event) -> dict:
    attempts = []
    for attempt in range(READ_RETRIES):
        raw, modified, status, frame = None, None, "missing", None
        try:
            before = path.stat()
            raw = path.read_bytes()
            after = path.stat()
            parsed = re.fullmatch(rb"first-frame\nframe=([1-9][0-9]*)\n", raw)
            if parsed is not None and (before.st_mtime_ns, before.st_size) == (after.st_mtime_ns, after.st_size):
                frame, modified, status = int(parsed[1]), after.st_mtime_ns, "observed"
            else:
                status = "partial_write"
        except FileNotFoundError:
            pass
        attempts.append({"status": status, "observed_at": datetime.now(timezone.utc).isoformat(),
                         "monotonic_ns": time.monotonic_ns(), "frame": frame, "mtime_ns": modified,
                         "sha256": hashlib.sha256(raw).hexdigest() if raw is not None else None})
        if status == "observed" or attempt+1 == READ_RETRIES:
            break
        stop.wait(READ_RETRY_SECONDS)
    return dict(attempts[-1], attempts=attempts)


class HeartbeatObserver:
    def __init__(self, path: Path, output: Path):
        self.path, self.output = path, output
        self.stop_event = threading.Event()
        self.error = None
        self.thread = threading.Thread(target=self.run, name="existing-ui-heartbeat-observer", daemon=True)

    def run(self) -> None:
        try:
            with self.output.open("x") as handle:
                while not self.stop_event.is_set():
                    handle.write(json.dumps(observe(self.path, self.stop_event), allow_nan=False)+"\n")
                    handle.flush()
                    self.stop_event.wait(0.1)
        except (OSError, ValueError) as error:
            self.error = error

    def start(self) -> None:
        self.thread.start()

    def finish(self) -> None:
        self.stop_event.set()
        self.thread.join()
        require(self.error is None, "actual heartbeat observer failed")


def verify(observations: object, cases: list[dict]) -> None:
    require(isinstance(observations, list) and observations, "continuous heartbeat bounded proof missing")
    previous_time, previous_frame = -1, 0
    for item in observations:
        fields(item, {"status", "observed_at", "monotonic_ns", "frame", "mtime_ns", "sha256", "attempts"}, "heartbeat read")
        moment(item["observed_at"], "actual heartbeat read time")
        instant = integer(item["monotonic_ns"], "actual heartbeat monotonic time", minimum=1)
        require(instant > previous_time, "heartbeat observation time reversed")
        previous_time = instant
        require(isinstance(item["attempts"], list) and 1 <= len(item["attempts"]) <= READ_RETRIES, "heartbeat read retry contract differs")
        if item["status"] == "observed":
            frame = integer(item["frame"], "actual heartbeat frame", minimum=1)
            integer(item["mtime_ns"], "actual heartbeat mtime", minimum=1)
            expected = hashlib.sha256(f"first-frame\nframe={frame}\n".encode()).hexdigest()
            require(item["sha256"] == expected and frame >= previous_frame, "heartbeat content/monotonic frame differs")
            previous_frame = frame
        else:
            require(item["status"] in {"missing", "partial_write"} and item["frame"] is None and item["mtime_ns"] is None,
                    "missing/partial heartbeat cannot substitute a completed frame")
    for case in cases:
        start, end = moment(case["open_started_at"], "case start"), moment(case["closed_at"], "case closed")
        before = [index for index, item in enumerate(observations) if moment(item["observed_at"], "heartbeat") <= start]
        after = [index for index, item in enumerate(observations) if moment(item["observed_at"], "heartbeat") >= end]
        require(before and after, "bounded heartbeat proof unavailable at operation boundaries")
        interval = observations[before[-1]:after[0]+1]
        require(all(item["status"] == "observed" for item in interval), "bounded heartbeat proof missing after original read retries")
        anchor = interval[0]
        previous = anchor
        for current in interval[1:]:
            gap = current["monotonic_ns"]-previous["monotonic_ns"]
            require(gap <= PROGRESS_INTERVAL_NS, "bounded heartbeat proof missing: actual observer gap exceeds existing 200ms check interval")
            if current["monotonic_ns"]-anchor["monotonic_ns"] >= PROGRESS_INTERVAL_NS:
                require(current["frame"] > anchor["frame"], "UI heartbeat stalled across existing 200ms advance check")
                anchor = current
            previous = current
