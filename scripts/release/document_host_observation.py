"""Validate actual readonly receipt and external observer identities."""

import json
from pathlib import Path

from document_host_io import decode, integer, moment, require


def receipt(raw: bytes, run_id: str, pid: int) -> dict:
    value = decode(raw)
    require(isinstance(value, dict) and value.get("schema") == "katana-completed-frame-resource-receipt-v1"
            and value.get("pid") == pid and value.get("run_id") == run_id, "foreign receipt identity")
    integer(value.get("seq"), "receipt sequence", minimum=1)
    return value


def stable_sample(path: Path, seq: int, receipt_sha: str, pid: int, start: str, after: str,
                  expected: dict | None = None) -> bool:
    if not path.is_file() or path.is_symlink():
        return False
    with path.open("r", encoding="utf-8") as stream:
        for line in stream:
            item = json.loads(line)
            memory = item.get("memory", {})
            if (item.get("receipt_seq") != seq or item.get("receipt_sha256") != receipt_sha
                    or item.get("receipt_stable_during_sample") is not True
                    or memory.get("status") != "ok" or memory.get("pid") != pid
                    or memory.get("identity", {}).get("start_identity") != start
                    or moment(memory.get("observed_at"), "sample") < moment(after, "boundary")):
                continue
            from document_host_transcript import resources
            if expected is not None:
                require(all(item[key] == expected[key] for key in ("host", "kdv", "document_workers_live")),
                        "observer differs from actual receipt")
            resources({"host": item["host"], "kdv": item["kdv"], "workers": item["document_workers_live"],
                       "children": item["kernel_processes"]["children"]}, idle=True)
            return True
    return False
