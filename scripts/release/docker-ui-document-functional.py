#!/usr/bin/env python3
"""Observe packaged document open/close mechanics without accepting Terms."""
import importlib.util
import json
import sys
import time
from pathlib import Path


BASE_PATH = Path(__file__).with_name("docker-ui-functional.py")
BASE_SPEC = importlib.util.spec_from_file_location("docker_ui_functional", BASE_PATH)
BASE = importlib.util.module_from_spec(BASE_SPEC)
assert BASE_SPEC and BASE_SPEC.loader
sys.modules[BASE_SPEC.name] = BASE
BASE_SPEC.loader.exec_module(BASE)
PARSER_PATH = Path(__file__).with_name("docker-ui-document-log-parser.py")
PARSER_SPEC = importlib.util.spec_from_file_location("document_log_parser", PARSER_PATH)
PARSER = importlib.util.module_from_spec(PARSER_SPEC)
assert PARSER_SPEC and PARSER_SPEC.loader
sys.modules[PARSER_SPEC.name] = PARSER
PARSER_SPEC.loader.exec_module(PARSER)
OBSERVER_PATH = Path(__file__).with_name("docker-ui-document-observer.py")
OBSERVER_SPEC = importlib.util.spec_from_file_location("document_observer", OBSERVER_PATH)
OBSERVER = importlib.util.module_from_spec(OBSERVER_SPEC)
assert OBSERVER_SPEC and OBSERVER_SPEC.loader
sys.modules[OBSERVER_SPEC.name] = OBSERVER
OBSERVER_SPEC.loader.exec_module(OBSERVER)
CLI_PATH = Path(__file__).with_name("docker-ui-document-cli.py")
CLI_SPEC = importlib.util.spec_from_file_location("document_cli", CLI_PATH)
CLI = importlib.util.module_from_spec(CLI_SPEC)
assert CLI_SPEC and CLI_SPEC.loader
CLI_SPEC.loader.exec_module(CLI)
DOCUMENT_CLOSE_DEADLINE_SECONDS = 5


class TermsBlocked(RuntimeError):
    pass


