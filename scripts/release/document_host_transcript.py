"""全sampleと実操作を保ったhost transcriptの独立検査。合成fixtureは受入の入力にしない。"""

from __future__ import annotations

import re

from document_fidelity_divergence import verify_pptx_first_frame_disposition
from document_host_io import fields, integer, moment, number, require, sha

HOST = {"previews", "html_surfaces", "document_surfaces", "frames", "textures", "cache_entries"}
KDV = {"live_document_sessions", "live_spreadsheet_workers", "live_worker_workspaces",
       "retained_artifact_bytes", "cached_page_count", "cached_page_bytes",
       "cached_spreadsheet_cell_count", "cached_spreadsheet_cell_bytes"}
SAMPLE = {"receipt_seq", "receipt_sha256", "stable", "observed_at", "finished_at", "pid",
          "start_identity", "rss_bytes", "physical_footprint_bytes", "host", "kdv", "workers",
          "main_cpu_percent", "kernel_observed_at", "children"}
CHILD = {"pid", "ppid", "start_identity", "role", "member", "executable_sha256", "executable_path_sha256",
         "rss_bytes", "physical_footprint_bytes", "cpu_percent", "observed_at"}
CASE = {"cycle", "kind", "format", "input_sha256", "uri_sha256", "generation", "adapter_instance_id", "open_seq", "close_seq",
        "open_started_at", "ready_at", "ready_ms", "close_started_at", "closed_at", "close_ms",
        "open_receipt_sha256", "close_receipt_sha256", "selected_sample", "first_frame_disposition", "render_observation"}
TRANSCRIPT = {"schema", "provenance", "run_id", "target", "pid", "start_identity", "main_sha256",
              "sidecar_sha256", "input_sha256", "html_sha256", "cold_observed_at", "cold_sample",
              "samples", "cases", "operations", "quit_started_at", "exit_observed_at", "exit_code",
              "raw_artifacts", "settings", "producer_sha256"}
TRANSCRIPT.add("post_exit_processes")
TRANSCRIPT.add("heartbeat_observations")
TRANSCRIPT.add("launch")
SETTINGS = {"debug", "readonly_recorder", "malloc_inspector", "vmmap_inspector", "heap_inspector",
            "isolation", "clean_machine", "protections_disabled", "personal_profile_changed",
            "terms_scope"}


def resources(value: dict, *, idle: bool) -> None:
    for label, expected in (("host", HOST), ("kdv", KDV)):
        counts = fields(value[label], expected, label)
        for key, count in counts.items():
            integer(count, f"{label}.{key}")
            require(not idle or count == 0, f"resources not released: {label}.{key}")
    integer(value["workers"], "workers")
    require(not idle or value["workers"] == 0, "document worker retained")
    require(not idle or value["children"] == [], "OS-owned child remains after close")


def settings(value: object) -> None:
    record = fields(value, SETTINGS, "settings")
    require(type(record["debug"]) is bool, "DEBUG must record its actual boolean value")
    require(record["readonly_recorder"] is True, "readonly recorder required")
    require(all(record[key] is False for key in ("malloc_inspector", "vmmap_inspector", "heap_inspector",
                                                "protections_disabled", "personal_profile_changed")),
            "inspector/protection/profile configuration differs from approved QA")
    require(record["isolation"] == "isolated_profile", "isolated profile required")
    require(record["clean_machine"] in (False, None), "isolated profile is not a clean machine")
    require(record["terms_scope"] == "this_isolated_run_only", "Terms approval cannot be copied")


def loaded_source(value: object) -> dict:
    record = fields(value, {"preview_input_fnv1a64", "last_imported_disk_fnv1a64", "document_revision", "saved_revision",
                            "dirty", "is_reference"}, "actual loaded source")
    for key in ("preview_input_fnv1a64", "last_imported_disk_fnv1a64"):
        require(isinstance(record[key], str) and re.fullmatch(r"[0-9a-f]{16}", record[key]) is not None,
                "loaded source must retain exact FNV64 labels, not a fabricated SHA")
    integer(record["document_revision"], "document revision")
    integer(record["saved_revision"], "saved revision")
    require(type(record["dirty"]) is bool and record["is_reference"] is False, "actual editable source state required")
    require(record["dirty"] == (record["document_revision"] != record["saved_revision"]), "dirty/revision state differs")
    return record


