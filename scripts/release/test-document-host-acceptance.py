#!/usr/bin/env python3
"""合成UT fixture専用。ここで生成した値を実host受入へ使用しない。"""

from __future__ import annotations

import copy
from datetime import datetime, timedelta, timezone
import hashlib
import json
import tempfile
import unittest
from pathlib import Path

from document_host_io import HostEvidenceError, decode, digest, reference
from document_host_transcript import HOST, KDV, verify
from document_host_acceptance import assessments, same_source_content
from document_host_publication import module


def test_sha(label: str) -> str:
    return hashlib.sha256(("SYNTHETIC-UNIT-TEST-ONLY:"+label).encode()).hexdigest()


def source_state() -> dict:
    return {"preview_input_fnv1a64": "0123456789abcdef", "last_imported_disk_fnv1a64": "0123456789abcdef",
            "document_revision": 1, "saved_revision": 1, "dirty": False, "is_reference": False}


def fixture(fmt: str = "docx") -> dict:
    base = datetime(2026, 10, 25, tzinfo=timezone.utc)
    stamp = lambda ms: (base+timedelta(milliseconds=ms)).isoformat()
    pid, start, main, worker, input_sha, html_sha = 123, "456", test_sha("main"), test_sha("worker"), test_sha(fmt), test_sha("html")
    source = {"source_commit": "a"*40, "source_tree_sha256": test_sha("source"),
              "cargo_lock_sha256": test_sha("lock"), "published_dependencies": {}}
    samples = []

    def sample(seq, at, receipt_sha):
        return {"receipt_seq": seq, "receipt_sha256": receipt_sha, "stable": True, "observed_at": stamp(at),
                "finished_at": stamp(at+1), "pid": pid, "start_identity": start, "rss_bytes": 1000000,
                "physical_footprint_bytes": 2000000, "host": dict.fromkeys(HOST, 0), "kdv": dict.fromkeys(KDV, 0), "workers": 0,
                "main_cpu_percent": 1.0, "kernel_observed_at": stamp(at), "children": []}

    samples.append(sample(1, 0, test_sha("cold-receipt")))
    cases = []
    raw_digests = set()
    for index in range(20):
        kind = "html" if index % 2 == 0 else "office"
        at = 100 + index*200
        open_sha, close_sha = test_sha(f"open{index}"), test_sha(f"close{index}")
        raw_digests.update((open_sha, close_sha))
        if kind == "office":
            active = sample(index*10+2, at+60, open_sha)
            active["children"] = [{"pid": 500+index, "ppid": pid, "start_identity": str(1000+index), "role": "office_sidecar",
                                   "member": "KatanA Desktop.app/Contents/MacOS/kdv-office-worker", "executable_sha256": worker,
                                   "executable_path_sha256": test_sha("private-path"), "rss_bytes": 100000, "physical_footprint_bytes": 90000,
                                   "cpu_percent": 1.0, "observed_at": stamp(at+60)}]
            active["host"]["frames"] = 1
            active["workers"] = 1
            samples.append(active)
        samples.append(sample(index*10+9, at+120, close_sha))
        rendered = {"kind": "html", "loaded_source": source_state(), "input_fnv1a64": "0123456789abcdef",
                    "frame_generation": 1, "viewport_width": 1280, "viewport_height": 900, "content_height": 2000, "scroll_y": 0} if kind == "html" else {
                    "kind": "office", "source_revision": "sha256:"+input_sha, "idle": True,
                    "frame": {"format": fmt, "active_index": 0, "item_count": 2, "surface_kind": "Grid" if fmt == "xlsx" else "Page"}}
        cases.append({"cycle": index//2+1, "kind": kind, "format": "html" if kind == "html" else fmt,
                      "input_sha256": html_sha if kind == "html" else input_sha,
                      "uri_sha256": test_sha(kind+"URI"), "generation": 1 if kind == "html" else index//2+1,
                      "adapter_instance_id": index//2+1 if kind == "html" else None,
                      "open_seq": index*10+2, "close_seq": index*10+9,
                      "open_started_at": stamp(at), "ready_at": stamp(at+50), "ready_ms": 50,
                      "close_started_at": stamp(at+100), "closed_at": stamp(at+110), "close_ms": 10,
                      "open_receipt_sha256": open_sha, "close_receipt_sha256": close_sha,
                      "selected_sample": len(samples)-1, "first_frame_disposition": None, "render_observation": rendered})
    heartbeats = []
    for index in range(44):
        observation = {"status": "observed", "observed_at": stamp(index*100), "monotonic_ns": 1_000_000_000+index*100_000_000,
                       "frame": index+1, "mtime_ns": 1_000_000_000+index*100_000_000,
                       "sha256": hashlib.sha256(f"first-frame\nframe={index+1}\n".encode()).hexdigest()}
        heartbeats.append(dict(observation, attempts=[dict(observation)]))
    return {"schema": "katana-host-transcript-v1", "provenance": source, "run_id": "synthetic-unit-fixture-only",
            "target": "macos-arm64", "pid": pid, "start_identity": start, "main_sha256": main, "sidecar_sha256": worker,
            "input_sha256": input_sha, "html_sha256": html_sha, "cold_observed_at": stamp(10), "cold_sample": 0,
            "samples": samples, "cases": cases, "operations": [], "quit_started_at": stamp(4200),
            "exit_observed_at": stamp(4300), "exit_code": 0, "raw_artifacts": sorted(raw_digests),
            "settings": {"debug": True, "readonly_recorder": True, "malloc_inspector": False, "vmmap_inspector": False,
                         "heap_inspector": False, "isolation": "isolated_profile", "clean_machine": False,
                         "protections_disabled": False, "personal_profile_changed": False, "terms_scope": "this_isolated_run_only"},
            "producer_sha256": test_sha("producer"),
            "post_exit_processes": {"observed_at": stamp(4300), "main_alive": False, "children": []},
            "heartbeat_observations": heartbeats,
            "launch": {"method": "explicit_macos_arch", "target": "macos-arm64", "requested_slice": "arm64",
                       "observer_machine": "arm64", "argv_sha256": test_sha("launch")}}


