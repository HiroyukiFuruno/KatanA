#!/usr/bin/env python3
"""Record timestamps and completed-frame receipts around manually performed UI actions."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
from pathlib import Path
import re
import sys

from document_host_io import decode, digest, moment, read, require, write_new
from document_host_boundaries import require_closed, require_same_surface
from document_host_observation import receipt, stable_sample
from generated_asset_identity import module
from process_memory_sampler import process_identity


def current_time() -> str:
    return datetime.now(timezone.utc).isoformat()


def validate_run(run: Path) -> None:
    root = Path(__file__).resolve().parents[2]
    require(run.is_absolute() and run.parent == root / "tmp" and run.name.startswith("macos-document-host.")
            and run.is_dir() and not run.is_symlink(), "actual isolated host run required")


def run_identity(run: Path) -> tuple[int, dict]:
    validate_run(run)
    pid = int(read(run / "app.pid").strip())
    launch = decode(read(run / "launch.json"))
    identity = process_identity(pid)
    require(identity.get("executable") == launch["argv"][2], "actual process executable differs")
    return pid, identity


def latest(run: Path, pid: int) -> tuple[bytes, dict]:
    raw = read(run / "resource-receipts/latest.json")
    return raw, receipt(raw, run.name, pid)


def validate_label(prefix: str, cycle: int, kind: str) -> str:
    require(re.fullmatch(r"[A-Za-z0-9_-]+", prefix) is not None, "invalid label")
    require(1 <= cycle <= 10 and kind in {"html", "office"}, "invalid case")
    return f"{prefix}-cycle{cycle:02}-{kind}"


def observe_start(run: Path, label: str, input_path: Path, kind: str) -> None:
    pid, identity = run_identity(run)
    producer = module("_host_observation_input", "produce-document-host-evidence.py")
    input_sha = producer.input_digest(input_path)
    fixtures = producer.legacy()
    require(input_sha in fixtures.SUPPLIED_OFFICE_FIXTURES if kind == "office"
            else input_sha == fixtures.ORIGINAL_HTML_SHA256, "input is not a supplied original")
    observed_at = current_time()
    raw, item = latest(run, pid)
    require_closed(item)
    write_raw(run / (label + "-start.receipt.json"), raw)
    write_new(run / (label + ".start.json"), {"pid": pid, "start_identity": identity["start_identity"],
              "input": str(input_path), "input_sha256": input_sha, "kind": kind,
              "start_seq": item["seq"], "receipt_sha256": digest(raw), "observedAt": observed_at})


def observe_ready(run: Path, label: str) -> None:
    start = decode(read(run / (label + ".start.json")))
    pid, identity = run_identity(run)
    require(pid == start["pid"] and identity["start_identity"] == start["start_identity"], "process changed")
    raw, item = latest(run, pid)
    require(item["seq"] > start["start_seq"], "ready frame did not advance")
    producer = module("_host_observation_ready", "produce-document-host-evidence.py")
    producer.opened_identity(item, start["kind"], Path(start["input"]), start["input_sha256"])
    write_raw(run / (label + "-open.receipt.json"), raw)
    write_new(run / (label + ".ready.json"), {"seq": item["seq"], "observedAt": current_time()})


def write_raw(path: Path, raw: bytes) -> None:
    with path.open("xb") as output:
        output.write(raw)


def observe_close_start(run: Path, label: str) -> None:
    start = decode(read(run / (label + ".start.json")))
    ready = decode(read(run / (label + ".ready.json")))
    pid, identity = run_identity(run)
    require(pid == start["pid"] and identity["start_identity"] == start["start_identity"], "process changed")
    observed_at = current_time()
    raw, item = latest(run, pid)
    require(item["seq"] >= ready["seq"], "receipt sequence moved backwards")
    opened = receipt(read(run / (label + "-open.receipt.json")), run.name, pid)
    require_same_surface(opened, item, start["kind"])
    write_raw(run / (label + "-close-start.receipt.json"), raw)
    write_new(run / (label + ".close-start.json"), {"seq": item["seq"], "receipt_sha256": digest(raw), "observedAt": observed_at})


def observe_closed(run: Path, label: str) -> None:
    start = decode(read(run / (label + ".start.json")))
    close = decode(read(run / (label + ".close-start.json")))
    pid, identity = run_identity(run)
    require(pid == start["pid"] and identity["start_identity"] == start["start_identity"], "process changed")
    raw, item = latest(run, pid)
    require(item["seq"] > close["seq"] and item.get("document_surfaces") == []
            and item.get("html_observations") == [], "closed frame not observed")
    write_raw(run / (label + "-closed.receipt.json"), raw)
    write_new(run / (label + ".closed.json"), {"seq": item["seq"], "observedAt": current_time()})


def duration_ms(start: str, end: str) -> float:
    elapsed = (moment(end, "end") - moment(start, "start")).total_seconds() * 1000
    require(elapsed >= 0, "observation time moved backwards")
    return elapsed


def finish_case(run: Path, label: str) -> None:
    start, ready, close, closed = (decode(read(run / (label + suffix))) for suffix in
                                   (".start.json", ".ready.json", ".close-start.json", ".closed.json"))
    raw = read(run / (label + "-closed.receipt.json"))
    actual = receipt(raw, run.name, start["pid"])
    require(stable_sample(run / "memory-receipt-observations.jsonl", closed["seq"], digest(raw),
                          start["pid"], start["start_identity"], closed["observedAt"], actual), "stable close sample missing")
    write_new(run / (label + ".result.json"), {"pid": start["pid"], "runId": run.name,
              "filename": Path(start["input"]).name, "openSeq": ready["seq"], "readySeq": ready["seq"],
              "closeSeq": closed["seq"], "openStartedAt": start["observedAt"], "readyObservedAt": ready["observedAt"],
              "readyObservedMs": duration_ms(start["observedAt"], ready["observedAt"]),
              "closeStartedAt": close["observedAt"], "closeObservedAt": closed["observedAt"],
              "closeObservedMs": duration_ms(close["observedAt"], closed["observedAt"])})


def observe_cold(run: Path) -> None:
    pid, identity = run_identity(run)
    raw, item = latest(run, pid)
    require(item.get("document_surfaces") == [] and item.get("html_observations") == [], "cold document surface remains")
    require(stable_sample(run / "memory-receipt-observations.jsonl", item["seq"], digest(raw), pid,
                          identity["start_identity"], "1970-01-01T00:00:00+00:00", item), "stable cold sample missing")
    write_new(run / "cold-observed-at.json", {"observedAt": current_time()})


def observe_quit_start(run: Path) -> None:
    run_identity(run)
    write_new(run / "quit-started-at.json", {"observedAt": current_time()})


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-directory", type=Path, required=True)
    parser.add_argument("--prefix")
    parser.add_argument("--cycle", type=int)
    parser.add_argument("--kind", choices=("office", "html"))
    parser.add_argument("--input", type=Path)
    parser.add_argument("stage", choices=("cold", "quit-start", "start", "ready", "close-start", "closed", "finish"))
    args = parser.parse_args()
    try:
        run = args.run_directory
        validate_run(run)
        if args.stage in {"cold", "quit-start"}:
            {"cold": observe_cold, "quit-start": observe_quit_start}[args.stage](run)
        elif args.stage == "start":
            require(args.prefix is not None and args.cycle is not None and args.kind is not None, "case identity required")
            label = validate_label(args.prefix, args.cycle, args.kind)
            require(args.input is not None, "start requires actual absolute input")
            observe_start(run, label, args.input, args.kind)
        else:
            require(args.prefix is not None and args.cycle is not None and args.kind is not None, "case identity required")
            label = validate_label(args.prefix, args.cycle, args.kind)
            {"ready": observe_ready, "close-start": observe_close_start,
             "closed": observe_closed, "finish": finish_case}[args.stage](run, label)
    except (ValueError, RuntimeError, OSError, KeyError, TypeError) as error:
        print(f"ERROR: actual host observation: {error}", file=sys.stderr)
        return 1
    print("OK: actual host observation recorded")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
