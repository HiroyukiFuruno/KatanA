#!/usr/bin/env python3
"""Unit tests for the fixed localhost document-host control panel."""

from __future__ import annotations

import contextlib
import http.client
import io
import json
from pathlib import Path
import sys
import tempfile
import threading
import unittest
from unittest.mock import Mock

import document_host_control_panel as panel_module


class FakePanel:
    def __init__(self):
        self.csrf = "test-csrf-token"
        self.state = {"id": "panel-test-id", "cycle": 0, "kind": "none", "state": "cold",
                      "operation": None, "actual_watcher_ready": False, "watcher_id": None}
        self.actions = []

    def view(self):
        return dict(self.state)

    def action(self, name):
        self.actions.append(name)
        if name not in panel_module.ACTION_NAMES:
            raise panel_module.PanelError("invalid_action")
        return self.view()


class ControlPanelHttpTests(unittest.TestCase):
    def setUp(self):
        self.panel = FakePanel()
        self.server = panel_module.create_server(self.panel)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()

    def tearDown(self):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=2)

    def request(self, method, path, body=None, headers=None):
        connection = http.client.HTTPConnection("127.0.0.1", self.server.server_port, timeout=3)
        connection.request(method, path, body=body, headers=headers or {})
        response = connection.getresponse()
        result = response.status, response.getheader("Content-Type"), response.read().decode()
        connection.close()
        return result

    def post_headers(self, origin=None, csrf=None):
        return {"Origin": origin or f"http://127.0.0.1:{self.server.server_port}",
                "Host": f"127.0.0.1:{self.server.server_port}",
                "Content-Type": "application/json", "X-Panel-CSRF": csrf or self.panel.csrf}

    def test_state_response_contains_only_allowed_fields(self):
        status, content_type, body = self.request("GET", "/state")
        self.assertEqual(status, 200)
        self.assertEqual(content_type, "application/json")
        self.assertEqual(set(json.loads(body)), {"id", "cycle", "kind", "state",
                                                 "operation", "actual_watcher_ready", "watcher_id"})
        self.assertNotIn("path", body)
        self.assertNotIn("filename", body)

    def test_html_is_fixed_and_has_no_private_input_controls_or_external_urls(self):
        status, _, body = self.request("GET", "/")
        self.assertEqual(status, 200)
        self.assertIn('data-action="cold"', body)
        self.assertIn('data-action="operation-before"', body)
        self.assertIn("role=status", body)
        self.assertIn("out.dataset.actualWatcherReady", body)
        self.assertIn("out.dataset.watcherId", body)
        self.assertIn("out.dataset.operation", body)
        self.assertNotIn("type=\"file\"", body)
        self.assertNotIn("http://", body.replace(f"http://127.0.0.1:{self.server.server_port}", ""))
        self.assertNotIn("/Users/", body)
        self.assertIn("role=alert", body)
        self.assertIn("response.ok?'':'request_failed'", body)
        self.assertIn("catch{document.querySelector('#error').textContent='request_failed';}", body)
        self.assertNotIn("response.text()", body)

    def test_foreign_host_is_rejected(self):
        status, _, _ = self.request("GET", "/state", headers={"Host": "foreign.example"})
        self.assertEqual(status, 400)

    def test_foreign_origin_is_rejected(self):
        status, _, _ = self.request("POST", "/action", body='{"action":"cold"}',
                                    headers=self.post_headers(origin="http://foreign.example"))
        self.assertEqual(status, 403)
        self.assertEqual(self.panel.actions, [])

    def test_invalid_csrf_is_rejected(self):
        status, _, _ = self.request("POST", "/action", body='{"action":"cold"}',
                                    headers=self.post_headers(csrf="wrong"))
        self.assertEqual(status, 403)
        self.assertEqual(self.panel.actions, [])

    def test_action_body_rejects_arbitrary_fields_and_duplicate_keys(self):
        extra, _, _ = self.request("POST", "/action", body='{"action":"cold","path":"/private"}',
                                   headers=self.post_headers())
        duplicate, _, _ = self.request("POST", "/action", body='{"action":"cold","action":"open"}',
                                       headers=self.post_headers())
        self.assertEqual(extra, 400)
        self.assertEqual(duplicate, 400)
        self.assertEqual(self.panel.actions, [])

    def test_arbitrary_action_name_is_rejected(self):
        status, _, body = self.request("POST", "/action", body='{"action":"run-code"}',
                                       headers=self.post_headers())
        self.assertEqual(status, 409)
        self.assertEqual(json.loads(body), {"error": "invalid_action"})

    def test_query_and_non_action_routes_are_rejected(self):
        status, _, _ = self.request("POST", "/action?path=/tmp", body='{"action":"cold"}',
                                    headers=self.post_headers())
        self.assertEqual(status, 404)


