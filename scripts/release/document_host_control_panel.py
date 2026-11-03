#!/usr/bin/env python3
"""Serve a fixed localhost control surface for the existing host recorder."""

from __future__ import annotations

import argparse
import builtins
import hashlib
import hmac
import http.server
import importlib.util
import json
from pathlib import Path
import secrets
import sys
import threading
from types import SimpleNamespace

ROOT = Path(__file__).resolve().parents[2]
TMP = ROOT / "tmp" / "pr405-resolution"
RELEASE = ROOT / "scripts" / "release"
RECORDER_PATH = RELEASE / "record-document-host-observation.py"
WATCHER_PATH = RELEASE / "run-document-host-observation-watch.py"
OPERATION_PATH = RELEASE / "record-document-host-operation.py"
HTML_INPUT = TMP / "host-acceptance" / "requirements.html"
FIXTURE_PATH = RELEASE / "check-document-fidelity-acceptance-evidence.py"
ACTION_NAMES = frozenset({"cold", "open", "close", "operation-before", "operation-after",
                          "quit-start", "status"})
OPERATION_NAMES = frozenset({"scroll", "reload", "single_item", "page", "sheet", "slide"})
HTML_SHA256 = "c02d2d7a2420e4e15e3d98a044a310c67bc75fa858c95c4867b9c3f5d7aca012"


class PanelError(RuntimeError):
    """Raised when the fixed panel contract is violated."""