class DocumentAcceptance(BASE.DesktopAcceptance):
    parser = PARSER
    base = BASE
    terms_blocked_type = TermsBlocked

    def __init__(self, evidence: Path, expected_sha256: str, document: Path, fmt: str,
                 debug_log: Path, receipt_dir: Path, receipt_run_id: str,
                 prepared_settings: Path | None):
        super().__init__(evidence, expected_sha256)
        self.expected_uri = document.resolve().as_uri()
        self.format = fmt
        self.debug_log = debug_log
        self.receipt_dir = receipt_dir
        self.receipt_run_id = receipt_run_id
        self.report.update({
            "mode": "packaged_document_debug_diagnostic",
            "terms_accepted": None,
            "terms_state": "not_yet_observed",
            "resource_counters": "receipt_observed",
            "acceptance_status": "partial_mechanics_only",
        })
        if prepared_settings is not None:
            accepted_version = PARSER.validate_prepared_config(
                json.loads(prepared_settings.read_text(encoding="utf-8")), str(document)
            )
            self.report["prepared_terms_version_before"] = accepted_version

    def debug_lines(self):
        if not self.debug_log.is_file():
            return []
        return self.debug_log.read_text(encoding="utf-8", errors="replace").splitlines()

    def debug_cursor(self) -> int:
        return len(self.debug_lines())

    def start_document(self):
        self.window_id = BASE.until("visible KatanA window", self.find_window)
        BASE.until("completed UI frame", self.frame)
        try:
            receipt_pid = int((self.root / "app.pid").read_text(encoding="utf-8"))
        except (OSError, ValueError) as error:
            raise AssertionError(f"application pid receipt is unavailable: {error}") from error
        self.provenance()
        text = self.rendered("terms-probe", lambda value: bool(value.strip()))
        if "terms" in text.lower():
            self.report["terms_state"] = "blocked_terms"
            self.record("blocked_terms", reason="existing Terms modal was visible")
            self.close_window()
            raise TermsBlocked("Terms were not accepted through the packaged UI")
        self.report["terms_state"] = "modal_not_observed"
        self.receipt = OBSERVER.ReceiptVerifier(self.receipt_dir, self.receipt_run_id, receipt_pid, PARSER)
        self.receipt.baseline()
        deadline = time.monotonic() + BASE.FUNCTIONAL_DEADLINE_SECONDS
        intake_index, intake = OBSERVER.wait_for_debug(
            self, "document source intake", 0, "document_source_intake", deadline,
            format=self.format, uri=self.expected_uri,
        )
        handoff_index, handoff = OBSERVER.wait_for_debug(
            self,
            "document worker handoff", intake_index + 1, "document_worker_handoff", deadline,
            format=self.format, uri=self.expected_uri,
        )
        generation = PARSER.generation(handoff)
        assert generation is not None, "document handoff did not include a numeric generation"
        frame_index, frame = OBSERVER.wait_for_debug(
            self,
            "document frame", handoff_index + 1, "document_frame_received", deadline,
            generation=generation, format=self.format,
        )
        frame_fields = PARSER.parse(frame).fields
        surface = frame_fields.get("surface")
        assert surface in {"Page", "Grid"}, "document frame did not describe its surface"
        texture = None
        if PARSER.requires_texture(self.format, surface):
            _, texture = OBSERVER.wait_for_debug(
                self,
                "page texture", frame_index + 1, "document_texture_updated", deadline,
            )
        heartbeat_before_render = self.frame()
        OBSERVER.wait_for_heartbeat(self, heartbeat_before_render, deadline)
        OBSERVER.capture_nonblank(self, "document-after-frame")
        OBSERVER.require_before_deadline("document frame observation", deadline)
        visible_receipt = self.receipt.wait(
            "document frame", deadline,
            lambda receipt: PARSER.receipt_has_document(receipt, self.expected_uri, int(generation)),
            require_new=False,
        )
        self.record(
            "document-visible-frame",
            document_uri=self.expected_uri,
            format=self.format,
            generation=int(generation),
            intake_log=intake,
            frame_log=frame,
            surface=surface,
            texture_log=texture,
            texture_log_binding=(
                "unbound diagnostic; not a texture-count assertion" if texture else "not-applicable-grid"
            ),
            resource_receipt_seq=visible_receipt["seq"],
        )
        return generation

    def close_document(self, generation: str):
        before = self.last_capture
        assert before is not None, "document capture was missing"
        close_offset = self.debug_cursor()
        close_started = time.monotonic()
        close_deadline = close_started + DOCUMENT_CLOSE_DEADLINE_SECONDS
        BASE.command("xdotool", "key", "--window", self.window_id, "ctrl+w")
        drop_index, _ = OBSERVER.wait_for_debug(
            self,
            "document surface drop", close_offset, "document_surface_drop", close_deadline,
            guard_terms=False, generation=generation, uri=self.expected_uri,
        )
        OBSERVER.wait_for_debug(
            self,
            "document session close", drop_index + 1, "document_session_closed", close_deadline,
            guard_terms=False, generation=generation, uri=self.expected_uri,
        )
        OBSERVER.require_before_deadline("document close event observation", close_deadline)
        OBSERVER.capture_nonblank(self, "document-after-close")
        OBSERVER.require_before_deadline("document close capture", close_deadline)
        changed = self.last_capture.read_bytes() != before.read_bytes()
        OBSERVER.require_before_deadline("document close comparison", close_deadline)
        assert changed, "visible document did not change after Ctrl+W"
        zero_receipt = self.receipt.wait(
            "post-close zero resources", close_deadline,
            PARSER.receipt_resources_are_zero,
        )
        close_elapsed_ms = int((time.monotonic() - close_started) * 1000)
        if close_elapsed_ms > DOCUMENT_CLOSE_DEADLINE_SECONDS * 1000:
            raise AssertionError(f"document close exceeded {DOCUMENT_CLOSE_DEADLINE_SECONDS}s")
        self.record("document-close-observed", generation=int(generation),
                    close_deadline_s=DOCUMENT_CLOSE_DEADLINE_SECONDS,
                    close_elapsed_ms=close_elapsed_ms,
                    resource_receipt_seq=zero_receipt["seq"],
                    resource_receipt_status="post-close-raw-zero")

    def close_window(self):
        BASE.command("wmctrl", "-i", "-c", hex(int(self.window_id)))
        BASE.until("window closed through WM request", lambda: not self.find_window())
        self.record("wm-close-acknowledged", final_frame=self.frame())

    def run(self):
        generation = self.start_document()
        self.close_document(generation)
        self.close_window()
        self.record("partial-result", resource_counters="receipt_observed",
                    acceptance_status="partial_mechanics_only")


def main() -> int:
    return CLI.main(DocumentAcceptance, TermsBlocked)


if __name__ == "__main__":
    raise SystemExit(main())
