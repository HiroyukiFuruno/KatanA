#!/usr/bin/env python3
"""実pair rawを検査して公開可能な型付きtranscriptへ落とす。UI入力や測定値は生成しない。"""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
from pathlib import Path
import sys
from urllib.parse import unquote, urlsplit

from document_host_acceptance import APPROVAL, EVIDENCE_RELATIVE, PUBLISHED_EVIDENCE_RELATIVE, POLICY, provenance
from document_host_io import decode, digest, fields, integer, legacy, read, require, sha, write_new
from document_host_transcript import HOST, KDV, verify as verify_transcript


class PrivateRun:
    def __init__(self, directory: Path):
        require(directory.is_absolute() and directory.is_dir() and not directory.is_symlink(), "actual private run directory required")
        self.directory = directory
        self.inventory: set[str] = set()

    def raw(self, name: str) -> bytes:
        path = self.directory / name
        require(path.resolve().is_relative_to(self.directory.resolve()), "private run path escapes directory")
        for part in path.relative_to(self.directory).parts:
            require(part not in {".", ".."}, "private run traversal")
        data = read(path)
        self.inventory.add(digest(data))
        return data

    def json(self, name: str) -> object:
        return decode(self.raw(name))


def input_digest(path: Path) -> str:
    require(path.is_absolute() and path.is_file() and not path.is_symlink(), "private original input must be a regular absolute file")
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def receipt(run: PrivateRun, name: str, pid: int) -> tuple[dict, str]:
    raw = run.raw(name)
    record = decode(raw)
    require(isinstance(record, dict) and record.get("schema") == "katana-completed-frame-resource-receipt-v1"
            and record.get("pid") == pid and record.get("run_id") == run.directory.name, "actual receipt run/PID/schema differs")
    integer(record.get("seq"), "receipt seq", minimum=1)
    fields(record.get("host"), HOST, "raw host resources")
    fields(record.get("kdv"), KDV, "raw KDV resources")
    return record, digest(raw)


def opened_identity(record: dict, kind: str, expected_input: Path, expected_sha: str) -> tuple[str, int, int | None, dict]:
    observations = record.get("html_observations" if kind == "html" else "document_surfaces")
    other = record.get("document_surfaces" if kind == "html" else "html_observations")
    require(isinstance(observations, list) and len(observations) == 1 and other == [], "one actual document surface required")
    item = observations[0]
    require(isinstance(item, dict), "actual surface observation required")
    uri = item.get("source_uri" if kind == "html" else "uri")
    require(isinstance(uri, str), "actual source URI required")
    parsed = urlsplit(uri)
    require(parsed.scheme == "file" and parsed.netloc in {"", "localhost"}
            and Path(unquote(parsed.path)) == expected_input, "actual source URI differs from selected original")
    generation = item.get("katana_renderer_session_epoch" if kind == "html" else "generation")
    adapter = integer(item.get("katana_adapter_instance_id"), "actual HTML adapter", minimum=1) if kind == "html" else None
    require(integer(record["host"]["frames"], "actual opened host frames", minimum=1) > 0, "opened receipt has no rendered host frame")
    if kind == "html":
        state = item.get("loaded_source")
        fnv = input_fnv(expected_input)
        require(isinstance(state, dict) and state.get("preview_input_fnv1a64") == state.get("last_imported_disk_fnv1a64") == fnv,
                "opened HTML loaded source differs from actual original bytes")
        rendered = {"kind": "html", "loaded_source": state, "input_fnv1a64": fnv,
                    **{key: item[key] for key in ("frame_generation", "viewport_width", "viewport_height", "content_height", "scroll_y")}}
        fmt = "html"
    else:
        measured = record.get("document_observations")
        matches = [value for value in measured if value.get("uri") == uri and value.get("generation") == generation] if isinstance(measured, list) else []
        require(len(matches) == 1, "opened Office actual loaded-source/frame observation missing")
        observed = matches[0]
        require(observed.get("source_revision") == "sha256:"+expected_sha, "opened Office loaded revision differs from actual supplied bytes")
        rendered = {"kind": "office", **{key: observed[key] for key in ("source_revision", "idle", "frame")}}
        fmt = legacy().SUPPLIED_OFFICE_FIXTURES.get(expected_sha)
    from document_host_transcript import rendered_input
    rendered_input(rendered, fmt, expected_sha)
    return digest(uri.encode()), integer(generation, "actual generation", minimum=1), adapter, rendered