def rendered_input(value: object, fmt: str, input_sha: str) -> None:
    if fmt == "html":
        record = fields(value, {"kind", "loaded_source", "input_fnv1a64", "frame_generation", "viewport_width",
                                "viewport_height", "content_height", "scroll_y"}, "actual opened HTML input/frame")
        state = loaded_source(record["loaded_source"])
        require(record["kind"] == "html" and state["dirty"] is False
                and state["preview_input_fnv1a64"] == state["last_imported_disk_fnv1a64"] == record["input_fnv1a64"],
                "opened HTML source differs from actual supplied input bytes")
        integer(record["frame_generation"], "actual HTML frame generation", minimum=1)
        for key in ("viewport_width", "viewport_height", "content_height"):
            require(number(record[key], "actual HTML "+key) > 0, "HTML frame geometry missing")
        number(record["scroll_y"], "actual HTML scroll")
    else:
        record = fields(value, {"kind", "source_revision", "idle", "frame"}, "actual opened Office input/frame")
        require(record["kind"] == "office" and record["source_revision"] == "sha256:"+input_sha and record["idle"] is True,
                "opened Office source revision differs from actual supplied input bytes")
        frame = fields(record["frame"], {"format", "active_index", "item_count", "surface_kind"}, "actual opened Office frame")
        require(frame["format"] == fmt and frame["surface_kind"] == ("Grid" if fmt == "xlsx" else "Page"), "opened Office frame kind differs")
        require(integer(frame["active_index"], "actual opened index") < integer(frame["item_count"], "actual item count", minimum=1),
                "opened Office frame index out of range")


def operation(value: object, transcript: dict, used: set[tuple[str, int | None, int]]) -> None:
    record = fields(value, {"name", "kind", "input_sha256", "pid", "run_id", "start_identity",
                            "uri_sha256", "generation", "adapter_instance_id", "seq_before", "seq_after", "started_at",
                            "completed_at", "before", "after", "raw_sha256"}, "operation")
    for key in ("pid", "run_id", "start_identity"):
        require(record[key] == transcript[key], f"operation {key} differs from run")
    require(record["kind"] in {"html", "office"}, "invalid operation kind")
    expected_input = transcript["html_sha256" if record["kind"] == "html" else "input_sha256"]
    require(record["input_sha256"] == expected_input, "operation input differs from run")
    uri = sha(record["uri_sha256"], "operation URI")
    generation = integer(record["generation"], "operation generation", minimum=1)
    adapter = record["adapter_instance_id"]
    if record["kind"] == "html":
        integer(adapter, "actual HTML adapter instance", minimum=1)
    else:
        require(adapter is None, "Office must not invent an HTML adapter identity")
    require((uri, adapter, generation) in used, "operation is not bound to an opened generation")
    before_seq = integer(record["seq_before"], "operation before seq", minimum=1)
    after_seq = integer(record["seq_after"], "operation after seq", minimum=1)
    require(after_seq > before_seq or (record["name"] == "single_item" and after_seq == before_seq),
            "operation did not advance completed frame")
    started = moment(record["started_at"], "operation started")
    completed = moment(record["completed_at"], "operation completed")
    require(started <= completed, "operation time reversed")
    case = next((item for item in transcript["cases"] if item["uri_sha256"] == uri
                 and item["generation"] == generation and item["adapter_instance_id"] == adapter), None)
    require(case is not None and case["open_seq"] <= before_seq <= after_seq < case["close_seq"],
            "operation sequence outside opened generation")
    require(moment(case["ready_at"], "ready") <= started <= completed <= moment(case["close_started_at"], "close"),
            "operation outside open document interval")
    sha(record["raw_sha256"], "operation private raw digest")
    before, after, name = record["before"], record["after"], record["name"]
    require(isinstance(before, dict) and isinstance(after, dict), "operation observations required")
    if name == "scroll" and record["kind"] == "html":
        for item in (before, after):
            fields(item, {"scroll_y", "frame_generation", "session", "saved_input_sha256", "loaded_source"}, name)
            number(item["scroll_y"], "scroll y")
            integer(item["frame_generation"], "scroll frame", minimum=1)
            integer(item["session"], "scroll session", minimum=1)
            require(item["saved_input_sha256"] == expected_input, "scroll changed source")
            loaded_source(item["loaded_source"])
        require(after["scroll_y"] > before["scroll_y"] and after["frame_generation"] > before["frame_generation"]
                and after["session"] == before["session"] and after["loaded_source"] == before["loaded_source"],
                "scroll did not reach a newer frame while preserving source")
    elif name == "reload" and record["kind"] == "html":
        # session/frame_sessionはKatanAのrenderer epochと実frame受理時epochを表し、上流IDではない。
        for item in (before, after):
            fields(item, {"session", "frame_session", "frame_generation", "saved_input_sha256", "loaded_source"}, name)
            for key in ("session", "frame_session", "frame_generation"):
                integer(item[key], key, minimum=1)
            require(item["saved_input_sha256"] == expected_input, "reload changed source")
            loaded_source(item["loaded_source"])
        require(after["session"] > before["session"] and after["frame_session"] == after["session"]
                and after["loaded_source"] == before["loaded_source"],
                "reload reused the old renderer session/frame")
    elif name == "dirty_source" and record["kind"] == "html":
        for item in (before, after):
            fields(item, {"loaded_source", "saved_input_sha256"}, name)
            state = loaded_source(item["loaded_source"])
            require(state["dirty"] is True and item["saved_input_sha256"] == expected_input,
                    "dirty source state/saved input lost")
        require(before == after, "unsaved source revision/content identity was discarded")
    elif name in {"sheet", "page", "slide", "single_item"} and record["kind"] == "office":
        for item in (before, after):
            fields(item, {"source_revision", "idle", "frame", "reason"}, name)
            require(item["source_revision"] == "sha256:"+expected_input and item["idle"] is True,
                    "Office source/revision is not the actual idle input")
            frame = fields(item["frame"], {"format", "active_index", "item_count", "surface_kind"}, "actual Office frame")
            require(frame["format"] == case["format"] and frame["surface_kind"] == ("Grid" if case["format"] == "xlsx" else "Page"),
                    "actual Office frame kind differs")
            integer(frame["item_count"], "navigation count", minimum=1)
            integer(frame["active_index"], "navigation index")
            require(frame["active_index"] < frame["item_count"], "navigation index out of range")
        require(before["frame"]["item_count"] == after["frame"]["item_count"], "Office item count changed")
        if name == "single_item":
            require(before == after and before["frame"]["item_count"] == 1 and before["frame"]["active_index"] == 0
                    and before["reason"] == "single_item", "single-item reason cannot hide a multi-item navigation failure")
        else:
            require(before["frame"]["item_count"] > 1 and before["frame"]["active_index"] != after["frame"]["active_index"]
                    and before["reason"] is None and after["reason"] is None, "document navigation not observed")
    else:
        require(False, "unknown operation cannot be deferred as upstream")


