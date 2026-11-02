#!/usr/bin/env python3
"""Capture actual before/after receipt evidence for one manual document operation."""

from __future__ import annotations

import argparse
from pathlib import Path
import sys

from document_host_io import decode, digest, read, require, write_new
from document_host_operation import matching_observation, operation_observation, verify_semantics
from generated_asset_identity import module


def context(run: Path, label: str, name: str) -> tuple[dict, dict, str]:
    start = decode(read(run / (label + ".start.json")))
    require(not (run / (label + ".close-start.json")).exists(), "operation must precede closing")
    require(name in ({"sheet", "page", "slide", "single_item"} if start["kind"] == "office"
                     else {"scroll", "reload", "dirty_source"}), "operation and kind differ")
    raw = read(run / (label + "-open.receipt.json"))
    producer = module("_host_op_producer", "produce-document-host-evidence.py")
    opened = decode(raw)
    uri_sha, generation, adapter, _ = producer.opened_identity(
        opened, start["kind"], Path(start["input"]), start["input_sha256"])
    return start, {"uri_sha256": uri_sha, "generation": generation, "adapter_instance_id": adapter}, label + "-" + name


def capture(run: Path, label: str, name: str, phase: str) -> None:
    recorder = module("_host_case_recorder", "record-document-host-observation.py")
    start, identity, base = context(run, label, name)
    pid, process = recorder.run_identity(run)
    require(pid == start["pid"] and process["start_identity"] == start["start_identity"], "process changed")
    observed_at = recorder.current_time()
    raw, actual = recorder.latest(run, pid)
    item = matching_observation(actual, start["kind"], identity["uri_sha256"],
                                identity["generation"], identity["adapter_instance_id"])
    producer = module("_host_op_hash", "produce-document-host-evidence.py")
    require(producer.input_digest(Path(start["input"])) == start["input_sha256"], "original input changed")
    observation = operation_observation(item, start["kind"], name, start["input_sha256"])
    receipt_name = f"{base}-{phase}.receipt.json"
    recorder.write_raw(run / receipt_name, raw)
    write_new(run / f"{base}-{phase}.json", {"receipt": receipt_name, "sha256": digest(raw),
              "seq": actual["seq"], "observedAt": observed_at if phase == "before" else recorder.current_time(),
              "observation": observation})
    if phase == "after":
        write_packet(run, start, identity, base, name)


def write_packet(run: Path, start: dict, identity: dict, base: str, name: str) -> None:
    before, after = (decode(read(run / f"{base}-{phase}.json")) for phase in ("before", "after"))
    require(after["seq"] > before["seq"] or (name == "single_item" and after["seq"] == before["seq"]),
            "operation did not advance completed frame")
    verify_semantics(name, before["observation"], after["observation"])
    packet = {"schema": "katana-host-operation-observation-v1", "name": name, "kind": start["kind"],
              "input_sha256": start["input_sha256"], "pid": start["pid"], "run_id": run.name,
              "start_identity": start["start_identity"], **identity, "seq_before": before["seq"],
              "seq_after": after["seq"], "started_at": before["observedAt"], "completed_at": after["observedAt"],
              "before": before["observation"], "after": after["observation"],
              "before_receipt": before["receipt"], "after_receipt": after["receipt"],
              "observation_artifacts": [{"path": x["receipt"], "sha256": x["sha256"]} for x in (before, after)]}
    write_new(run / (base + ".operation.json"), packet)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-directory", type=Path, required=True)
    parser.add_argument("--prefix", required=True)
    parser.add_argument("--cycle", type=int, required=True)
    parser.add_argument("--kind", choices=("office", "html"), required=True)
    parser.add_argument("--name", required=True)
    parser.add_argument("phase", choices=("before", "after"))
    args = parser.parse_args()
    try:
        recorder = module("_host_case_label", "record-document-host-observation.py")
        recorder.validate_run(args.run_directory)
        label = recorder.validate_label(args.prefix, args.cycle, args.kind)
        capture(args.run_directory, label, args.name, args.phase)
    except (ValueError, RuntimeError, OSError, KeyError, TypeError) as error:
        print(f"ERROR: actual host operation: {error}", file=sys.stderr)
        return 1
    print("OK: actual operation receipt recorded")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
