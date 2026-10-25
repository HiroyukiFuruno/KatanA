#!/usr/bin/env python3
"""Discoverable parser/config regressions; no UI, Docker, or process mocks."""

import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import time
import tempfile
import unittest


ROOT = Path(__file__).resolve().parent
PARSER_PATH = ROOT / "docker-ui-document-log-parser.py"
SPEC = importlib.util.spec_from_file_location("document_log_parser", PARSER_PATH)
PARSER = importlib.util.module_from_spec(SPEC)
assert SPEC and SPEC.loader
sys.modules[SPEC.name] = PARSER
SPEC.loader.exec_module(PARSER)
OBSERVER_PATH = ROOT / "docker-ui-document-observer.py"
OBSERVER_SPEC = importlib.util.spec_from_file_location("document_observer", OBSERVER_PATH)
OBSERVER = importlib.util.module_from_spec(OBSERVER_SPEC)
assert OBSERVER_SPEC and OBSERVER_SPEC.loader
OBSERVER_SPEC.loader.exec_module(OBSERVER)

URI = "file:///fixture/only.xlsx"
UTF8_URI = "file:///fixture/%E8%A1%A8.xlsx"


def event(name: str, **fields: str) -> str:
    return "[KATANA_DEBUG] event=" + name + " " + " ".join(
        f"{key}={value}" for key, value in fields.items()
    )


