#!/usr/bin/env python3
"""Synthetic unit tests for v47 host evidence boundaries."""

import copy
import json
import unittest
import tempfile
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from document_host_io import HostEvidenceError, write_new
from document_host_acceptance import APPROVAL, MANIFEST, PUBLISHED_V47_EVIDENCE_RELATIVE, PUBLISHED_EVIDENCE_RELATIVE, verify, verify_v47_timing
from document_host_boundaries import validate_case
from document_host_io import digest, reference
from v047_evidence_paths import HOST_BUNDLE, HOST_RUNS
from generated_asset_identity import module


class HostV47BatchTests(unittest.TestCase):
    def test_v47_reference_accepts_only_the_exact_expected_public_file(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            record = write_new(root / HOST_BUNDLE, {"schema": "synthetic"})
            record["path"] = HOST_BUNDLE
            self.assertEqual(reference(root, record, "bundle", allowed_paths=frozenset({HOST_BUNDLE}))[1],
                             {"schema": "synthetic"})
            with self.assertRaises(HostEvidenceError):
                reference(root, record, "legacy")
            for invalid in (HOST_RUNS[0], HOST_BUNDLE.replace("v0.22.47", "v0.22.46"), HOST_BUNDLE + ".extra"):
                changed = dict(record, path=invalid)
                with self.subTest(path=invalid), self.assertRaises(HostEvidenceError):
                    reference(root, changed, "bundle", allowed_paths=frozenset({HOST_BUNDLE}))

    def test_failed_final_validation_does_not_create_canonical_record(self):
        producer = module("_unit_v47_output_producer", "produce-document-host-evidence-v47.py")
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            approval = root / APPROVAL
            approval.parent.mkdir(parents=True)
            approval.write_text("synthetic-only")
            bundle = {"target": "macos-arm64", "main_path": "/unit-main", "worker_path": "/unit-worker",
                      "main_sha256": "a"*64, "sidecar_sha256": "a"*64, "archive_sha256": "b"*64, "provenance": {}}
            helper = SimpleNamespace(legacy=lambda: SimpleNamespace(SUPPORTED_TARGETS={"macos-arm64"}),
                                     input_digest=lambda _: "a"*64)
            with patch.object(producer, "module", return_value=helper), patch.object(producer, "provenance"), \
                 patch.object(producer, "verify_payload", side_effect=HostEvidenceError("invalid")):
                with self.assertRaisesRegex(HostEvidenceError, "invalid"):
                    producer.write_evidence(root, root/"openspec/evidence/unit", {}, {"upstream_assessments": {}},
                                            bundle, {}, "a"*64, [])
            self.assertFalse((root / PUBLISHED_V47_EVIDENCE_RELATIVE).exists())

    def test_html_epoch_change_requires_matching_observed_reload(self):
        label, run_id, pid = "test-cycle01-html", "unit", 123
        uri = "file:///unit-only.html"
        def raw(seq, epoch=None):
            html = [] if epoch is None else [{"source_uri": uri, "katana_renderer_session_epoch": epoch,
                                              "katana_adapter_instance_id": 3}]
            return json.dumps({"schema": "katana-completed-frame-resource-receipt-v1", "pid": pid, "run_id": run_id,
                               "seq": seq, "document_surfaces": [], "html_observations": html}).encode()
        before, opened, closing = raw(1), raw(2, 4), raw(3, 5)
        start = {"pid": pid, "start_identity": "start", "input": "/unit-only.html", "input_sha256": "a"*64,
                 "kind": "html", "start_seq": 1, "receipt_sha256": digest(before), "observedAt": "start"}
        close = {"seq": 3, "receipt_sha256": digest(closing), "observedAt": "close"}
        values = {label+".start.json": start, label+".close-start.json": close,
                  label+"-start.receipt.json": before, label+"-open.receipt.json": opened, label+"-close-start.receipt.json": closing}
        run = type("UnitRun", (), {"json": lambda _, key: values[key], "raw": lambda _, key: values[key]})()
        case = {"input_sha256": "a"*64, "kind": "html", "open_seq": 2, "close_seq": 4,
                "open_started_at": "start", "close_started_at": "close", "uri_sha256": digest(uri.encode()),
                "generation": 4, "adapter_instance_id": 3}
        transcript = {"pid": pid, "run_id": run_id, "start_identity": "start", "operations": []}
        with self.assertRaisesRegex(HostEvidenceError, "without an observed reload"):
            validate_case(run, label, case, transcript)
        transcript["operations"].append({"name": "reload", "kind": "html", "uri_sha256": case["uri_sha256"],
                                         "generation": 4, "adapter_instance_id": 3, "seq_after": 3,
                                         "before": {"session": 4}, "after": {"session": 5}})
        validate_case(run, label, case, transcript)

    def test_open_and_close_boundaries_require_actual_closed_then_same_open_surface(self):
        label, run_id, pid = "test-cycle01-office", "unit", 123
        surface = {"uri": "file:///unit-only.pptx", "generation": 9}
        def raw(seq, surfaces):
            return json.dumps({"schema": "katana-completed-frame-resource-receipt-v1", "pid": pid, "run_id": run_id,
                               "seq": seq, "document_surfaces": surfaces, "html_observations": []}).encode()
        before, opened, closing = raw(1, []), raw(2, [surface]), raw(3, [surface])
        start = {"pid": pid, "start_identity": "start", "input": "/unit-only.pptx", "input_sha256": "a"*64,
                 "kind": "office", "start_seq": 1, "receipt_sha256": digest(before), "observedAt": "start"}
        close = {"seq": 3, "receipt_sha256": digest(closing), "observedAt": "close"}
        values = {label+".start.json": start, label+".close-start.json": close,
                  label+"-start.receipt.json": before, label+"-open.receipt.json": opened, label+"-close-start.receipt.json": closing}
        run = type("UnitRun", (), {"json": lambda _, key: values[key], "raw": lambda _, key: values[key]})()
        case = {"input_sha256": "a"*64, "kind": "office", "open_seq": 2, "close_seq": 4,
                "open_started_at": "start", "close_started_at": "close", "uri_sha256": digest(surface["uri"].encode()),
                "generation": 9, "adapter_instance_id": None}
        transcript = {"pid": pid, "run_id": run_id, "start_identity": "start", "operations": []}
        validate_case(run, label, case, transcript)
        for name, invalid in (("-start.receipt.json", raw(1, [surface])), ("-close-start.receipt.json", raw(3, []))):
            changed = dict(values)
            changed[label+name] = invalid
            meta = copy.deepcopy(start if "start.receipt" in name and "close-start" not in name else close)
            meta["receipt_sha256"] = digest(invalid)
            changed[label+(".start.json" if "close-start" not in name else ".close-start.json")] = meta
            bad = type("UnitRun", (), {"json": lambda _, key: changed[key], "raw": lambda _, key: changed[key]})()
            with self.subTest(name=name), self.assertRaises(HostEvidenceError):
                validate_case(bad, label, case, transcript)

    def test_v47_rejects_old_pptx_disposition_above_limit(self):
        verify_v47_timing({"cases": [{"format": "pptx", "ready_ms": 15000}]})
        with self.assertRaisesRegex(HostEvidenceError, "exceeds 15000ms"):
            verify_v47_timing({"cases": [{"format": "pptx", "ready_ms": 15001,
                                        "first_frame_disposition": {"release_target": "v0.22.42"}}]})

    def test_evidence_version_cannot_be_relabelled_between_v42_and_v47(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for path, expected, wrong in ((PUBLISHED_V47_EVIDENCE_RELATIVE, "v0.22.47", "v0.22.42"),
                                          (PUBLISHED_EVIDENCE_RELATIVE, "v0.22.42", "v0.22.47")):
                record = {key: None for key in MANIFEST}
                record.update(schema="katana-host-acceptance-v1", version=wrong, scope="katana-host",
                              policy="2026-10-24-katana-host-advance-publication")
                write_new(root / path, record)
                with self.assertRaisesRegex(HostEvidenceError, "host version or approval scope differs"):
                    verify(root, root / path, phase="postpublication", publication_version=expected)

    def test_seven_unique_inputs_runs_processes_and_operations_are_required(self):
        producer = module("_unit_v47_producer", "produce-document-host-evidence-v47.py")
        fixtures = {str(index): "pptx" for index in range(7)}
        records = [{"input_sha256": key, "run_id": key, "pid": index + 1,
                    "start_identity": key, "cases": [{"format": "pptx", "ready_ms": 15000}],
                    "operations": [{"kind": "office", "name": "slide"}]}
                   for index, key in enumerate(fixtures)]
        records[0]["operations"] += [{"kind": "html", "name": name}
                                     for name in ("scroll", "reload", "dirty_source")]
        producer.validate_batch(records, fixtures)
        for change in (lambda values: values[1].update(run_id="0"),
                       lambda values: values[1].update(input_sha256="0"),
                       lambda values: values[0].update(operations=[]),
                       lambda values: values[0]["cases"][0].update(ready_ms=15001)):
            changed = copy.deepcopy(records)
            change(changed)
            with self.subTest(change=change), self.assertRaises(HostEvidenceError):
                producer.validate_batch(changed, fixtures)


if __name__ == "__main__":
    unittest.main()