def input_fnv(path: Path) -> str:
    value = 14695981039346656037
    with path.open("rb") as handle:
        while block := handle.read(1024*1024):
            for byte in block:
                value = ((value ^ byte) * 1099511628211) & ((1 << 64)-1)
    return f"{value:016x}"


def operation(run: PrivateRun, filename: str, pid: int, start: str, inputs: dict[str, str], cases: list[dict], html_fnv: str) -> dict:
    packet = run.json(filename)
    fields(packet, {"schema", "name", "kind", "input_sha256", "pid", "run_id", "start_identity", "uri_sha256",
                    "generation", "adapter_instance_id", "seq_before", "seq_after", "started_at", "completed_at", "before", "after",
                    "before_receipt", "after_receipt", "observation_artifacts"}, "actual operation packet")
    require(packet["schema"] == "katana-host-operation-observation-v1" and packet["pid"] == pid
            and packet["run_id"] == run.directory.name and packet["start_identity"] == start, "operation actual process differs")
    require(packet["kind"] in inputs and packet["input_sha256"] == inputs[packet["kind"]], "operation input differs")
    before, _ = receipt(run, packet["before_receipt"], pid)
    after, _ = receipt(run, packet["after_receipt"], pid)
    require(before["seq"] == packet["seq_before"] and after["seq"] == packet["seq_after"], "operation frame receipts differ")
    # WHY: 本文やAX全文はprivateで実読し、public出力へは内容でなく検査済digestだけを残す。
    require(isinstance(packet["observation_artifacts"], list) and packet["observation_artifacts"], "actual operation raw observations required")
    for item in packet["observation_artifacts"]:
        fields(item, {"path", "sha256"}, "operation private observation")
        require(digest(run.raw(item["path"])) == sha(item["sha256"], "operation raw observation"), "operation raw digest changed")
    result = {key: packet[key] for key in packet if key not in {"schema", "before_receipt", "after_receipt", "observation_artifacts"}}
    result["raw_sha256"] = digest(run.raw(filename))
    from document_host_transcript import operation as verify_operation
    verify_operation(result, {"pid": pid, "run_id": run.directory.name, "start_identity": start,
                              "input_sha256": inputs["office"], "html_sha256": inputs["html"], "cases": cases},
                     {(case["uri_sha256"], case["adapter_instance_id"], case["generation"]) for case in cases})
    if packet["kind"] == "html":
        for raw_receipt, observation in ((before, packet["before"]), (after, packet["after"])):
            items = raw_receipt.get("html_observations")
            require(isinstance(items, list) and len(items) == 1, "operation HTML frame missing")
            item = items[0]
            require(digest(item["source_uri"].encode()) == packet["uri_sha256"]
                    and item["katana_adapter_instance_id"] == packet["adapter_instance_id"]
                    and item.get("loaded_source") == observation["loaded_source"], "operation source differs from actual loaded document")
            require(observation["loaded_source"]["last_imported_disk_fnv1a64"] == html_fnv,
                    "actual loaded source does not bind the original file bytes")
            if packet["name"] in {"scroll", "reload"}:
                require(item["katana_renderer_session_epoch"] == observation["session"]
                        and item["frame_generation"] == observation["frame_generation"], "operation observations differ from actual frame")
            if packet["name"] == "scroll":
                require(item["scroll_y"] == observation["scroll_y"], "scroll coordinates differ from actual receipt")
    elif packet["kind"] == "office":
        for raw_receipt, observation in ((before, packet["before"]), (after, packet["after"])):
            items = raw_receipt.get("document_observations")
            matches = [item for item in items if digest(item["uri"].encode()) == packet["uri_sha256"]
                       and item["generation"] == packet["generation"]] if isinstance(items, list) else []
            require(len(matches) == 1 and all(matches[0].get(key) == observation[key] for key in ("source_revision", "idle", "frame")),
                    "Office operation differs from actual source/frame observation")
    return result


