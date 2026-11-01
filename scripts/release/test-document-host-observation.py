#!/usr/bin/env python3
"""Synthetic unit tests for the actual-observation recorder."""

import json
import tempfile
import unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from document_host_io import HostEvidenceError
from document_host_observation import receipt, stable_sample
from document_host_operation import matching_observation, operation_observation, verify_semantics
from document_host_io import digest
from generated_asset_identity import module
from document_host_transcript import HOST, KDV


def idle_resources():
    return {"host": {key: 0 for key in HOST}, "kdv": {key: 0 for key in KDV},
            "document_workers_live": 0, "kernel_processes": {"children": []}}


def write_unit_sample(run, label, identity):
    raw = (run / (label+"-closed.receipt.json")).read_bytes()
    sample = {"receipt_seq": 3, "receipt_sha256": digest(raw), "receipt_stable_during_sample": True,
              "memory": {"status": "ok", "pid": 123, "identity": identity,
                         "observed_at": (datetime.now(timezone.utc)+timedelta(seconds=1)).isoformat()}} | idle_resources()
    (run / "memory-receipt-observations.jsonl").write_text(json.dumps(sample)+"\n")


def unit_case_fixture():
    item = {"schema": "katana-completed-frame-resource-receipt-v1", "pid": 123,
            "run_id": "unit", "seq": 1, "document_surfaces": [], "html_observations": []}
    fixtures = SimpleNamespace(SUPPLIED_OFFICE_FIXTURES={"a"*64: "pptx"}, ORIGINAL_HTML_SHA256="b"*64)
    producer = SimpleNamespace(input_digest=lambda _: "a"*64, opened_identity=lambda *_: None,
                               legacy=lambda: fixtures)
    identity = {"start_identity": "start", "executable": "/synthetic-only"}
    return item, producer, identity