def _load(path: Path, name: str):
    if path.is_symlink() or not path.is_file():
        raise PanelError("required release helper is missing or unsafe")
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise PanelError("required release helper could not be loaded")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _sha(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _bundle_hash(path_text: str, expected: str) -> None:
    path = Path(path_text)
    if not path.is_absolute() or path.is_symlink() or not path.is_file() or _sha(path) != expected:
        raise PanelError("published bundle bytes differ from before identity")


def _json(path: Path) -> dict:
    if path.is_symlink() or not path.is_file():
        raise PanelError("required before snapshot is missing or unsafe")
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise PanelError("required before snapshot is malformed")
    return value


def validate_cli_path(path: Path, label: str) -> Path:
    if not path.is_absolute() or path.is_symlink() or not path.exists():
        raise PanelError(f"{label} must be an existing absolute path")
    return path.resolve(strict=True)


class DocumentHostPanel:
    """Own a terminal, fixed action sequence and one active watcher worker."""

    def __init__(self, run: Path, office_input: Path, output: Path):
        self.run = validate_cli_path(run, "run directory")
        self.office_input = validate_cli_path(office_input, "Office input")
        self.output = output
        if self.run.parent != ROOT / "tmp" or not self.run.name.startswith("macos-document-host.") or not self.run.is_dir():
            raise PanelError("actual isolated host run required")
        if not self.office_input.is_file():
            raise PanelError("Office input must be a regular file")
        if not output.is_absolute() or not output.parent.resolve().is_relative_to(TMP.resolve()):
            raise PanelError("output directory must be a new child of tmp/pr405-resolution")
        if output.parent.is_symlink() or not output.parent.is_dir() or output.is_symlink() or output.exists():
            raise PanelError("output directory already exists or is unsafe")
        current = TMP
        for part in output.parent.relative_to(TMP).parts:
            current = current / part
            if current.is_symlink():
                raise PanelError("output path contains a symlink")

        sys.path.insert(0, str(RELEASE)) if str(RELEASE) not in sys.path else None
        self.recorder = _load(RECORDER_PATH, "_document_host_panel_recorder")
        self.watcher = _load(WATCHER_PATH, "_document_host_panel_watcher")
        self.operation = _load(OPERATION_PATH, "_document_host_panel_operation")
        self.controller_module = _load(RELEASE / "document_host_controller.py", "_document_host_panel_controller")
        self.transcript = _load(RELEASE / "document_host_transcript.py", "_document_host_panel_transcript")
        self.fixtures = _load(FIXTURE_PATH, "_document_host_panel_fixtures")
        self.recorder.validate_run(self.run)

        self.html_input = HTML_INPUT.resolve(strict=True)
        self.html_sha256 = _sha(self.html_input)
        self.office_sha256 = _sha(self.office_input)
        if self.html_sha256 != self.fixtures.ORIGINAL_HTML_SHA256 or self.html_sha256 != HTML_SHA256:
            raise PanelError("fixed HTML input identity differs")
        if self.office_sha256 not in self.fixtures.SUPPLIED_OFFICE_FIXTURES:
            raise PanelError("Office input is not a supplied original")
        self.office_format = self.fixtures.SUPPLIED_OFFICE_FIXTURES[self.office_sha256]

        self.bundle_path = self.run / "bundle-identity.before.json"
        self.source_path = self.run / "source-identity.before.json"
        self.controller_path = self.run / "controller-identity.before.json"
        self.head_path = self.run / "head.before"
        self.pid_path = self.run / "app.pid"
        self.launch_path = self.run / "launch.json"
        self.settings_path = self.run / "settings.before.json"
        self.run_settings_path = self.run / "run-settings.json"
        self._load_baseline()

        output.mkdir(mode=0o700)
        self.condition = threading.Condition()
        self.action_lock = threading.Lock()
        self.failed = False
        self.failure_code = ""
        self.worker = None
        self.worker_stage = None
        self.worker_done = True
        self.watcher_row = None
        self.watcher_ready = False
        self.watcher_ready_seen = False
        self.watcher_id = None
        self.sequence = [(cycle, kind) for cycle in range(1, 11) for kind in ("html", "office")]
        self.position = -1
        self.phase = "cold"
        self.current_cycle = None
        self.current_kind = None
        self.current_label = None
        self.current_operation = None
        self.panel_id = secrets.token_hex(16)
        self.csrf = secrets.token_urlsafe(32)

    def _load_baseline(self):
        self.source = _json(self.source_path)
        self.controller = _json(self.controller_path)
        self.bundle = _json(self.bundle_path)
        if self.bundle.get("provenance") != self.source:
            raise PanelError("before source and bundle identity differ")
        if self.bundle.get("controller") != self.controller:
            raise PanelError("before controller and bundle identity differ")
        binding = self.controller_module.verify_binding(ROOT, self.source, self.controller)
        if not isinstance(binding, dict):
            raise PanelError("published controller binding is invalid")
        if self.head_path.is_symlink() or not self.head_path.is_file():
            raise PanelError("actual before HEAD snapshot missing")
        head = self.head_path.read_text(encoding="utf-8").strip()
        if head != self.controller.get("controller_commit"):
            raise PanelError("actual before HEAD differs from controller")
        launch = _json(self.launch_path)
        architecture = "arm64" if self.bundle.get("target") == "macos-arm64" else "x86_64"
        if self.bundle.get("target") not in {"macos-arm64", "macos-x86_64"} \
                or launch.get("argv") != ["/usr/bin/arch", "-" + architecture, self.bundle.get("main_path")] \
                or launch.get("target") != self.bundle.get("target"):
            raise PanelError("actual launch identity differs from before bundle")
        _bundle_hash(self.bundle["main_path"], self.bundle["main_sha256"])
        _bundle_hash(self.bundle["worker_path"], self.bundle["sidecar_sha256"])
        settings = _json(self.run_settings_path)
        self.transcript.settings(settings)
        self._baseline_hashes = {
            path: _sha(path) for path in (self.bundle_path, self.source_path, self.controller_path,
                                           self.head_path, self.pid_path, self.launch_path,
                                           self.settings_path, self.run_settings_path)
        }
        self._settings = settings
        self._pid, self._identity = self.recorder.run_identity(self.run)
        if self._identity.get("executable") != self.bundle.get("main_path"):
            raise PanelError("actual process differs from published main binary")
        self._verify_invariants()

    def _verify_invariants(self):
        for path, expected in self._baseline_hashes.items():
            if path.is_symlink() or not path.is_file() or _sha(path) != expected:
                raise PanelError("before identity snapshot changed")
        if _json(self.bundle_path) != self.bundle or _json(self.source_path) != self.source \
                or _json(self.controller_path) != self.controller:
            raise PanelError("before identity snapshot changed")
        identity_pid, identity = self.recorder.run_identity(self.run)
        if identity_pid != self._pid or identity != self._identity:
            raise PanelError("actual process PID/start identity changed")
        if identity.get("executable") != self.bundle.get("main_path"):
            raise PanelError("actual process executable changed")
        _bundle_hash(self.bundle["main_path"], self.bundle["main_sha256"])
        _bundle_hash(self.bundle["worker_path"], self.bundle["sidecar_sha256"])
        if _sha(self.office_input) != self.office_sha256 or _sha(self.html_input) != self.html_sha256:
            raise PanelError("fixed input bytes changed")

    def _view(self):
        cycle = self.current_cycle
        kind = self.current_kind
        if self.phase == "cold":
            cycle, kind = 0, "none"
        elif self.phase == "done":
            cycle, kind = 10, "office"
        elif cycle is None:
            cycle, kind = (self.sequence[self.position + 1] if self.position + 1 < len(self.sequence)
                           else (10, "office"))
        actual = self.watcher_row.get("state") if isinstance(self.watcher_row, dict) else None
        state = self.phase if actual is None else self.phase + "/" + actual
        operation = self.current_operation if self.current_operation in OPERATION_NAMES else None
        return {"id": self.panel_id, "cycle": cycle, "kind": kind, "state": state,
                "operation": operation,
                "actual_watcher_ready": self.watcher_ready, "watcher_id": self.watcher_id}

    def view(self):
        with self.condition:
            return self._view()

    def notify_status(self, row):
        with self.condition:
            if (row.get("stage"), row.get("label")) != (self.worker_stage, self.current_label):
                self._fail_locked()
                return
            self.watcher_row = row
            self.watcher_ready = bool(row.get("watcher_ready"))
            if self.watcher_ready:
                self.watcher_ready_seen = True
            self.watcher_id = row.get("watch_id")
            if row.get("state") == "failed":
                self._fail_locked()
            self.condition.notify_all()

    def _fail_locked(self):
        self.failed = True
        self.failure_code = "case_failed"
        self.phase = "failed"
        self.watcher_ready = False
        self.condition.notify_all()

    def _ensure_phase(self, expected):
        if self.failed:
            raise PanelError("case_failed")
        if self.phase != expected:
            raise PanelError("action_not_available")
        self._verify_invariants()

    def _stage_args(self, stage):
        cycle, kind = self.current_cycle, self.current_kind
        label = self.current_label
        common = {"run_directory": self.run, "prefix": self.panel_id[:12], "cycle": cycle,
                  "kind": kind, "status_file": self.output / f"{label}-{stage}.status.json",
                  "log_file": self.output / f"{label}-{stage}.log"}
        if stage == "open":
            common["input"] = self.html_input if kind == "html" else self.office_input
        else:
            common["sample_timeout_seconds"] = 300.0
        return SimpleNamespace(**common)

    def _worker_status(self, stage, label):
        panel = self
        watcher = self.watcher

        class PanelStatus(watcher.Status):
            def write(self, state, ready, **details):
                super().write(state, ready, **details)
                row = json.loads(self.path.read_text(encoding="utf-8"))
                panel.notify_status(row)

        args = self._stage_args(stage)
        status = None
        try:
            status = PanelStatus(args.status_file, args.log_file, stage, self.run, label)
            if stage == "open":
                watcher.open_stage(args, self.recorder, status, label)
            else:
                watcher.close_stage(args, self.recorder, status, label)
        except Exception as error:
            diagnostic = f"{type(error).__name__}: {error}"
            print("ERROR: document-host watcher: " + diagnostic, file=sys.stderr, flush=True)
            if status is not None:
                try:
                    status.write("failed", False, error_type=type(error).__name__, error=diagnostic[:2048])
                except Exception as status_error:
                    print("ERROR: document-host failure status: "
                          + f"{type(status_error).__name__}: {status_error}", file=sys.stderr, flush=True)
            with self.condition:
                self._fail_locked()
        finally:
            if status is not None:
                try:
                    status.close()
                except Exception as close_error:
                    print("ERROR: document-host status close: "
                          + f"{type(close_error).__name__}: {close_error}", file=sys.stderr, flush=True)
                    with self.condition:
                        self._fail_locked()
            with self.condition:
                self.worker_done = True
                self.condition.notify_all()

    def _start_watch(self, stage):
        label = self.current_label
        self.worker_stage = stage
        self.worker_done = False
        self.watcher_ready = False
        self.watcher_ready_seen = False
        self.watcher_id = None
        self.watcher_row = None
        self.worker = threading.Thread(target=self._worker_status, args=(stage, label), daemon=True)
        self.worker.start()
        with self.condition:
            while not self.watcher_ready_seen and not self.failed and not self.worker_done:
                self.condition.wait()
            if self.failed or not self.watcher_ready_seen:
                self._fail_locked()
                raise PanelError("case_failed")
            return self.worker_done

    def _wait_worker(self):
        with self.condition:
            while not self.worker_done and not self.failed:
                self.condition.wait()
            if self.failed:
                raise PanelError("case_failed")
            if self.worker_stage == "open":
                self.current_operation = self._choose_operation() if self._requires_operation() else None
                self.phase = "operation_before" if self._requires_operation() else "close"
            elif self.worker_stage == "close":
                self.phase = "finished"
            self.worker_stage = None

    def _advance_case(self):
        self.position += 1
        self.current_operation = None
        self.watcher_row = None
        self.watcher_ready = False
        self.watcher_id = None
        if self.position >= len(self.sequence):
            return False
        self.current_cycle, self.current_kind = self.sequence[self.position]
        self.current_label = self.recorder.validate_label(self.panel_id[:12], self.current_cycle,
                                                          self.current_kind)
        self.phase = "open"
        return True

    def _requires_operation(self):
        return ((self.current_kind == "html" and self.current_cycle in (1, 2))
                or (self.current_kind == "office" and self.current_cycle == 1))

    def _choose_operation(self):
        if self.current_kind == "html":
            return {1: "scroll", 2: "reload"}.get(self.current_cycle)
        if self.current_cycle != 1:
            return None
        label = self.current_label
        receipt_mod = self.recorder.module("_host_panel_operation_receipt", "document_host_observation.py")
        item = receipt_mod.receipt(self.recorder.read(self.run / (label + "-open.receipt.json")),
                                   self.run.name, self._pid)
        observations = item.get("document_observations")
        if not isinstance(observations, list) or len(observations) != 1:
            raise PanelError("case_failed")
        frame = observations[0].get("frame")
        if not isinstance(frame, dict):
            raise PanelError("case_failed")
        if frame.get("item_count") == 1:
            return "single_item"
        return {"docx": "page", "xlsx": "sheet", "pptx": "slide"}[self.office_format]

    def _capture_operation(self, phase):
        self._verify_invariants()
        self.operation.capture(self.run, self.current_label, self.current_operation, phase)

    def action(self, name):
        with self.action_lock:
            try:
                return self._action(name)
            except PanelError as error:
                if str(error) == "case_failed":
                    with self.condition:
                        self._fail_locked()
                raise
            except (OSError, ValueError, RuntimeError, KeyError, TypeError, IndexError):
                with self.condition:
                    self._fail_locked()
                raise PanelError("case_failed") from None

    def _action(self, name):
        if name not in ACTION_NAMES:
            raise PanelError("invalid_action")
        if name == "status":
            if self.worker is not None:
                self._wait_worker()
            return self.view()
        if name == "cold":
            self._ensure_phase("cold")
            self.recorder.observe_cold(self.run)
            self.position = 0
            self.current_cycle, self.current_kind = self.sequence[0]
            self.current_label = self.recorder.validate_label(self.panel_id[:12], self.current_cycle,
                                                              self.current_kind)
            self.phase = "open"
        elif name == "open":
            if self.phase == "finished":
                if not self._advance_case():
                    raise PanelError("action_not_available")
            self._ensure_phase("open")
            self.worker_stage = "open"
            self.phase = "native_open"
            completed = self._start_watch("open")
            if completed:
                self._wait_worker()
        elif name == "operation-before":
            if self.phase == "native_open":
                self._wait_worker()
                if self.current_operation is not None:
                    self._capture_operation("before")
                    self.phase = "operation_after"
                else:
                    self.phase = "close"
            elif self.phase == "operation_before":
                if self.current_operation is None:
                    self.phase = "close"
                else:
                    self._capture_operation("before")
                    self.phase = "operation_after"
            else:
                raise PanelError("action_not_available")
        elif name == "operation-after":
            self._ensure_phase("operation_after")
            self._capture_operation("after")
            self.phase = "close"
        elif name == "close":
            if self.phase == "native_open":
                self._wait_worker()
                if self.current_operation:
                    raise PanelError("action_not_available")
            elif self.phase not in {"close", "native_open"}:
                raise PanelError("action_not_available")
            self._ensure_phase("close")
            self.worker_stage = "close"
            self.phase = "native_close"
            completed = self._start_watch("close")
            if completed:
                self._wait_worker()
        elif name == "quit-start":
            if self.phase == "finished" and self.position == len(self.sequence) - 1:
                self.phase = "quit_start"
            self._ensure_phase("quit_start")
            self.recorder.observe_quit_start(self.run)
            self.phase = "done"
        else:
            raise PanelError("action_not_available")
        return self.view()


class PanelHandler(http.server.BaseHTTPRequestHandler):
    server_version = "DocumentHostPanel/1"
    sys_version = ""

    def log_message(self, _format, *args):
        return

    @property
    def panel(self):
        return self.server.panel

    def _send(self, status, content_type, body):
        encoded = body.encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(encoded)))
        self.send_header("Cache-Control", "no-store")
        self.send_header("X-Content-Type-Options", "nosniff")
        self.send_header("Content-Security-Policy", "default-src 'none'; script-src 'unsafe-inline'; connect-src 'self'; style-src 'none'; img-src 'none'; base-uri 'none'; form-action 'none'")
        self.end_headers()
        self.wfile.write(encoded)

    def _valid_host(self):
        expected = f"127.0.0.1:{self.server.server_port}"
        values = self.headers.get_all("Host", [])
        return len(values) == 1 and values[0] == expected

    def _valid_origin(self):
        values = self.headers.get_all("Origin", [])
        return len(values) == 1 and values[0] == f"http://127.0.0.1:{self.server.server_port}"

    def do_GET(self):
        if not self._valid_host():
            return self._send(400, "application/json", '{"error":"invalid_host"}')
        if self.path == "/state":
            return self._send(200, "application/json", json.dumps(self.panel.view(), separators=(",", ":")))
        if self.path != "/":
            return self._send(404, "application/json", '{"error":"not_found"}')
        page = _page(self.server.port, self.panel.csrf)
        self._send(200, "text/html; charset=utf-8", page)

    def do_POST(self):
        if not self._valid_host() or not self._valid_origin():
            return self._send(403, "application/json", '{"error":"origin_rejected"}')
        if self.path != "/action":
            return self._send(404, "application/json", '{"error":"not_found"}')
        supplied = self.headers.get("X-Panel-CSRF", "")
        if not hmac.compare_digest(supplied, self.panel.csrf):
            return self._send(403, "application/json", '{"error":"csrf_rejected"}')
        if self.headers.get("Content-Type", "").split(";", 1)[0].strip().lower() != "application/json":
            return self._send(400, "application/json", '{"error":"invalid_request"}')
        try:
            size = int(self.headers.get("Content-Length", "-1"))
            if size < 0 or size > 256:
                raise PanelError("invalid_request")
            raw = self.rfile.read(size)
            pairs = json.loads(raw, object_pairs_hook=lambda items: _unique_pairs(items))
            if not isinstance(pairs, dict) or set(pairs) != {"action"} or not isinstance(pairs["action"], str):
                raise PanelError("invalid_request")
            result = self.panel.action(pairs["action"])
            return self._send(200, "application/json", json.dumps(result, separators=(",", ":")))
        except PanelError as error:
            if str(error) == "invalid_request":
                return self._send(400, "application/json", '{"error":"invalid_request"}')
            return self._send(409, "application/json", json.dumps({"error": str(error)}))
        except (ValueError, UnicodeError, json.JSONDecodeError):
            return self._send(400, "application/json", '{"error":"invalid_request"}')