def warm_sample(item: dict) -> dict:
    return item["samples"][item["cases"][1]["selected_sample"]]


def validate(value: dict, fmt: str = "docx"):
    return verify(value, fixtures={test_sha(fmt): fmt}, html_sha=test_sha("html"), provenance=fixture()["provenance"],
                  main_sha=test_sha("main"), worker_sha=test_sha("worker"), producer_sha=test_sha("producer"))


class HostTranscriptBoundaryTests(unittest.TestCase):
    def operation_fixture(self, name, kind="html"):
        item = fixture()
        case = item["cases"][0 if kind == "html" else 1]
        raw_sha = test_sha("actual-operation-packet-unit-only")
        before = after = None
        if name == "scroll":
            before = {"scroll_y": 0, "frame_generation": 1, "session": 1,
                      "saved_input_sha256": case["input_sha256"], "loaded_source": source_state()}
            after = dict(before, scroll_y=100, frame_generation=2)
        elif name == "reload":
            before = {"session": 1, "frame_session": 1, "frame_generation": 1,
                      "saved_input_sha256": case["input_sha256"], "loaded_source": source_state()}
            after = dict(before, session=2, frame_session=2)
        elif name == "dirty_source":
            state = dict(source_state(), document_revision=2, dirty=True, preview_input_fnv1a64="fedcba9876543210")
            before = {"loaded_source": state, "saved_input_sha256": case["input_sha256"]}
            after = copy.deepcopy(before)
        else:
            before = {"source_revision": "sha256:"+case["input_sha256"], "idle": True,
                      "frame": {"format": "docx", "active_index": 0, "item_count": 1 if name == "single_item" else 2, "surface_kind": "Page"},
                      "reason": "single_item" if name == "single_item" else None}
            after = copy.deepcopy(before)
            if name != "single_item":
                after["frame"]["active_index"] = 1
        operation = {"name": name, "kind": kind, "input_sha256": case["input_sha256"], "pid": item["pid"], "run_id": item["run_id"],
                     "start_identity": item["start_identity"], "uri_sha256": case["uri_sha256"], "generation": case["generation"],
                     "adapter_instance_id": case["adapter_instance_id"], "seq_before": case["open_seq"],
                     "seq_after": case["open_seq"]+(0 if name == "single_item" else 1),
                     "started_at": case["ready_at"], "completed_at": case["close_started_at"], "before": before, "after": after, "raw_sha256": raw_sha}
        item["operations"] = [operation]
        item["raw_artifacts"].append(raw_sha)
        validate(item)
        return item

    def reject(self, mutation, fmt="docx"):
        value = fixture(fmt)
        mutation(value)
        with self.assertRaises((HostEvidenceError, ValueError, KeyError, IndexError)):
            validate(value, fmt)

    def test_complete_synthetic_fixture_is_a_unit_test_only(self):
        validate(fixture())

    def test_every_required_top_field_missing_is_rejected(self):
        for key in fixture():
            with self.subTest(key=key):
                self.reject(lambda item: item.pop(key))

    def test_every_case_field_missing_is_rejected(self):
        for key in fixture()["cases"][0]:
            with self.subTest(key=key):
                self.reject(lambda item: item["cases"][0].pop(key))

    def test_unknown_public_fields_reject_private_text(self):
        self.reject(lambda item: item.update(private_original_name="PRIVATE-NAME"))
        self.reject(lambda item: item["cases"][0].update(uri="file:///private/original.html"))

    def test_source_registry_lock_main_worker_and_input_each_bound(self):
        for field in ("provenance", "main_sha256", "sidecar_sha256", "input_sha256", "html_sha256", "producer_sha256"):
            with self.subTest(field=field):
                self.reject(lambda item: item.__setitem__(field, test_sha("other")))

    def test_mac_measurement_cannot_be_reused_for_linux(self):
        self.reject(lambda item: item.update(target="linux-x86_64"))

    def test_pid_start_identity_and_run_reuse_do_not_hide(self):
        self.reject(lambda item: item["samples"][3].update(pid=124))
        self.reject(lambda item: item["samples"][3].update(start_identity="457"))
        self.reject(lambda item: item.update(run_id="../private"))

    def test_nine_cycles_and_pair_aliasing_are_rejected(self):
        self.reject(lambda item: item["cases"].__delitem__(slice(-2, None)))
        self.reject(lambda item: item["cases"][2].update(adapter_instance_id=1))
        self.reject(lambda item: item["cases"][1].update(kind="html"))

    def test_nonzero_resources_independently_rejected(self):
        for group, counters in (("host", HOST), ("kdv", KDV)):
            for counter in counters:
                with self.subTest(group=group, counter=counter):
                    self.reject(lambda item: warm_sample(item)[group].__setitem__(counter, 1))
        self.reject(lambda item: warm_sample(item).update(workers=1))

    def test_all_four_memory_boundaries_accept_exact_limit(self):
        for field in ("rss_bytes", "physical_footprint_bytes"):
            item = fixture()
            baseline = item["samples"][0][field]
            warm_sample(item)[field] = baseline+196608*1024
            item["samples"][-1][field] = warm_sample(item)[field]+65536*1024
            validate(item)

    def test_cold_rss_or_physical_plus_one_byte_rejected(self):
        for field in ("rss_bytes", "physical_footprint_bytes"):
            with self.subTest(field=field):
                self.reject(lambda item: warm_sample(item).__setitem__(field, item["samples"][0][field]+196608*1024+1))

    def test_steady_rss_or_physical_plus_one_byte_rejected(self):
        for field in ("rss_bytes", "physical_footprint_bytes"):
            with self.subTest(field=field):
                self.reject(lambda item: item["samples"][-1].__setitem__(field, warm_sample(item)[field]+65536*1024+1))

    def test_missing_memory_does_not_become_zero(self):
        self.reject(lambda item: item["samples"][0].update(rss_bytes=None))
        self.reject(lambda item: item["samples"][0].update(physical_footprint_bytes=0))
        self.reject(lambda item: item["samples"][0].update(rss_bytes=True))

    def test_minimum_sample_selection_and_time_alias_rejected(self):
        self.reject(lambda item: item["cases"][0].update(selected_sample=2))
        self.reject(lambda item: item["samples"][1].update(finished_at=item["cases"][1]["open_started_at"]))
        self.reject(lambda item: item["samples"][2].update(observed_at=item["samples"][1]["observed_at"]))

    def test_raw_close_receipt_must_match_sample(self):
        self.reject(lambda item: warm_sample(item).update(receipt_sha256=test_sha("different")))

    def test_missing_worker_observation_is_not_nonuse_or_zero(self):
        self.reject(lambda item: item["samples"][2].update(children=[]))
        self.reject(lambda item: item["samples"][2]["children"][0].update(pid=item["pid"]))
        self.reject(lambda item: item["samples"][2]["children"][0].update(executable_sha256=test_sha("foreign-worker")))
        self.reject(lambda item: warm_sample(item).update(children=item["samples"][2]["children"]))
        self.reject(lambda item: item["post_exit_processes"].update(children=item["samples"][2]["children"]))

    def test_close_and_exit_limits_are_not_exempt(self):
        self.reject(lambda item: item["cases"][0].update(close_ms=5000.001))
        self.reject(lambda item: item.update(exit_observed_at="2026-10-25T00:00:09.200001+00:00"))
        self.reject(lambda item: item.update(exit_code=1))

    def test_docx_xlsx_and_html_timing_cannot_use_pptx_exception(self):
        for fmt in ("docx", "xlsx"):
            with self.subTest(fmt=fmt):
                self.reject(lambda item: item["cases"][1].update(ready_ms=15001), fmt)
        self.reject(lambda item: item["cases"][0].update(ready_ms=60001))
        for value in (0, True, float("nan"), float("inf")):
            self.reject(lambda item: item["cases"][0].update(ready_ms=value))
        def zero_with_matching_timestamps(item):
            for case in item["cases"]:
                case["ready_ms"] = 0
                case["ready_at"] = case["open_started_at"]
        self.reject(zero_with_matching_timestamps)

    def test_every_cycle_must_bind_the_actual_loaded_input(self):
        self.reject(lambda item: item["cases"][15]["render_observation"].update(source_revision="sha256:"+"0"*64))
        self.reject(lambda item: item["cases"][14]["render_observation"]["loaded_source"].update(preview_input_fnv1a64="0"*16))
        self.reject(lambda item: item["cases"][14]["render_observation"]["loaded_source"].update(last_imported_disk_fnv1a64="0"*16))

    def test_continuous_heartbeat_is_not_a_case_end_sequence(self):
        self.reject(lambda item: item.update(heartbeat_observations=[]))
        self.reject(lambda item: item["heartbeat_observations"].__delitem__(slice(10, 14)))
        def stall(item):
            for observed in item["heartbeat_observations"][1:5]:
                observed["frame"] = 1
                observed["sha256"] = hashlib.sha256(b"first-frame\nframe=1\n").hexdigest()
        self.reject(stall)

    def test_universal_bundle_cannot_relabel_selected_launch_slice(self):
        self.reject(lambda item: item.update(target="macos-x86_64"))
        self.reject(lambda item: item["launch"].update(requested_slice="x86_64"))

    def test_html_scroll_reload_and_dirty_source_are_actual_bound_operations(self):
        for name, mutate in (("scroll", lambda value: value["after"].update(scroll_y=0)),
                             ("reload", lambda value: value["after"].update(session=1, frame_session=1)),
                             ("dirty_source", lambda value: value["after"]["loaded_source"].update(dirty=False))):
            item = self.operation_fixture(name)
            mutate(item["operations"][0])
            with self.subTest(name=name), self.assertRaises(HostEvidenceError):
                validate(item)

    def test_office_navigation_and_single_item_reasons_are_distinct(self):
        item = self.operation_fixture("page", "office")
        item["operations"][0]["after"]["frame"]["active_index"] = 0
        with self.assertRaises(HostEvidenceError):
            validate(item)
        item = self.operation_fixture("single_item", "office")
        for side in ("before", "after"):
            item["operations"][0][side]["frame"]["item_count"] = 2
        with self.assertRaises(HostEvidenceError):
            validate(item)

    def test_operation_cannot_alias_another_run_generation_or_interval(self):
        for key, replacement in (("run_id", "different-run"), ("pid", 999), ("generation", 999),
                                 ("adapter_instance_id", 999), ("seq_after", 999)):
            item = self.operation_fixture("scroll")
            item["operations"][0][key] = replacement
            with self.subTest(key=key), self.assertRaises(HostEvidenceError):
                validate(item)

    def test_debug_false_is_not_a_new_requirement_but_values_are_exact(self):
        item = fixture()
        item["settings"]["debug"] = False
        validate(item)
        for field in ("malloc_inspector", "vmmap_inspector", "heap_inspector", "protections_disabled", "personal_profile_changed", "clean_machine"):
            self.reject(lambda value: value["settings"].__setitem__(field, True))
        self.reject(lambda value: value["settings"].update(debug="false"))

    def test_source_content_and_build_commit_are_distinct(self):
        source = fixture()["provenance"]
        same = dict(source, source_commit="b"*40)
        self.assertTrue(same_source_content(source, same))
        self.assertFalse(same_source_content(source, dict(same, cargo_lock_sha256=test_sha("changed"))))

    def test_duplicate_json_fields_and_nan_rejected(self):
        for raw in (b'{"x":1,"x":2}', b'{"x":NaN}'):
            with self.assertRaises(HostEvidenceError):
                decode(raw)

    def test_public_references_must_be_read_and_rehashed(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "openspec/changes/test/evidence/test.json"
            path.parent.mkdir(parents=True)
            path.write_bytes(b'{"synthetic_unit_test_only":true}')
            ref = {"path": path.relative_to(root).as_posix(), "sha256": digest(path.read_bytes())}
            reference(root, ref, "UT")
            for changed in (dict(ref, path="tmp/private.json"), dict(ref, path="../outside.json"), dict(ref, sha256="0"*64)):
                with self.assertRaises(HostEvidenceError):
                    reference(root, changed, "UT")
            path.unlink()
            path.symlink_to(root / "missing-private.json")
            with self.assertRaises(HostEvidenceError):
                reference(root, ref, "UT")

    def test_unmeasured_upstream_cannot_be_score95_or_passed(self):
        value = {"chrome_html": {"owner": "KRR", "issue": "https://github.com/HiroyukiFuruno/katana-render-runtime/issues/106",
                                 "status": "not_evaluated", "score": None, "observation": "reference_unavailable", "measurement": None},
                 "office_independent_95": {"owner": "KDV", "issue": "https://github.com/HiroyukiFuruno/katana-document-viewer/issues/58",
                                           "status": "not_evaluated", "score": None, "observation": "independent_95_incomplete", "measurement": None}}
        assessments(Path("/not-read-for-unmeasured-assessment"), value)
        for field, replacement in (("score", 95), ("status", "passed"), ("owner", "unknown"), ("observation", "measured_difference")):
            changed = copy.deepcopy(value)
            changed["chrome_html"][field] = replacement
            with self.assertRaises(HostEvidenceError):
                assessments(Path("/not-read-for-unmeasured-assessment"), changed)



class HostProducerReloadBoundaryTests(unittest.TestCase):
    def opened_html_fixture(self, root):
        producer = module("_unit_opened_html_producer", "produce-document-host-evidence.py")
        original = root / "synthetic-unit-only.html"
        original.write_bytes(b"<html><body>SYNTHETIC-UNIT-TEST-ONLY</body></html>")
        fnv = producer.input_fnv(original)
        observed_source = dict(source_state(), preview_input_fnv1a64=fnv, last_imported_disk_fnv1a64=fnv)
        record = {"host": dict(dict.fromkeys(HOST, 0), frames=1), "document_surfaces": [],
                  "html_observations": [{"source_uri": original.as_uri(), "katana_adapter_instance_id": 7,
                                         "katana_renderer_session_epoch": 4, "katana_frame_renderer_session_epoch": 4,
                                         "frame_generation": 3, "viewport_width": 320, "viewport_height": 240,
                                         "content_height": 480, "scroll_y": 0, "idle": True,
                                         "loaded_source": observed_source}]}
        return producer, original, record

    def test_each_opened_html_case_requires_its_actual_frame_session(self):
        with tempfile.TemporaryDirectory() as temporary:
            producer, original, record = self.opened_html_fixture(Path(temporary))
            _, generation, adapter, rendered = producer.opened_identity(record, "html", original, digest(original.read_bytes()))
            self.assertEqual(generation, 4)
            self.assertEqual(adapter, 7)
            self.assertEqual(rendered["frame_generation"], 3)

    def test_opened_html_rejects_missing_zero_or_old_frame_marker_even_with_correct_source(self):
        for marker in (None, 0, 3):
            with self.subTest(raw_marker=marker), tempfile.TemporaryDirectory() as temporary:
                producer, original, record = self.opened_html_fixture(Path(temporary))
                observed = record["html_observations"][0]
                if marker is None:
                    observed.pop("katana_frame_renderer_session_epoch")
                else:
                    observed["katana_frame_renderer_session_epoch"] = marker
                self.assertTrue(observed["idle"])
                self.assertEqual(observed["loaded_source"]["preview_input_fnv1a64"], producer.input_fnv(original))
                with self.assertRaises(HostEvidenceError):
                    producer.opened_identity(record, "html", original, digest(original.read_bytes()))

    def producer_fixture(self, root):
        transcript = HostTranscriptBoundaryTests().operation_fixture("reload")
        case = transcript["cases"][0]
        uri = "file:///synthetic-unit-only-reload.html"
        case["uri_sha256"] = digest(uri.encode())
        packet = copy.deepcopy(transcript["operations"][0])
        packet.pop("raw_sha256")
        packet.update(schema="katana-host-operation-observation-v1", uri_sha256=case["uri_sha256"],
                      before_receipt="before.json", after_receipt="after.json",
                      observation_artifacts=[{"path": "unit-observation.txt", "sha256": digest(b"SYNTHETIC-UNIT-TEST-ONLY")}])
        directory = root / transcript["run_id"]
        directory.mkdir()
        (directory / "unit-observation.txt").write_bytes(b"SYNTHETIC-UNIT-TEST-ONLY")
        for side in ("before", "after"):
            observed = packet[side]
            receipt = {"schema": "katana-completed-frame-resource-receipt-v1", "pid": packet["pid"],
                       "run_id": packet["run_id"], "seq": packet["seq_"+side],
                       "host": dict.fromkeys(HOST, 0), "kdv": dict.fromkeys(KDV, 0),
                       "html_observations": [{"source_uri": uri, "katana_adapter_instance_id": packet["adapter_instance_id"],
                                              "katana_renderer_session_epoch": observed["session"],
                                              "katana_frame_renderer_session_epoch": observed["frame_session"],
                                              "frame_generation": observed["frame_generation"],
                                              "loaded_source": observed["loaded_source"]}]}
            (directory / (side+".json")).write_text(json.dumps(receipt))
        (directory / "operation.json").write_text(json.dumps(packet))
        return directory, transcript, packet

    def produce(self, directory, transcript, packet):
        producer = module("_unit_reload_producer", "produce-document-host-evidence.py")
        return producer.operation(producer.PrivateRun(directory), "operation.json", packet["pid"], packet["start_identity"],
                                  {"office": transcript["input_sha256"], "html": transcript["html_sha256"]},
                                  transcript["cases"], source_state()["last_imported_disk_fnv1a64"])

    def test_reload_accepts_only_actual_before_and_after_frame_markers(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory, transcript, packet = self.producer_fixture(Path(temporary))
            observed = self.produce(directory, transcript, packet)
            self.assertEqual(observed["before"]["frame_session"], 1)
            self.assertEqual(observed["after"]["frame_session"], 2)

    def test_reload_rejects_missing_zero_or_mismatched_marker_in_each_raw_receipt(self):
        for side in ("before", "after"):
            for marker in (None, 0, 999):
                with self.subTest(side=side, raw_marker=marker), tempfile.TemporaryDirectory() as temporary:
                    directory, transcript, packet = self.producer_fixture(Path(temporary))
                    path = directory / (side+".json")
                    receipt = json.loads(path.read_text())
                    observed = receipt["html_observations"][0]
                    if marker is None:
                        observed.pop("katana_frame_renderer_session_epoch")
                    else:
                        observed["katana_frame_renderer_session_epoch"] = marker
                    path.write_text(json.dumps(receipt))
                    with self.assertRaises(HostEvidenceError):
                        self.produce(directory, transcript, packet)

    def test_self_consistent_packet_cannot_relabel_a_retained_old_raw_frame(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory, transcript, packet = self.producer_fixture(Path(temporary))
            path = directory / "after.json"
            receipt = json.loads(path.read_text())
            receipt["html_observations"][0]["katana_frame_renderer_session_epoch"] = packet["before"]["frame_session"]
            path.write_text(json.dumps(receipt))
            self.assertEqual(packet["after"]["frame_session"], packet["after"]["session"])
            with self.assertRaises(HostEvidenceError):
                self.produce(directory, transcript, packet)


if __name__ == "__main__":
    unittest.main()