def verify(value: object, *, fixtures: dict[str, str], html_sha: str, provenance: dict,
           main_sha: str, worker_sha: str, producer_sha: str) -> dict:
    record = fields(value, TRANSCRIPT, "host transcript")
    require(record["schema"] == "katana-host-transcript-v1", "wrong transcript schema")
    require(record["provenance"] == provenance, "run source/registry differs from admission")
    require(record["target"] in {"macos-arm64", "macos-x86_64"}, "host run cannot claim other OS")
    require(record["main_sha256"] == main_sha and record["sidecar_sha256"] == worker_sha, "run bundle changed")
    require(record["producer_sha256"] == producer_sha, "run producer changed")
    input_sha = sha(record["input_sha256"], "input")
    require(input_sha in fixtures and record["html_sha256"] == html_sha, "run input is not the fixed fixture")
    fmt = fixtures[input_sha]
    require(isinstance(record["run_id"], str) and re.fullmatch(r"[A-Za-z0-9._-]{1,128}", record["run_id"]) is not None,
            "invalid run id")
    integer(record["pid"], "pid", minimum=1)
    require(isinstance(record["start_identity"], str) and record["start_identity"].isdigit()
            and int(record["start_identity"]) > 0, "process start identity required")
    settings(record["settings"])
    launch = fields(record["launch"], {"method", "target", "requested_slice", "observer_machine", "argv_sha256"}, "actual slice launch")
    require(launch["method"] == "explicit_macos_arch" and launch["target"] == record["target"]
            and launch["requested_slice"] == ("arm64" if record["target"] == "macos-arm64" else "x86_64")
            and launch["observer_machine"] in {"arm64", "x86_64"}, "run map target differs from actual explicit launch slice")
    sha(launch["argv_sha256"], "actual launch argv hash")
    require(isinstance(record["raw_artifacts"], list) and record["raw_artifacts"], "raw digest inventory required")
    require(len(set(record["raw_artifacts"])) == len(record["raw_artifacts"]), "duplicate raw artifact inventory")
    for raw_sha in record["raw_artifacts"]:
        sha(raw_sha, "private raw hash")
    samples = record["samples"]
    require(isinstance(samples, list) and samples, "all external memory samples required")
    previous_end = None
    previous_seq = 0
    for item in samples:
        fields(item, SAMPLE, "memory sample")
        require(item["pid"] == record["pid"] and item["start_identity"] == record["start_identity"],
                "memory sample PID/start identity changed")
        seq = integer(item["receipt_seq"], "sample receipt seq", minimum=1)
        require(seq >= previous_seq, "sample receipt sequences reversed")
        previous_seq = seq
        sha(item["receipt_sha256"], "sample receipt")
        require(type(item["stable"]) is bool, "sample stability not recorded")
        observed = moment(item["observed_at"], "memory observed")
        ended = moment(item["finished_at"], "memory finished")
        require(observed <= ended and (previous_end is None or observed >= previous_end), "memory sample order invalid")
        previous_end = ended
        integer(item["rss_bytes"], "sample RSS", minimum=1)
        integer(item["physical_footprint_bytes"], "sample physical footprint", minimum=1)
        number(item["main_cpu_percent"], "actual OS main CPU")
        require(observed <= moment(item["kernel_observed_at"], "kernel observation") <= ended, "kernel sample outside observation interval")
        require(isinstance(item["children"], list), "actual OS descendant observations required")
        seen_children = set()
        for child in item["children"]:
            fields(child, CHILD, "actual OS child")
            child_pid = integer(child["pid"], "actual child PID", minimum=1)
            require(child_pid != record["pid"] and child_pid not in seen_children, "main PID/duplicate cannot stand in for sidecar")
            seen_children.add(child_pid)
            integer(child["ppid"], "actual child PPID", minimum=1)
            require(isinstance(child["start_identity"], str) and child["start_identity"].isdigit()
                    and int(child["start_identity"]) > 0, "actual child start identity required")
            require(child["role"] == "office_sidecar" and child["member"] == "KatanA Desktop.app/Contents/MacOS/kdv-office-worker"
                    and child["executable_sha256"] == worker_sha, "actual descendant is not selected bundled worker")
            sha(child["executable_path_sha256"], "actual worker path hash")
            integer(child["rss_bytes"], "actual worker RSS", minimum=1)
            integer(child["physical_footprint_bytes"], "actual worker physical", minimum=1)
            number(child["cpu_percent"], "actual worker CPU")
            require(observed <= moment(child["observed_at"], "actual worker observation") <= ended,
                    "actual worker observation outside same poll")
        resources(item, idle=False)
    cold_cutoff = moment(record["cold_observed_at"], "cold cutoff")
    cold_candidates = [index for index, item in enumerate(samples)
                       if item["stable"] and moment(item["finished_at"], "cold sample end") <= cold_cutoff]
    require(cold_candidates and record["cold_sample"] == cold_candidates[-1], "cold must use last stable sample before declaration")
    cold = samples[integer(record["cold_sample"], "cold sample")]
    resources(cold, idle=True)
    cases = record["cases"]
    require(isinstance(cases, list) and len(cases) == 20, "ten HTML/Office pairs required")
    used = set()
    uris = {}
    previous_close = 0
    previous_ended = cold_cutoff
    for index, case in enumerate(cases):
        fields(case, CASE, "case")
        expected_kind = "html" if index % 2 == 0 else "office"
        expected_format = "html" if expected_kind == "html" else fmt
        require(case["cycle"] == index // 2 + 1 and type(case["cycle"]) is int
                and case["kind"] == expected_kind and case["format"] == expected_format, "cycle order/format invalid")
        require(case["input_sha256"] == (html_sha if expected_kind == "html" else input_sha), "case input differs")
        rendered_input(case["render_observation"], expected_format, case["input_sha256"])
        adapter = case["adapter_instance_id"]
        require(expected_kind == "html" or adapter is None, "Office must not invent an HTML adapter")
        if expected_kind == "html":
            integer(adapter, "HTML adapter instance", minimum=1)
        uri = sha(case["uri_sha256"], "case URI")
        require(expected_kind not in uris or uris[expected_kind] == uri, "repeated input URI identity changed")
        uris[expected_kind] = uri
        identity = (uri, adapter, integer(case["generation"], "generation", minimum=1))
        require(identity not in used, "old generation reused")
        used.add(identity)
        open_seq = integer(case["open_seq"], "open seq", minimum=1)
        close_seq = integer(case["close_seq"], "close seq", minimum=1)
        require(previous_close < open_seq < close_seq, "generation not opened/closed in order")
        previous_close = close_seq
        for field in ("open_receipt_sha256", "close_receipt_sha256"):
            require(sha(case[field], field) in record["raw_artifacts"], "receipt absent from raw inventory")
        started = moment(case["open_started_at"], "case started")
        ready = moment(case["ready_at"], "case ready")
        closing = moment(case["close_started_at"], "case close started")
        closed = moment(case["closed_at"], "case closed")
        require(previous_ended <= started <= ready <= closing <= closed, "case order/timestamps invalid")
        previous_ended = closed
        ready_ms = number(case["ready_ms"], "first frame")
        require(ready_ms > 0, "first frame must retain the positive legacy timing contract")
        close_ms = number(case["close_ms"], "close")
        require(abs((ready-started).total_seconds()*1000-ready_ms) < 2, "first-frame duration differs from raw timestamps")
        require(abs((closed-closing).total_seconds()*1000-close_ms) < 2 and close_ms <= 5000, "normal close exceeds 5000ms")
        if ready_ms > (60000 if expected_kind == "html" else 15000):
            require(expected_format == "pptx", "unapproved first-frame budget exception")
            verify_pptx_first_frame_disposition(case["first_frame_disposition"], ready_ms)
        else:
            require(case["first_frame_disposition"] is None, "unneeded first-frame disposition")
        candidates = [sample_index for sample_index, sample in enumerate(samples)
                      if sample["receipt_seq"] == close_seq and sample["stable"]
                      and moment(sample["observed_at"], "close sample start") >= closed]
        require(candidates and case["selected_sample"] == candidates[0], "close must use first stable post-close sample, never minimum")
        snapshot = samples[integer(case["selected_sample"], "selected close sample")]
        require(snapshot["receipt_sha256"] == case["close_receipt_sha256"], "close sample differs from close receipt")
        resources(snapshot, idle=True)
        next_start = moment(cases[index+1]["open_started_at"], "next open") if index+1 < len(cases) else moment(record["quit_started_at"], "quit")
        require(moment(snapshot["finished_at"], "close sample finished") < next_start, "sample occurs after next action")
        if expected_kind == "office":
            require(any(sample["children"] and started <= moment(sample["observed_at"], "worker observed") <= closing
                        for sample in samples), "actual Office sidecar was not observed in this case; short-lived/missing is not zero")
    warm, final = (samples[cases[index]["selected_sample"]] for index in (1, -1))
    for field in ("rss_bytes", "physical_footprint_bytes"):
        require(warm[field] - cold[field] <= 196608*1024, f"{field}: cold budget exceeds 196608KiB")
        require(final[field] - warm[field] <= 65536*1024, f"{field}: steady budget exceeds 65536KiB")
    require(type(record["exit_code"]) is int and record["exit_code"] == 0, "app did not exit normally")
    quit_time, exit_time = moment(record["quit_started_at"], "quit"), moment(record["exit_observed_at"], "exit")
    require(previous_ended <= quit_time <= exit_time and (exit_time-quit_time).total_seconds()*1000 <= 5000,
            "normal app exit exceeds 5000ms")
    terminal = fields(record["post_exit_processes"], {"observed_at", "main_alive", "children"}, "actual after-exit process snapshot")
    require(terminal["main_alive"] is False and terminal["children"] == []
            and moment(terminal["observed_at"], "after-exit process snapshot") >= exit_time,
            "owned processes were not observed released after normal exit")
    from document_host_heartbeat import verify as verify_heartbeat
    verify_heartbeat(record["heartbeat_observations"], cases)
    operations = record["operations"]
    require(isinstance(operations, list), "actual host operations required")
    for item in operations:
        operation(item, record, used)
        require(item["raw_sha256"] in record["raw_artifacts"], "operation absent from raw inventory")
    return record