class FixedInputAndFailureTests(unittest.TestCase):
    def test_cli_path_requires_existing_absolute_non_symlink(self):
        with tempfile.TemporaryDirectory(prefix="panel-path-") as name:
            path = Path(name) / "input"
            path.write_text("fixture", encoding="utf-8")
            self.assertEqual(panel_module.validate_cli_path(path, "input"), path.resolve())
            with self.assertRaises(panel_module.PanelError):
                panel_module.validate_cli_path(Path("relative"), "input")
            link = path.with_name("input-link")
            try:
                link.symlink_to(path)
            except OSError as error:
                if sys.platform == "win32" and getattr(error, "winerror", None) == 1314:
                    self.skipTest("Windows symlink privilege is unavailable in this runner")
                raise
            with self.assertRaises(panel_module.PanelError):
                panel_module.validate_cli_path(link, "input")

    def test_view_exposes_only_a_whitelisted_operation(self):
        panel = panel_module.DocumentHostPanel.__new__(panel_module.DocumentHostPanel)
        panel.condition = threading.Condition()
        panel.current_cycle = 1
        panel.current_kind = "html"
        panel.phase = "operation_after"
        panel.current_operation = "../../private/input"
        panel.watcher_row = None
        panel.watcher_ready = False
        panel.watcher_id = None
        panel.panel_id = "panel-id"
        panel.position = 0
        panel.sequence = [(1, "html")]
        self.assertIsNone(panel.view()["operation"])
        panel.current_operation = "scroll"
        self.assertEqual(panel.view()["operation"], "scroll")

    def test_close_uses_original_separate_postclosed_sample_timeout(self):
        panel = panel_module.DocumentHostPanel.__new__(panel_module.DocumentHostPanel)
        panel.run = Path("/unit-only")
        panel.output = Path("/unit-output")
        panel.current_label = "unit-cycle01-office"
        panel.current_cycle = 1
        panel.current_kind = "office"
        panel.panel_id = "unit-panel-identity"
        args = panel._stage_args("close")
        self.assertEqual(args.sample_timeout_seconds, 300.0)

    def test_recorder_failure_is_terminal_and_cannot_be_retried(self):
        panel = panel_module.DocumentHostPanel.__new__(panel_module.DocumentHostPanel)
        panel.action_lock = threading.Lock()
        panel.condition = threading.Condition()
        panel.failed = False
        panel.failure_code = ""
        panel.phase = "cold"
        panel.recorder = Mock()
        panel.recorder.observe_cold.side_effect = RuntimeError("simulated recorder failure")
        panel.run = Path("/unit-only")
        panel.sequence = [(1, "html"), (1, "office")]
        panel.position = -1
        panel.current_cycle = None
        panel.current_kind = None
        panel.current_label = None
        panel.panel_id = "unit-panel-identity"
        panel.watcher_ready = False
        panel.watcher_id = None
        panel.worker = None
        panel._verify_invariants = Mock()
        with self.assertRaisesRegex(panel_module.PanelError, "case_failed"):
            panel.action("cold")
        self.assertTrue(panel.failed)
        with self.assertRaisesRegex(panel_module.PanelError, "case_failed"):
            panel.action("cold")
        panel.recorder.observe_cold.assert_called_once()

    def test_open_wait_returns_on_actual_watcher_ready_before_worker_finishes(self):
        ready_written = threading.Event()
        allow_finish = threading.Event()

        class FakeStatus:
            def __init__(self, path, log, stage, run, label):
                self.path = path
                self.log_path = log
                self.row = {"watch_id": "actual-unit-watch", "stage": stage, "label": label}

            def write(self, state, ready, **details):
                self.row.update({"state": state, "watcher_ready": ready, **details})
                self.path.write_text(json.dumps(self.row), encoding="utf-8")
                if ready:
                    ready_written.set()

            def close(self):
                return

        def open_stage(_args, _recorder, status, _label):
            status.write("watching_open", True, pid=123, start_identity="unit-start")
            allow_finish.wait(timeout=2)
            status.write("ready_observed", False, pid=123, start_identity="unit-start")

        panel = panel_module.DocumentHostPanel.__new__(panel_module.DocumentHostPanel)
        panel.condition = threading.Condition()
        panel.failed = False
        panel.failure_code = ""
        panel.watcher_ready = False
        panel.watcher_id = None
        panel.watcher_row = None
        panel.worker_done = True
        panel.worker = None
        panel.worker_stage = "open"
        panel.current_label = "unit-cycle01-html"
        panel.current_cycle = 1
        panel.current_kind = "html"
        panel.run = Path("/unit-only")
        panel.recorder = object()
        temporary = tempfile.TemporaryDirectory(prefix="panel-status-")
        panel.output = Path(temporary.name)
        panel._stage_args = lambda stage: type("Args", (), {
            "status_file": panel.output / (stage + ".json"),
            "log_file": panel.output / (stage + ".log"),
        })()
        panel.watcher = type("FakeWatcher", (), {"Status": FakeStatus, "open_stage": open_stage})
        panel.worker = None

        try:
            panel._start_watch("open")
            self.assertTrue(ready_written.is_set())
            self.assertFalse(panel.worker_done)
            self.assertTrue(panel.watcher_ready)
            self.assertEqual(panel.watcher_id, "actual-unit-watch")
            allow_finish.set()
            panel._wait_worker()
            self.assertTrue(panel.worker_done)
            self.assertEqual(panel.phase, "operation_before")
        finally:
            allow_finish.set()
            panel.worker.join(timeout=2)
            temporary.cleanup()

    def test_watcher_failure_is_private_and_preserves_actual_diagnostic(self):
        class FakeStatus:
            def __init__(self, path, log, stage, run, label):
                self.path = path
                self.log_path = log
                self.row = {"stage": stage, "label": label, "watch_id": "watch-private"}

            def write(self, state, ready, **details):
                self.row.update({"state": state, "watcher_ready": ready, **details})
                encoded = json.dumps(self.row)
                self.path.write_text(encoded, encoding="utf-8")
                with self.log_path.open("a", encoding="utf-8") as stream:
                    stream.write(encoded + "\n")

            def close(self):
                return

        def fail_open(_args, _recorder, _status, _label):
            raise RuntimeError("diagnostic sample")

        with tempfile.TemporaryDirectory(prefix="panel-failure-") as name:
            panel = panel_module.DocumentHostPanel.__new__(panel_module.DocumentHostPanel)
            panel.condition = threading.Condition()
            panel.failed = False
            panel.failure_code = ""
            panel.watcher_ready = False
            panel.watcher_id = None
            panel.watcher_row = None
            panel.worker_done = True
            panel.worker = None
            panel.worker_stage = "open"
            panel.current_label = "unit-cycle01-html"
            panel.current_cycle = 1
            panel.current_kind = "html"
            panel.current_operation = None
            panel.panel_id = "panel-unit"
            panel.position = 0
            panel.run = Path("/unit-only")
            panel.recorder = object()
            panel.output = Path(name)
            panel._stage_args = lambda stage: type("Args", (), {
                "status_file": panel.output / (stage + ".json"),
                "log_file": panel.output / (stage + ".log"),
            })()
            panel.watcher = type("FakeWatcher", (), {"Status": FakeStatus, "open_stage": fail_open})
            error_output = io.StringIO()
            with contextlib.redirect_stderr(error_output):
                panel._worker_status("open", panel.current_label)
            saved = json.loads((panel.output / "open.json").read_text(encoding="utf-8"))
            logged = (panel.output / "open.log").read_text(encoding="utf-8")
            self.assertEqual(saved["state"], "failed")
            self.assertEqual(saved["error_type"], "RuntimeError")
            self.assertIn("diagnostic sample", saved["error"])
            self.assertIn("diagnostic sample", logged)
            self.assertIn("RuntimeError: diagnostic sample", error_output.getvalue())
            self.assertEqual(set(panel._view()), {"id", "cycle", "kind", "state", "operation",
                                                  "actual_watcher_ready", "watcher_id"})

    def test_twenty_case_sequence_finishes_cycle_ten_html_then_office_before_quit(self):
        panel = panel_module.DocumentHostPanel.__new__(panel_module.DocumentHostPanel)
        panel.condition = threading.Condition()
        panel.action_lock = threading.Lock()
        panel.failed = False
        panel.failure_code = ""
        panel.sequence = [(cycle, kind) for cycle in range(1, 11) for kind in ("html", "office")]
        panel.position = -1
        panel.current_cycle = None
        panel.current_kind = None
        panel.current_label = None
        panel.current_operation = None
        panel.phase = "cold"
        panel.run = Path("/unit-only")
        panel.panel_id = "unit-panel-identity"
        panel.watcher_row = None
        panel.watcher_ready = False
        panel.watcher_id = None
        panel.worker = None
        panel.worker_stage = None
        panel.worker_done = True
        panel.recorder = Mock()
        panel.recorder.validate_label.side_effect = lambda _prefix, cycle, kind: f"cycle{cycle:02}-{kind}"
        panel._verify_invariants = Mock()

        def choose_operation():
            if panel.current_kind == "html":
                return {1: "scroll", 2: "reload"}.get(panel.current_cycle)
            return "slide" if panel.current_cycle == 1 else None

        panel._choose_operation = choose_operation
        panel._capture_operation = Mock()
        starts = []

        def start_watch(stage):
            starts.append((panel.current_cycle, panel.current_kind, stage))
            panel.worker = object()
            panel.worker_stage = stage
            panel.worker_done = True
            panel.watcher_ready = True
            panel.watcher_id = f"watch-{len(starts)}"
            panel.watcher_row = {"state": "watching_open" if stage == "open" else "watching_closed"}

        panel._start_watch = start_watch

        panel.action("cold")
        for index, (cycle, kind) in enumerate(panel.sequence[:-1]):
            if index:
                panel.action("open")
            self.assertEqual((panel.current_cycle, panel.current_kind), (cycle, kind))
            if index == 0:
                panel.action("open")
            if panel._requires_operation():
                result = panel.action("operation-before")
                self.assertIn(result["operation"], panel_module.OPERATION_NAMES)
                panel.action("operation-after")
            panel.action("close")
            result = panel.action("status")
            self.assertTrue(result["state"].startswith("finished/"))

        self.assertEqual(panel.view()["cycle"], 10)
        self.assertEqual(panel.view()["kind"], "html")
        self.assertEqual(panel.phase, "finished")
        html = panel_module._page(32123, "unit-csrf")
        self.assertIn("!(d.cycle===10&&d.kind==='office')", html)
        self.assertIn("d.cycle===10&&d.kind==='office'", html)
        self.assertIn("phase==='finished'&&b.dataset.action==='quit-start'", html)
        self.assertIn("phase==='finished'&&b.dataset.action==='open'", html)

        panel.action("open")
        self.assertEqual((panel.current_cycle, panel.current_kind), (10, "office"))
        panel.action("close")
        finished = panel.action("status")
        self.assertEqual((finished["cycle"], finished["kind"], panel.phase), (10, "office", "finished"))
        panel.action("quit-start")
        self.assertEqual(panel.phase, "done")
        self.assertEqual(sum(stage == "open" for _, _, stage in starts), 20)
        self.assertEqual(sum(stage == "close" for _, _, stage in starts), 20)
        self.assertEqual(panel._capture_operation.call_count, 6)


if __name__ == "__main__":
    unittest.main()