class ObservationTests(unittest.TestCase):
    def test_receipt_rejects_foreign_pid_run_and_nonpositive_sequence(self):
        value = {"schema": "katana-completed-frame-resource-receipt-v1", "pid": 123,
                 "run_id": "unit", "seq": 1}
        self.assertEqual(receipt(json.dumps(value).encode(), "unit", 123)["seq"], 1)
        for key, invalid in (("pid", 124), ("run_id", "other"), ("seq", 0)):
            with self.subTest(key=key), self.assertRaises(HostEvidenceError):
                receipt(json.dumps(value | {key: invalid}).encode(), "unit", 123)

    def test_sample_must_match_receipt_and_be_stable(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "observations.jsonl"
            item = {"receipt_seq": 2, "receipt_sha256": "a"*64, "receipt_stable_during_sample": True,
                    "memory": {"status": "ok", "pid": 123, "identity": {"start_identity": "start"},
                               "observed_at": "2026-01-01T00:00:01+00:00"}} | idle_resources()
            path.write_text(json.dumps(item)+"\n")
            args = (path, 2, "a"*64, 123, "start", "2026-01-01T00:00:00+00:00")
            self.assertTrue(stable_sample(*args))
            self.assertFalse(stable_sample(path, 2, "b"*64, 123, "start", args[-1]))
            self.assertFalse(stable_sample(path, 2, "a"*64, 124, "start", args[-1]))
            self.assertFalse(stable_sample(path, 2, "a"*64, 123, "start", "2026-01-01T00:00:02+00:00"))
            item["receipt_stable_during_sample"] = False
            path.write_text(json.dumps(item)+"\n")
            self.assertFalse(stable_sample(*args))
            item["receipt_stable_during_sample"] = True
            item["document_workers_live"] = 1
            path.write_text(json.dumps(item)+"\n")
            with self.assertRaises(HostEvidenceError):
                stable_sample(*args)

    def test_operation_observation_uses_actual_surface_and_rejects_foreign_identity(self):
        uri = "file:///synthetic-unit-input.html"
        item = {"source_uri": uri, "katana_adapter_instance_id": 12,
                "loaded_source": {"dirty": True}, "katana_renderer_session_epoch": 5,
                "katana_frame_renderer_session_epoch": 5, "frame_generation": 7, "scroll_y": 2}
        packet = {"html_observations": [item]}
        observed = matching_observation(packet, "html", digest(uri.encode()), 5, 12)
        self.assertEqual(operation_observation(observed, "html", "reload", "a"*64)["frame_session"], 5)
        with self.assertRaises(HostEvidenceError):
            matching_observation(packet, "html", digest(uri.encode()), 5, 13)

    def test_single_item_reason_is_only_emitted_for_that_operation(self):
        item = {"source_revision": "sha256:"+"a"*64, "idle": True,
                "frame": {"format": "pptx", "active_index": 0, "item_count": 1, "surface_kind": "Page"}}
        self.assertEqual(operation_observation(item, "office", "single_item", "a"*64)["reason"], "single_item")
        self.assertIsNone(operation_observation(item, "office", "slide", "a"*64)["reason"])

    def test_navigation_and_reload_must_change_actual_frames(self):
        before = {"idle": True, "frame": {"item_count": 2, "active_index": 0}}
        after = {"idle": True, "frame": {"item_count": 2, "active_index": 1}}
        verify_semantics("slide", before, after)
        with self.assertRaises(HostEvidenceError):
            verify_semantics("slide", before, before)
        first = {"session": 1, "frame_session": 1, "loaded_source": {"dirty": False}}
        second = {"session": 2, "frame_session": 2, "loaded_source": {"dirty": False}}
        verify_semantics("reload", first, second)
        with self.assertRaises(HostEvidenceError):
            verify_semantics("reload", first, first)

    def test_case_recorder_binds_process_sequences_and_stable_close(self):
        recorder = module("_unit_host_case_recorder", "record-document-host-observation.py")
        with tempfile.TemporaryDirectory() as temporary:
            run = Path(temporary) / "unit"
            run.mkdir()
            item, producer, identity = unit_case_fixture()
            def latest(_run, _pid):
                return json.dumps(item).encode(), dict(item)
            label = "unit-cycle01-office"
            with patch.object(recorder, "run_identity", return_value=(123, identity)), \
                 patch.object(recorder, "latest", side_effect=latest), patch.object(recorder, "module", return_value=producer):
                recorder.observe_start(run, label, run / "input.pptx", "office")
                with self.assertRaises(HostEvidenceError):
                    recorder.observe_start(run, "unit-cycle01-html", run / "input.html", "html")
                item["seq"] = 2
                item["document_surfaces"] = [{"uri": "file:///synthetic.pptx", "generation": 9}]
                recorder.observe_ready(run, label)
                with self.assertRaises(HostEvidenceError):
                    recorder.observe_start(run, "unit-cycle02-office", run / "input.pptx", "office")
                item["document_surfaces"] = []
                with self.assertRaises(HostEvidenceError):
                    recorder.observe_close_start(run, label)
                item["document_surfaces"] = [{"uri": "file:///synthetic.pptx", "generation": 9}]
                recorder.observe_close_start(run, label)
                item.update(seq=3, document_surfaces=[], html_observations=[], **idle_resources())
                recorder.observe_closed(run, label)
                with self.assertRaises(HostEvidenceError):
                    recorder.finish_case(run, label)
                write_unit_sample(run, label, identity)
                recorder.finish_case(run, label)
                result = json.loads((run / (label+".result.json")).read_text())
                self.assertEqual((result["openSeq"], result["closeSeq"]), (2, 3))

    def test_operation_recorder_uses_receipt_values_and_rejects_unchanged_navigation(self):
        recorder = module("_unit_host_op_recorder", "record-document-host-operation.py")
        with tempfile.TemporaryDirectory() as temporary:
            run = Path(temporary) / "unit"
            run.mkdir()
            label, uri, source_sha = "unit-cycle01-office", "file:///synthetic-only.pptx", "a"*64
            start = {"pid": 123, "start_identity": "start", "kind": "office", "input": "/synthetic-only.pptx", "input_sha256": source_sha}
            (run / (label+".start.json")).write_text(json.dumps(start))
            (run / (label+"-open.receipt.json")).write_text("{}")
            item = {"uri": uri, "generation": 9, "source_revision": "sha256:"+source_sha,
                    "idle": True, "frame": {"format": "pptx", "item_count": 2, "active_index": 0, "surface_kind": "Page"}}
            current = {"seq": 10, "document_observations": [item]}
            adapter = SimpleNamespace(run_identity=lambda _: (123, {"start_identity": "start"}),
                latest=lambda *_: (json.dumps(current).encode(), current), write_raw=lambda path, raw: path.write_bytes(raw),
                current_time=lambda: datetime.now(timezone.utc).isoformat(), input_digest=lambda _: source_sha,
                opened_identity=lambda *_: (digest(uri.encode()), 9, None, {}))
            with patch.object(recorder, "module", return_value=adapter):
                recorder.capture(run, label, "slide", "before")
                current["seq"] = 11
                item["frame"]["active_index"] = 1
                recorder.capture(run, label, "slide", "after")
            packet = json.loads((run / (label+"-slide.operation.json")).read_text())
            self.assertEqual((packet["seq_before"], packet["seq_after"]), (10, 11))
            self.assertEqual(packet["after"]["frame"]["active_index"], 1)


if __name__ == "__main__":
    unittest.main()