class DocumentLogParserTests(unittest.TestCase):
    def test_generation_is_an_exact_token_not_a_prefix(self):
        line = event("document_frame_received", generation="10", format="xlsx", surface="Grid")
        self.assertFalse(PARSER.matches(line, "document_frame_received", generation="1", format="xlsx"))
        self.assertTrue(PARSER.matches(line, "document_frame_received", generation="10", format="xlsx"))

    def test_fields_are_parsed_from_real_whitespace_tokens(self):
        line = event("document_worker_handoff", generation="1", format="xlsx", uri=URI)
        self.assertEqual(PARSER.parse(line).fields["generation"], "1")
        self.assertEqual(PARSER.generation(line), "1")

    def test_startup_events_at_fresh_log_offset_zero_are_not_discarded(self):
        lines = [
            event("document_source_intake", format="xlsx", uri=UTF8_URI),
            event("document_worker_handoff", format="xlsx", uri=UTF8_URI),
            event("document_frame_received", generation="1", format="xlsx", surface="Grid"),
        ]
        self.assertEqual(PARSER.first_after(lines, 0, "document_source_intake", format="xlsx", uri=UTF8_URI), 0)
        self.assertEqual(PARSER.first_after(lines, 1, "document_worker_handoff", format="xlsx", uri=UTF8_URI), 1)

    def test_canonical_utf8_uri_must_match_exactly(self):
        line = event("document_worker_handoff", generation="1", format="xlsx", uri=UTF8_URI)
        self.assertTrue(PARSER.matches(line, "document_worker_handoff", uri=UTF8_URI))
        self.assertFalse(PARSER.matches(line, "document_worker_handoff", uri="file:///fixture/表.xlsx"))

    def test_close_sequence_ignores_old_close_and_requires_drop_then_session_uri(self):
        lines = [
            event("document_session_closed", generation="1", uri=URI),
            event("document_surface_drop", generation="1", uri=URI),
            event("document_surface_drop", generation="1", uri=URI),
            event("document_session_closed", generation="1", uri="file:///fixture/other.xlsx"),
            event("document_session_closed", generation="1", uri=URI),
            event("document_surface_drop", generation="1", uri=URI),
        ]
        self.assertEqual(PARSER.close_sequence(lines, offset=2, generation="1", uri=URI), (2, 4))
        self.assertIsNone(PARSER.close_sequence(lines, offset=2, generation="2", uri=URI))

    def test_xlsx_grid_never_requires_page_texture_log(self):
        self.assertFalse(PARSER.requires_texture("xlsx", "Grid"))
        self.assertFalse(PARSER.requires_texture("xlsx", "Page"))
        self.assertTrue(PARSER.requires_texture("pptx", "Page"))

    def test_prepared_config_uses_fixture_path_and_rejects_multiple_tabs(self):
        settings = {"terms_accepted_version": "0.22.42", "workspace": {
            "last_workspace": "/fixture", "open_tabs": ["/fixture/only.xlsx"],
            "active_tab_idx": 0, "restore_session": True,
        }}
        self.assertEqual(PARSER.validate_prepared_config(settings, "/fixture/only.xlsx"), "0.22.42")
        settings["workspace"]["open_tabs"].append("/fixture/other.xlsx")
        with self.assertRaises(ValueError):
            PARSER.validate_prepared_config(settings, "/fixture/only.xlsx")

    def test_terms_field_is_required_only_for_prepared_input(self):
        with self.assertRaises(ValueError):
            PARSER.terms_version({"workspace": {}})
        self.assertEqual(PARSER.terms_version({"terms_accepted_version": "0.22.42"}), "0.22.42")

    def test_json_report_round_trip_has_no_literal_backslash_n(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "result.json"
            path.write_text('{"steps": [{"event": "partial-result"}]}\n', encoding="utf-8")
            self.assertEqual(json.loads(path.read_text(encoding="utf-8"))["steps"][0]["event"], "partial-result")

    def test_fresh_config_helper_round_trips_portable_fixture_filename(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            settings = root / "settings.json"
            workspace = root / "workspace.json"
            source = root / "日本語'sample file.xlsx"
            source.write_bytes(b"fixture")
            document = "/fixture/" + source.name
            result = subprocess.run(
                [sys.executable, str(PARSER_PATH), "--write-fresh-config", str(settings), str(workspace),
                 "--document", document],
                check=False, capture_output=True, text=True,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            saved = json.loads(settings.read_text(encoding="utf-8"))
            self.assertEqual(saved["workspace"]["open_tabs"], [document])
            self.assertEqual(saved["workspace"]["last_workspace"], "/fixture")
            self.assertNotIn("terms_accepted_version", saved)

    def test_fresh_config_helper_escapes_nonportable_document_string(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            settings = root / "settings.json"
            workspace = root / "workspace.json"
            document = '/fixture/quote"slash\\日本語.xlsx'
            result = subprocess.run(
                [sys.executable, str(PARSER_PATH), "--write-fresh-config", str(settings), str(workspace),
                 "--document", document],
                check=False, capture_output=True, text=True,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            saved = json.loads(settings.read_text(encoding="utf-8"))
            self.assertEqual(saved["workspace"]["open_tabs"], [document])
            self.assertNotIn("terms_accepted_version", saved)

    def test_resource_receipt_requires_pair_and_schema(self):
        receipt = {
            "schema": "katana-completed-frame-resource-receipt-v1",
            "run_id": "run-1",
            "pid": 42,
            "seq": 3,
            "host": {key: 0 for key in ("previews", "html_surfaces", "document_surfaces", "frames", "textures", "cache_entries")},
            "document_workers_live": 0,
            "kdv": {key: 0 for key in ("live_document_sessions", "live_spreadsheet_workers", "live_worker_workspaces", "retained_artifact_bytes", "cached_page_count", "cached_page_bytes", "cached_spreadsheet_cell_count", "cached_spreadsheet_cell_bytes")},
            "document_surfaces": [],
        }
        self.assertTrue(PARSER.receipt_resources_are_zero(receipt))
        receipt["document_surfaces"] = [{"uri": "file:///fixture/only.xlsx", "generation": 7}]
        self.assertTrue(PARSER.receipt_has_document(receipt, "file:///fixture/only.xlsx", 7))
        receipt["document_workers_live"] = False
        self.assertFalse(PARSER.receipt_resources_are_zero(receipt))
        receipt["run_id"] = "stale-run"
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "latest.json"
            path.write_text(json.dumps(receipt) + "\n", encoding="utf-8")
            with self.assertRaises(ValueError):
                PARSER.read_resource_receipt(path, "run-1", 42)

    def test_receipt_verifier_rejects_sequence_rollback(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            receipt_dir = root / "receipt"
            receipt_dir.mkdir()
            payload = {
                "schema": "katana-completed-frame-resource-receipt-v1",
                "run_id": "run-1", "pid": 42, "seq": 2,
                "host": {}, "document_workers_live": 0, "kdv": {}, "document_surfaces": [],
            }
            (receipt_dir / "latest.json").write_text(json.dumps(payload), encoding="utf-8")
            verifier = OBSERVER.ReceiptVerifier(receipt_dir, "run-1", 42, PARSER)
            verifier.seq = 3
            with self.assertRaises(AssertionError):
                verifier.wait("rollback", time.monotonic() + 0.1, lambda _: True)

    def test_receipt_verifier_times_out_on_stale_sequence(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            receipt_dir = root / "receipt"
            receipt_dir.mkdir()
            payload = {
                "schema": "katana-completed-frame-resource-receipt-v1",
                "run_id": "run-1", "pid": 42, "seq": 2,
                "host": {}, "document_workers_live": 0, "kdv": {}, "document_surfaces": [],
            }
            (receipt_dir / "latest.json").write_text(json.dumps(payload), encoding="utf-8")
            verifier = OBSERVER.ReceiptVerifier(receipt_dir, "run-1", 42, PARSER)
            verifier.seq = 2
            with self.assertRaisesRegex(AssertionError, "Timed out"):
                verifier.wait("stale", time.monotonic() + 0.1, lambda _: True)


if __name__ == "__main__":
    unittest.main()