def pair(root: Path, entry: dict, source: dict, bundle: dict, producer_sha: str) -> dict:
    fields(entry, {"directory", "prefix", "office_input", "html_input", "operations"}, "private run mapping")
    run = PrivateRun(Path(entry["directory"]))
    require(isinstance(entry["prefix"], str) and entry["prefix"] and "/" not in entry["prefix"], "explicit run label prefix required")
    office, html = Path(entry["office_input"]), Path(entry["html_input"])
    inputs = {"office": input_digest(office), "html": input_digest(html)}
    require(run.json("source-identity.before.json") == source and run.json("source-identity.after.json") == source,
            "run source/registry identity changed or was not recorded")
    require(run.json("bundle-identity.before.json") == bundle and run.json("bundle-identity.after.json") == bundle,
            "run actual bundle identity changed or was not recorded")
    require(run.raw("head.before").strip().decode() == source["source_commit"]
            and run.raw("head.before") == run.raw("head.after"), "run HEAD changed")
    pid = int(run.raw("app.pid").decode().strip())
    require(run.raw("app.exit").strip() == b"0", "app did not exit normally")
    raw_lines = run.raw("memory-receipt-observations.jsonl").splitlines()
    require(raw_lines, "all external observations required")
    observations = [decode(line) for line in raw_lines]
    launch_raw = run.raw("launch.json")
    samples = []
    start = None
    for value in observations:
        require(isinstance(value, dict) and isinstance(value.get("memory"), dict), "memory observation missing")
        require(value.get("launch_record_sha256") == digest(launch_raw), "actual launch record differs from process/memory observation")
        memory = value["memory"]
        require(memory.get("status") == "ok" and memory.get("pid") == pid and memory.get("sampler") == "external_process_memory_v2",
                "failed/foreign memory observation cannot be replaced by zero")
        identity = memory.get("identity")
        require(isinstance(identity, dict) and identity.get("executable") == bundle["main_path"], "memory executable differs from actual bundle")
        if start is None:
            start = identity.get("start_identity")
        require(identity.get("start_identity") == start, "PID was reused")
        kernel = value.get("kernel_processes")
        require(isinstance(kernel, dict) and kernel.get("main_alive") is True and isinstance(kernel.get("children"), list),
                "actual OS descendant snapshot missing; do not synthesize no children")
        children = []
        for child in kernel["children"]:
            require(child.get("status") == "observed" and child.get("role") == "office_sidecar"
                    and child.get("identity", {}).get("executable") == bundle["worker_path"]
                    and child.get("executable_sha256") == bundle["sidecar_sha256"],
                    "unobserved/foreign OS child cannot be treated as an absent Office worker")
            child_memory = child.get("memory")
            require(isinstance(child_memory, dict) and child_memory.get("status") == "ok"
                    and child_memory.get("identity") == child["identity"] and child_memory.get("pid") == child["pid"],
                    "actual worker memory/identity missing")
            children.append({"pid": child["pid"], "ppid": child["ppid"], "start_identity": child["identity"]["start_identity"],
                             "role": "office_sidecar", "member": "KatanA Desktop.app/Contents/MacOS/kdv-office-worker",
                             "executable_sha256": child["executable_sha256"], "executable_path_sha256": digest(child["identity"]["executable"].encode()),
                             "rss_bytes": child_memory["rss_bytes"], "physical_footprint_bytes": child_memory["physical_footprint_bytes"],
                             "cpu_percent": child["cpu_percent"], "observed_at": child_memory["observed_at"]})
        samples.append({"receipt_seq": value["receipt_seq"], "receipt_sha256": value["receipt_sha256"],
                        "stable": value["receipt_stable_during_sample"], "observed_at": memory["observed_at"],
                        "finished_at": value["sample_finished_at"], "pid": pid, "start_identity": start,
                        "rss_bytes": memory["rss_bytes"], "physical_footprint_bytes": memory["physical_footprint_bytes"],
                        "host": value["host"], "kdv": value["kdv"], "workers": value["document_workers_live"],
                        "main_cpu_percent": kernel["main_cpu_percent"], "kernel_observed_at": kernel["observed_at"], "children": children})
    from document_host_io import moment
    cutoff = run.json("cold-observed-at.json")["observedAt"]
    cold_candidates = [index for index, sample in enumerate(samples) if sample["stable"]
                       and moment(sample["finished_at"], "cold sample") <= moment(cutoff, "cold cutoff")]
    require(bool(cold_candidates), "cold sample missing")
    cases = []
    for cycle in range(1, 11):
        for kind in ("html", "office"):
            label = f"{entry['prefix']}-cycle{cycle:02}-{kind}"
            result = run.json(label+".result.json")
            require(isinstance(result, dict) and result.get("pid") == pid and result.get("runId") == run.directory.name,
                    "operation result PID/run differs")
            expected_input = html if kind == "html" else office
            require(result.get("filename") == expected_input.name, "result input filename differs")
            opened, open_sha = receipt(run, label+"-open.receipt.json", pid)
            closed, close_sha = receipt(run, label+"-closed.receipt.json", pid)
            uri_sha, generation, adapter, rendered = opened_identity(opened, kind, expected_input, inputs[kind])
            require(opened["seq"] == result.get("openSeq") == result.get("readySeq") and closed["seq"] == result.get("closeSeq"),
                    "result sequence differs from actual open/close receipt")
            require(closed.get("document_surfaces") == [] and closed.get("html_observations") == [], "closed document surface retained")
            selected = [index for index, sample in enumerate(samples) if sample["receipt_seq"] == closed["seq"] and sample["stable"]
                        and moment(sample["observed_at"], "sample") >= moment(result["closeObservedAt"], "close")]
            require(bool(selected), "completed-close stable memory sample missing")
            chosen = samples[selected[0]]
            require(chosen["receipt_sha256"] == close_sha and chosen["host"] == closed["host"]
                    and chosen["kdv"] == closed["kdv"] and chosen["workers"] == closed["document_workers_live"],
                    "external sample differs from actual close receipt")
            disposition = None
            fmt = "html" if kind == "html" else legacy().SUPPLIED_OFFICE_FIXTURES.get(inputs["office"])
            if fmt == "pptx" and result["readyObservedMs"] > 15000:
                from document_fidelity_divergence import PPTX_FIRST_FRAME_APPROVAL
                disposition = {"status": "known_krr_pptx_first_frame_miss", "owner": "KRR", "approval": PPTX_FIRST_FRAME_APPROVAL,
                               "release_target": "v0.22.42", "scope": "katana-host", "limit_ms": 15000,
                               "measured_first_frame_ms": result["readyObservedMs"], "unmet": True}
            cases.append({"cycle": cycle, "kind": kind, "format": fmt, "input_sha256": inputs[kind], "uri_sha256": uri_sha,
                          "generation": generation, "adapter_instance_id": adapter, "open_seq": opened["seq"], "close_seq": closed["seq"],
                          "open_started_at": result["openStartedAt"], "ready_at": result["readyObservedAt"],
                          "ready_ms": result["readyObservedMs"], "close_started_at": result["closeStartedAt"],
                          "closed_at": result["closeObservedAt"], "close_ms": result["closeObservedMs"],
                          "open_receipt_sha256": open_sha, "close_receipt_sha256": close_sha,
                          "selected_sample": selected[0], "first_frame_disposition": disposition, "render_observation": rendered})
    settings = run.json("run-settings.json")
    exit_observed = run.json("app-exit-observed.json")
    require(isinstance(exit_observed, dict) and exit_observed.get("pid") == pid and exit_observed.get("exit_code") == 0,
            "normal exit observation missing")
    operations = [operation(run, filename, pid, start, inputs, cases, input_fnv(html)) for filename in entry["operations"]]
    terminal = run.json("kernel-processes.after-exit.json")
    require(isinstance(terminal, dict) and terminal.get("main_alive") is False and terminal.get("children") == [],
            "actual OS owned processes remain/unobserved after normal exit")
    transcript = {"schema": "katana-host-transcript-v1", "provenance": source, "run_id": run.directory.name,
                  "target": bundle["target"], "pid": pid, "start_identity": start,
                  "main_sha256": bundle["main_sha256"], "sidecar_sha256": bundle["sidecar_sha256"],
                  "input_sha256": inputs["office"], "html_sha256": inputs["html"], "cold_observed_at": cutoff,
                  "cold_sample": cold_candidates[-1], "samples": samples, "cases": cases, "operations": operations,
                  "quit_started_at": run.json("quit-started-at.json")["observedAt"], "exit_observed_at": exit_observed["observed_at"],
                  "exit_code": 0, "raw_artifacts": sorted(run.inventory), "settings": settings, "producer_sha256": producer_sha}
    transcript["post_exit_processes"] = {key: terminal[key] for key in ("observed_at", "main_alive", "children")}
    transcript["heartbeat_observations"] = [decode(line) for line in run.raw("heartbeat-observations.jsonl").splitlines()]
    launch = run.json("launch.json")
    require(isinstance(launch, dict) and launch.get("target") == bundle["target"], "actual launch target differs from run map")
    requested = "arm64" if bundle["target"] == "macos-arm64" else "x86_64"
    require(launch.get("argv") == ["/usr/bin/arch", "-"+requested, bundle["main_path"]], "actual launch did not select the claimed slice")
    import json
    transcript["launch"] = {"method": "explicit_macos_arch", "target": launch["target"], "requested_slice": requested,
                            "observer_machine": launch["observer_machine"], "argv_sha256": digest(json.dumps(launch["argv"], separators=(",", ":")).encode())}
    transcript["raw_artifacts"] = sorted(run.inventory)
    helper = legacy()
    verify_transcript(transcript, fixtures=helper.SUPPLIED_OFFICE_FIXTURES, html_sha=helper.ORIGINAL_HTML_SHA256,
                      provenance=source, main_sha=bundle["main_sha256"], worker_sha=bundle["sidecar_sha256"], producer_sha=producer_sha)
    require(input_digest(office) == inputs["office"] and input_digest(html) == inputs["html"], "original input changed while producing evidence")
    return transcript


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--run-map", type=Path, required=True, help="private map of actual run directories, inputs and operation packets")
    parser.add_argument("--output-directory", type=Path, required=True)
    parser.add_argument("--phase", choices=("prepublish", "postpublication"), default="prepublish")
    parser.add_argument("--publication-receipt", type=Path)
    args = parser.parse_args()
    try:
        root = args.root.resolve()
        mapping = fields(decode(read(args.run_map)), {"schema", "provenance", "bundle", "runs", "upstream_assessments"}, "private run map")
        require(mapping["schema"] == "katana-host-private-run-map-v1", "wrong private run-map schema")
        source = provenance(root, mapping["provenance"])
        bundle = fields(mapping["bundle"], {"target", "main_path", "worker_path", "main_sha256", "sidecar_sha256", "archive_sha256", "provenance"}, "actual bundle")
        require(bundle["provenance"] == source, "actual bundle source differs")
        require(input_digest(Path(bundle["main_path"])) == bundle["main_sha256"]
                and input_digest(Path(bundle["worker_path"])) == bundle["sidecar_sha256"], "actual executable bytes changed")
        output = args.output_directory.resolve()
        require(output.is_relative_to(root) and "evidence" in output.relative_to(root).parts and "tmp" not in output.relative_to(root).parts,
                "sanitized output must be public repository evidence")
        require(not output.exists(), "refusing to overwrite earlier evidence")
        require(isinstance(mapping["runs"], list) and len(mapping["runs"]) == 7, "seven actual pair runs required")
        require((args.phase == "postpublication") == (args.publication_receipt is not None),
                "postpublication requires the actual Release download receipt")
        producer_sha = digest(read(Path(__file__)))
        transcripts = [pair(root, item, source, bundle, producer_sha) for item in mapping["runs"]]
        # WHY: 全raw検査を終えてからpublicへ書く。部分検査の結果を本受入データとして残さない。
        refs = []
        for index, transcript in enumerate(transcripts, 1):
            ref = write_new(output / f"run-{index:02}.json", transcript)
            ref["path"] = (output / f"run-{index:02}.json").relative_to(root).as_posix()
            refs.append(ref)
        public_bundle = {key: bundle[key] for key in ("target", "main_sha256", "sidecar_sha256", "archive_sha256", "provenance")}
        public_bundle["schema"] = "katana-host-bundle-v1"
        bundle_ref = write_new(output / "bundle.json", public_bundle)
        bundle_ref["path"] = (output / "bundle.json").relative_to(root).as_posix()
        evidence = {"schema": "katana-host-acceptance-v1", "version": "v0.22.42", "scope": "katana-host", "policy": POLICY,
                    "approval_sha256": digest(read(root / APPROVAL)), "phase": args.phase,
                    "claim": "candidate_host_accepted_postpublication_pending" if args.phase == "prepublish" else "published_macos_host_accepted", "provenance": source,
                    "candidate": {"target": bundle["target"], "main_sha256": bundle["main_sha256"],
                                  "sidecar_sha256": bundle["sidecar_sha256"], "archive_sha256": bundle["archive_sha256"], "bundle_manifest": bundle_ref},
                    "transcripts": refs, "upstream_assessments": mapping["upstream_assessments"],
                    "other_targets": {target: "document_qa_not_run" for target in legacy().SUPPORTED_TARGETS - {bundle["target"]}},
                    "producer_sha256": producer_sha, "publication": {"status": "postpublication_pending"}}
        if args.publication_receipt is not None:
            receipt_path = args.publication_receipt.resolve()
            require(receipt_path.is_relative_to(root), "public Release receipt must be inside the repository")
            evidence["publication"] = {"status": "published_macos_host_accepted",
                                       "receipt": {"path": receipt_path.relative_to(root).as_posix(), "sha256": digest(read(receipt_path))}}
        provenance(root, source)
        require(input_digest(Path(bundle["main_path"])) == bundle["main_sha256"]
                and input_digest(Path(bundle["worker_path"])) == bundle["sidecar_sha256"], "actual executable bytes changed during producer")
        from document_host_acceptance import verify
        canonical = root / (EVIDENCE_RELATIVE if args.phase == "prepublish" else PUBLISHED_EVIDENCE_RELATIVE)
        write_new(canonical, evidence)
        verify(root, canonical, phase=args.phase)
    except (ValueError, RuntimeError, OSError, KeyError, TypeError) as error:
        print(f"ERROR: host evidence producer: {error}", file=sys.stderr)
        return 1
    print("OK: actual private raw inspected; public sanitized transcripts validated; "
          + ("postpublication pending" if args.phase == "prepublish" else "actual published Mac host acceptance recorded"))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
