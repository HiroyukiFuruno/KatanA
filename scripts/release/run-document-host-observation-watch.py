#!/usr/bin/env python3
"""Coordinate the existing document-host recorder around native app actions."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import select
import sys
import time
import uuid

ROOT = Path(__file__).resolve().parents[2]
TMP = ROOT / "tmp" / "pr405-resolution"
RELEASE = ROOT / "scripts" / "release"
RECORDER_PATH = RELEASE / "record-document-host-observation.py"


class WatchError(RuntimeError):
    pass


def now() -> str:
    return datetime.now(timezone.utc).isoformat()


def load_recorder():
    sys.path.insert(0, str(RELEASE)) if str(RELEASE) not in sys.path else None
    spec = importlib.util.spec_from_file_location("_private_host_recorder", RECORDER_PATH)
    if spec is None or spec.loader is None:
        raise WatchError("existing recorder could not be imported")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def safe_new(path: Path, label: str) -> None:
    if not path.is_absolute() or not path.resolve().is_relative_to(TMP.resolve()):
        raise WatchError(f"{label} must be under tmp/pr405-resolution")
    parent = path.parent
    if not parent.is_dir() or path.is_symlink() or path.exists():
        raise WatchError(f"{label} parent is missing or output already exists")
    current = TMP
    if current.is_symlink():
        raise WatchError("private tmp root must not be a symlink")
    for part in parent.relative_to(TMP).parts:
        current = current / part
        if current.is_symlink():
            raise WatchError(f"{label} path contains a symlink")


def output_names(label: str) -> tuple[str, ...]:
    return (label + "-start.receipt.json", label + ".start.json", label + "-open.receipt.json",
            label + ".ready.json", label + "-close-start.receipt.json", label + ".close-start.json",
            label + "-closed.receipt.json", label + ".closed.json", label + ".result.json")


def ensure_run_outputs(run: Path, label: str, stage: str) -> None:
    if stage == "close":
        for name in (label + ".start.json", label + ".ready.json", label + "-open.receipt.json"):
            p = run / name
            if p.is_symlink() or not p.is_file():
                raise WatchError(f"required prior evidence missing or unsafe: {name}")
        names = output_names(label)[4:]
    else:
        names = output_names(label)
    if any((run / name).exists() or (run / name).is_symlink() for name in names):
        raise WatchError("recorder output already exists; refusing overwrite")


class Status:
    def __init__(self, path: Path, log: Path, stage: str, run: Path, label: str):
        safe_new(path, "status file")
        safe_new(log, "log file")
        if path.resolve() == log.resolve():
            raise WatchError("status and log paths must differ")
        self.path, self.log_path = path, log
        self.owner, self.last = str(uuid.uuid4()), None
        self.log = log.open("x", encoding="utf-8")
        self.base = {"schema": "private-native-observation-watch-v1", "stage": stage,
                     "run_directory": str(run), "label": label, "watch_id": self.owner}
        self.write("preparing", False)

    def write(self, state: str, ready: bool, **details) -> None:
        if state == self.last:
            return
        row = {**self.base, "state": state, "watcher_ready": ready,
               "updated_at": now(), **details}
        if self.path.exists():
            if self.path.is_symlink():
                raise WatchError("status file was replaced")
            try:
                old = json.loads(self.path.read_text(encoding="utf-8"))
            except (OSError, UnicodeError, json.JSONDecodeError) as error:
                raise WatchError("status file became unreadable") from error
            if old.get("watch_id") != self.owner:
                raise WatchError("status file ownership changed")
        temp = self.path.with_name("." + self.path.name + "." + self.owner + ".tmp")
        if temp.exists() or temp.is_symlink():
            raise WatchError("status temporary output already exists")
        with temp.open("x", encoding="utf-8") as stream:
            json.dump(row, stream, ensure_ascii=False, allow_nan=False, indent=2)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temp, self.path)
        self.log.write(json.dumps(row, ensure_ascii=False, allow_nan=False) + "\n")
        self.log.flush()
        os.fsync(self.log.fileno())
        self.last = state
        if ready:
            print("HOST_WATCH_READY " + json.dumps(
                row, ensure_ascii=False, allow_nan=False, separators=(",", ":")), flush=True)
        elif state == "finished":
            print("HOST_WATCH_DONE " + json.dumps(
                row, ensure_ascii=False, allow_nan=False, separators=(",", ":")), flush=True)

    def close(self):
        self.log.close()


class ReceiptWatch:
    def __init__(self, directory: Path):
        if platform.system() != "Darwin" or not hasattr(select, "kqueue"):
            raise WatchError("macOS vnode receipt watcher required")
        if directory.is_symlink() or not directory.is_dir():
            raise WatchError("receipt directory missing or unsafe")
        self.queue = select.kqueue()
        self.fd = os.open(directory, os.O_RDONLY)
        event = select.kevent(self.fd, filter=select.KQ_FILTER_VNODE,
                              flags=select.KQ_EV_ADD | select.KQ_EV_CLEAR,
                              fflags=select.KQ_NOTE_WRITE | getattr(select, "KQ_NOTE_RENAME", 0))
        self.queue.control([event], 0, 0)

    def wait(self, seconds: float):
        self.queue.control(None, 1, max(0.0, seconds))

    def close(self):
        self.queue.close()
        os.close(self.fd)


def identity(recorder, run: Path, expected_pid: int | None = None,
             expected_start: str | None = None) -> tuple[int, dict]:
    pid, actual = recorder.run_identity(run)
    if expected_pid is not None and (pid != expected_pid or actual.get("start_identity") != expected_start):
        raise WatchError("application PID/start identity changed")
    return pid, actual


def snapshot(recorder, run: Path, pid: int):
    from document_host_io import read
    path = run / "resource-receipts" / "latest.json"
    if path.is_symlink() or not path.is_file():
        raise WatchError("latest receipt missing or unsafe")
    first = read(path)
    if read(path) != first:
        return None
    raw, item = recorder.latest(run, pid)
    if raw != first or read(path) != first:
        return None
    return raw, item


def ready_candidate(recorder, producer, start: dict, item: dict) -> bool:
    from document_host_io import integer, require
    from urllib.parse import unquote, urlsplit
    if item["seq"] <= start["start_seq"]:
        return False
    kind = start["kind"]
    wanted, other = (("html_observations", "document_surfaces") if kind == "html"
                     else ("document_surfaces", "html_observations"))
    surfaces, other_surfaces = item.get(wanted), item.get(other)
    require(other_surfaces == [] and isinstance(surfaces, list) and len(surfaces) <= 1,
            "unexpected competing document surface")
    if not surfaces:
        return False
    surface = surfaces[0]
    require(isinstance(surface, dict), "surface observation is not an object")
    uri = surface.get("source_uri" if kind == "html" else "uri")
    require(isinstance(uri, str), "source URI missing")
    parsed = urlsplit(uri)
    require(parsed.scheme == "file" and parsed.netloc in {"", "localhost"}
            and Path(unquote(parsed.path)) == Path(start["input"]), "foreign source URI")
    generation = integer(surface.get("katana_renderer_session_epoch" if kind == "html" else "generation"),
                         "surface generation", minimum=1)
    if kind == "office":
        observed = item.get("document_observations")
        require(isinstance(observed, list), "Office observation list missing")
        if not observed:
            return False
        require(len(observed) == 1 and isinstance(observed[0], dict), "unexpected Office observations")
        observation = observed[0]
        require(observation.get("uri") == uri and observation.get("generation") == generation
                and observation.get("source_revision") == "sha256:" + start["input_sha256"],
                "foreign Office loaded source")
        require(type(observation.get("idle")) is bool, "Office idle state malformed")
        require("frame" in observation, "Office frame state missing")
        if observation.get("frame") is None or observation["idle"] is False:
            return False
    else:
        integer(surface.get("katana_adapter_instance_id"), "HTML adapter", minimum=1)
        frame_epoch = integer(surface.get("katana_frame_renderer_session_epoch"), "HTML frame epoch", minimum=1)
        require(frame_epoch <= generation, "HTML frame epoch exceeds current session")
        require("loaded_source" in surface, "HTML loaded source state missing")
        state = surface.get("loaded_source")
        if state is not None:
            fnv = producer.input_fnv(Path(start["input"]))
            require(isinstance(state, dict) and state.get("preview_input_fnv1a64")
                    == state.get("last_imported_disk_fnv1a64") == fnv, "foreign HTML loaded source")
        if state is None or frame_epoch < generation:
            return False
    if integer(item.get("host", {}).get("frames"), "host frames") == 0:
        return False
    # Existing source/frame validation remains the sole authority for readiness.
    producer.opened_identity(item, kind, Path(start["input"]), start["input_sha256"])
    return True


def open_stage(args, recorder, status: Status, label: str):
    from document_host_io import moment
    run = args.run_directory
    ensure_run_outputs(run, label, "open")
    recorder.observe_start(run, label, args.input, args.kind)
    start = json.loads((run / (label + ".start.json")).read_text(encoding="utf-8"))
    pid, proc = identity(recorder, run, start["pid"], start["start_identity"])
    producer = recorder.module("_private_ready_observation", "produce-document-host-evidence.py")
    budget = 60 if args.kind == "html" else 15
    deadline = moment(start["observedAt"], "start").timestamp() + budget
    remaining = max(0.0, deadline - time.time())
    monotonic_deadline = time.monotonic() + remaining
    watch = ReceiptWatch(run / "resource-receipts")
    try:
        status.write("watching_open", True, pid=pid, start_identity=proc["start_identity"],
                     label=label, started_at=start["observedAt"], budget_seconds=budget,
                     deadline_at=datetime.fromtimestamp(deadline, timezone.utc).isoformat())
        while True:
            if time.monotonic() >= monotonic_deadline:
                raise WatchError(f"actual frame not ready within original {budget}s budget")
            identity(recorder, run, pid, proc["start_identity"])
            current = snapshot(recorder, run, pid)
            if current is not None and ready_candidate(recorder, producer, start, current[1]):
                recorder.observe_ready(run, label)
                ready = json.loads((run / (label + ".ready.json")).read_text(encoding="utf-8"))
                raw = (run / (label + "-open.receipt.json")).read_bytes()
                from document_host_observation import receipt
                saved = receipt(raw, run.name, pid)
                producer.opened_identity(saved, args.kind, args.input, start["input_sha256"])
                if raw != current[0] or saved["seq"] != ready["seq"]:
                    raise WatchError("ready boundary differs from saved receipt")
                ensure_ready_budget(recorder, start, ready, budget)
                status.write("ready_observed", False, pid=pid, ready_seq=ready["seq"],
                             ready_observed_at=ready["observedAt"],
                             receipt_sha256=hashlib.sha256(raw).hexdigest(),
                             boundary_source="existing recorder observe_ready")
                return
            remaining = monotonic_deadline - time.monotonic()
            if remaining <= 0:
                raise WatchError(f"actual frame not ready within original {budget}s budget")
            watch.wait(min(0.1, remaining))
    finally:
        watch.close()


def ensure_ready_budget(recorder, start: dict, ready: dict, budget: int):
    elapsed = recorder.duration_ms(start["observedAt"], ready["observedAt"])
    if elapsed > budget * 1000:
        raise WatchError(f"recorded first frame exceeded original {budget}s budget")


def sample_journal_complete(path: Path) -> bool:
    if path.is_symlink():
        raise WatchError("external sample journal became a symlink")
    if not path.exists():
        return False
    if not path.is_file():
        raise WatchError("external sample journal is not a regular file")
    raw = path.read_bytes()
    for line in raw.split(b"\n")[:-1]:
        if not isinstance(json.loads(line), dict):
            raise WatchError("completed external sample is not an object")
    return not raw or raw.endswith(b"\n")


def stable_close_sample(run: Path, seq: int, receipt_sha: str, pid: int, start_id: str,
                        after: str, expected: dict) -> bool:
    from document_host_observation import stable_sample
    from document_host_io import moment, require
    from document_host_transcript import resources
    journal = run / "memory-receipt-observations.jsonl"
    if not sample_journal_complete(journal):
        return False
    try:
        rows = [json.loads(line) for line in journal.read_bytes().splitlines()]
        if not all(isinstance(row, dict) for row in rows):
            raise WatchError("completed external sample is not an object")
        for row in rows:
            memory = row.get("memory", {})
            if (row.get("receipt_seq") != seq or row.get("receipt_sha256") != receipt_sha
                    or row.get("receipt_stable_during_sample") is not True
                    or memory.get("status") != "ok" or memory.get("pid") != pid
                    or memory.get("identity", {}).get("start_identity") != start_id
                    or moment(memory.get("observed_at"), "sample") < moment(after, "boundary")):
                continue
            values = {"host": row["host"], "kdv": row["kdv"], "workers": row["document_workers_live"],
                      "children": row["kernel_processes"]["children"]}
            require(all(row[key] == expected[key] for key in ("host", "kdv", "document_workers_live")),
                    "observer differs from actual receipt")
            resources(values, idle=False)
            if any(values["host"].values()) or any(values["kdv"].values()) or values["workers"] or values["children"]:
                continue
            return stable_sample(journal, seq, receipt_sha, pid, start_id, memory["observed_at"], expected)
        return False
    except json.JSONDecodeError:
        if not sample_journal_complete(journal):
            return False
        raise
    except (RuntimeError, OSError, KeyError, TypeError):
        # WHY: tolerate only a last line that is still being appended; complete malformed rows remain errors.
        sample_journal_complete(journal)
        raise


def close_stage(args, recorder, status: Status, label: str):
    from document_host_io import digest, moment
    from document_host_observation import receipt
    run = args.run_directory
    ensure_run_outputs(run, label, "close")
    start = json.loads((run / (label + ".start.json")).read_text(encoding="utf-8"))
    ready = json.loads((run / (label + ".ready.json")).read_text(encoding="utf-8"))
    pid, proc = identity(recorder, run, start["pid"], start["start_identity"])
    opened = receipt((run / (label + "-open.receipt.json")).read_bytes(), run.name, pid)
    recorder.observe_close_start(run, label)
    close = json.loads((run / (label + ".close-start.json")).read_text(encoding="utf-8"))
    if close["seq"] < ready["seq"]:
        raise WatchError("close-start sequence moved backwards")
    started = moment(close["observedAt"], "close-start")
    deadline = started.timestamp() + 5
    monotonic_deadline = time.monotonic() + max(0.0, deadline - time.time())
    launch_sha = digest((run / "launch.json").read_bytes())
    watch = ReceiptWatch(run / "resource-receipts")
    closed = None
    try:
        status.write("watching_closed", True, pid=pid, start_identity=proc["start_identity"],
                     label=label, close_started_at=close["observedAt"], budget_seconds=5,
                     deadline_at=datetime.fromtimestamp(deadline, timezone.utc).isoformat(),
                     next_action="parent may close the active document")
        while closed is None:
            identity(recorder, run, pid, proc["start_identity"])
            current = snapshot(recorder, run, pid)
            if current is not None:
                raw, item = current
                if item["seq"] > close["seq"]:
                    from document_host_boundaries import require_closed, require_same_surface
                    if item.get("document_surfaces") or item.get("html_observations"):
                        require_same_surface(opened, item, args.kind)
                    else:
                        require_closed(item)
                    if item.get("document_surfaces") or item.get("html_observations"):
                        remaining = monotonic_deadline - time.monotonic()
                        if remaining <= 0:
                            raise WatchError("document surface remained open beyond original 5s budget")
                        watch.wait(min(0.1, remaining))
                        continue
                    sha = digest(raw)
                    if stable_close_sample(run, item["seq"], sha, pid, proc["start_identity"],
                                           close["observedAt"], item):
                        if snapshot(recorder, run, pid) != current:
                            raise WatchError("receipt changed after idle sample; rejecting race")
                        if time.monotonic() >= monotonic_deadline:
                            raise WatchError("idle close evidence exceeded original 5s budget")
                        recorder.observe_closed(run, label)
                        saved_raw = (run / (label + "-closed.receipt.json")).read_bytes()
                        closed = json.loads((run / (label + ".closed.json")).read_text(encoding="utf-8"))
                        if saved_raw != raw or closed["seq"] != item["seq"]:
                            raise WatchError("recorder saved a different receipt; rejecting race")
                        elapsed = recorder.duration_ms(close["observedAt"], closed["observedAt"])
                        if elapsed > 5000:
                            raise WatchError("recorded close observation exceeded original 5s budget")
                        status.write("waiting_post_closed_sample", False, pid=pid, label=label,
                                     close_seq=closed["seq"], close_receipt_sha256=digest(saved_raw),
                                     close_observed_at=closed["observedAt"], close_observed_ms=elapsed,
                                     sample_timeout_seconds=args.sample_timeout_seconds,
                                     sample_deadline_at=datetime.fromtimestamp(
                                         moment(closed["observedAt"], "closed").timestamp()
                                         + args.sample_timeout_seconds, timezone.utc).isoformat())
                        closed = (saved_raw, item, closed)
                        break
            remaining = monotonic_deadline - time.monotonic()
            if remaining <= 0:
                raise WatchError("closed surface and idle resource boundary not observed within original 5s budget")
            watch.wait(min(0.1, remaining))
    finally:
        watch.close()
    saved_raw, item, closed_meta = closed
    close_sha = digest(saved_raw)
    sample_deadline_at = moment(closed_meta["observedAt"], "closed boundary").timestamp() + args.sample_timeout_seconds
    sample_deadline = time.monotonic() + max(0.0, sample_deadline_at - time.time())
    journal = run / "memory-receipt-observations.jsonl"
    partial = None
    while True:
        if time.monotonic() >= sample_deadline:
            raise WatchError("separate post-close sample deadline expired; original close time unchanged")
        identity(recorder, run, pid, proc["start_identity"])
        complete = sample_journal_complete(journal)
        if complete != partial:
            partial = complete
            status.write("waiting_post_closed_sample" if complete else "waiting_partial_post_closed_sample",
                         False, pid=pid, label=label, close_seq=closed_meta["seq"],
                         close_receipt_sha256=close_sha, close_observed_at=closed_meta["observedAt"],
                         sample_timeout_seconds=args.sample_timeout_seconds)
        if complete and stable_close_sample(run, closed_meta["seq"], close_sha, pid,
                                             proc["start_identity"], closed_meta["observedAt"], item):
            recorder.finish_case(run, label)
            result = json.loads((run / (label + ".result.json")).read_text(encoding="utf-8"))
            status.write("finished", False, pid=pid, label=label, result_path=str(run / (label + ".result.json")),
                         close_seq=result["closeSeq"], close_receipt_sha256=close_sha,
                         close_observed_at=result["closeObservedAt"],
                         result_source="existing recorder finish_case")
            return
        remaining = sample_deadline - time.monotonic()
        if remaining <= 0:
            raise WatchError("separate post-close sample deadline expired; original close time unchanged")
        time.sleep(min(0.1, remaining))


def parser():
    root = argparse.ArgumentParser(description=__doc__)
    stages = root.add_subparsers(dest="stage", required=True)
    for name in ("open", "close"):
        p = stages.add_parser(name)
        p.add_argument("--run-directory", type=Path, required=True)
        p.add_argument("--prefix", required=True)
        p.add_argument("--cycle", type=int, required=True)
        p.add_argument("--kind", choices=("office", "html"), required=True)
        p.add_argument("--status-file", type=Path, required=True)
        p.add_argument("--log-file", type=Path, required=True)
        if name == "open":
            p.add_argument("--input", type=Path, required=True)
        else:
            p.add_argument("--sample-timeout-seconds", type=float, required=True,
                           help="separate operational deadline after the recorded closed time")
    return root


def main() -> int:
    args = parser().parse_args()
    status = None
    try:
        if RECORDER_PATH.is_symlink() or not RECORDER_PATH.is_file():
            raise WatchError("existing recorder missing or unsafe")
        recorder = load_recorder()
        recorder.validate_run(args.run_directory)
        label = recorder.validate_label(args.prefix, args.cycle, args.kind)
        ensure_run_outputs(args.run_directory, label, args.stage)
        if args.stage == "open":
            if not args.input.is_absolute() or args.input.is_symlink() or not args.input.is_file():
                raise WatchError("input must be an actual absolute original file")
        elif not 0 < args.sample_timeout_seconds <= 7200:
            raise WatchError("separate sample timeout must be in (0, 7200] seconds")
        status = Status(args.status_file, args.log_file, args.stage, args.run_directory, label)
        if args.stage == "open":
            open_stage(args, recorder, status, label)
        else:
            close_stage(args, recorder, status, label)
        return 0
    except KeyboardInterrupt:
        error = "watcher interrupted"
        if status is not None:
            status.write("failed", False, error=error)
        print("ERROR: " + error, file=sys.stderr)
        return 130
    except (ValueError, RuntimeError, OSError, KeyError, TypeError, IndexError) as error:
        if status is not None:
            try:
                status.write("failed", False, error=str(error))
            except Exception:
                pass
        print("ERROR: private native watcher: " + str(error), file=sys.stderr)
        return 1
    finally:
        if status is not None:
            status.close()


if __name__ == "__main__":
    raise SystemExit(main())