def _unique_pairs(items):
    result = {}
    for key, value in items:
        if key in result:
            raise ValueError("duplicate JSON key")
        result[key] = value
    return result


def _page(port, csrf):
    actions = "cold open operation-before operation-after close quit-start status".split()
    buttons = "".join(f'<button type="button" data-action="{action}">{action}</button>' for action in actions)
    return ("<!doctype html><meta charset=utf-8><title>Document host QA</title>"
            "<main><h1>Document host QA</h1><output id=state role=status aria-live=polite>loading</output>"
            "<output id=error role=alert></output><nav>" + buttons +
            "</nav><script>const token=" + json.dumps(csrf) + ";"
            "const fields=['cycle','kind','state','operation','actual_watcher_ready','watcher_id'];"
            "const allowed={cold:['cold'],open:['open'],native_open:['operation-before','close','status'],"
            "operation_before:['operation-before'],operation_after:['operation-after'],close:['close'],"
            "native_close:['status'],finished:['open','quit-start'],quit_start:['quit-start'],done:[],failed:[]};"
            "async function refresh(){const r=await fetch('/state',{cache:'no-store'});const d=await r.json();"
            "const out=document.querySelector('#state');out.textContent=fields.map(k=>k+': '+d[k]).join(' | ');"
            "out.dataset.cycle=d.cycle;out.dataset.kind=d.kind;out.dataset.state=d.state;"
            "out.dataset.operation=d.operation||'';"
            "out.dataset.actualWatcherReady=d.actual_watcher_ready;out.dataset.watcherId=d.watcher_id||'';"
            "const phase=d.state.split('/')[0];for(const b of document.querySelectorAll('button'))"
            "b.disabled=!(allowed[phase]||[]).includes(b.dataset.action)||"
            "(phase==='native_open'&&b.dataset.action==='operation-before'&&"
            "!((d.kind==='html'&&(d.cycle===1||d.cycle===2))||(d.kind==='office'&&d.cycle===1)))||"
            "(phase==='finished'&&b.dataset.action==='quit-start'&&"
            "!(d.cycle===10&&d.kind==='office'))||"
            "(phase==='finished'&&b.dataset.action==='open'&&"
            "d.cycle===10&&d.kind==='office');}"
            "document.querySelectorAll('button').forEach(b=>b.onclick=async()=>{b.disabled=true;"
            "try{const response=await fetch('/action',{method:'POST',headers:{'Content-Type':'application/json','X-Panel-CSRF':token},"
            "body:JSON.stringify({action:b.dataset.action})});await refresh();"
            "document.querySelector('#error').textContent=response.ok?'':'request_failed';}"
            "catch{document.querySelector('#error').textContent='request_failed';}});"
            "refresh();</script></main>")


def create_server(panel):
    class LoopbackServer(http.server.ThreadingHTTPServer):
        daemon_threads = True

    server = LoopbackServer(("127.0.0.1", 0), PanelHandler)
    server.panel = panel
    server.port = server.server_port
    return server


def parser():
    root = argparse.ArgumentParser(description=__doc__)
    root.add_argument("--run-directory", type=Path, required=True)
    root.add_argument("--office-input", type=Path, required=True)
    root.add_argument("--output-directory", type=Path, required=True)
    return root


def main(argv=None):
    args = parser().parse_args(argv)
    panel = DocumentHostPanel(args.run_directory, args.office_input, args.output_directory)
    # WHY: watcher readiness markers stay available on stderr while stdout remains a single launch record.
    panel.watcher.__dict__["print"] = lambda *args, **kwargs: builtins.print(*args, file=sys.stderr, **kwargs)
    server = create_server(panel)
    print(json.dumps({"url": f"http://127.0.0.1:{server.port}/"}, separators=(",", ":")), flush=True)
    server.serve_forever()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
