"""実OSの短命fixtureと単体境界でcollector終了競合を検査する。native受入ではない。"""

from contextlib import closing, contextmanager
import errno
import importlib.util
import json
import os
from pathlib import Path
import platform
import select
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

import document_host_processes as processes
import process_memory_sampler as sampler
from document_host_io import HostEvidenceError


HERE = Path(__file__).resolve().parent
WORK = Path(os.environ.get("KATANA_OBSERVER_TEST_ROOT", HERE.parents[1] / "tmp"))
COLLECTOR = HERE / "collect-document-host-memory.py"
APP = r'''
import json, os, subprocess, sys
from pathlib import Path
run = Path(sys.argv[1])
child = None
if sys.argv[2] == "child":
    child = subprocess.Popen([sys.executable, "-B", "-c", "import threading; threading.Event().wait()"],
                             stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
(run / "heartbeat.txt").write_text("first-frame\nframe=1\n", encoding="utf-8")
receipt = {"schema":"katana-completed-frame-resource-receipt-v1", "pid":os.getpid(),
           "run_id":run.name, "seq":1,
           "host":dict.fromkeys(("previews","html_surfaces","document_surfaces","frames","textures","cache_entries"),0),
           "kdv":dict.fromkeys(("live_document_sessions","live_spreadsheet_workers","live_worker_workspaces",
           "retained_artifact_bytes","cached_page_count","cached_page_bytes","cached_spreadsheet_cell_count",
           "cached_spreadsheet_cell_bytes"),0), "document_workers_live":0,
           "document_surfaces":[], "html_observations":[]}
(run / "resource-receipts" / "latest.json").write_text(json.dumps(receipt)+"\n", encoding="utf-8")
print(child.pid if child else 0, flush=True)
raise SystemExit(int(sys.stdin.readline()))
'''


@unittest.skipUnless(platform.system() == "Darwin", "actual macOS collector/kqueue")
class ActualCollectorExitTests(unittest.TestCase):
    @contextmanager
    def fixture(self, *, child=False, seconds=3):
        WORK.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="collector-actual-", dir=WORK) as directory:
            run = Path(directory)
            (run / "resource-receipts").mkdir()
            worker = run / "worker-fixture"
            worker.write_bytes(b"fixture worker identity\n")
            architecture = "arm64" if platform.machine() == "arm64" else "x86_64"
            app = subprocess.Popen(["/usr/bin/arch", "-"+architecture, sys.executable, "-B", "-c", APP,
                                    str(run), "child" if child else "none"],
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            observer = None
            owned_pid = 0
            owned_identity = None
            try:
                owned_pid = int(app.stdout.readline())
                if owned_pid:
                    owned_identity = sampler.process_identity(owned_pid)
                executable = Path(sampler.process_identity(app.pid)["executable"])
                (run / "app.pid").write_text(str(app.pid)+"\n", encoding="utf-8")
                (run / "launch.json").write_text(json.dumps({"argv":["/usr/bin/arch", "-"+architecture,
                                                                            str(executable)],
                                                            "target":"macos-"+architecture}), encoding="utf-8")
                observer = subprocess.Popen([sys.executable, "-B", str(COLLECTOR), "--run-directory", str(run),
                                             "--executable", str(executable), "--sidecar-executable", str(worker),
                                             "--timeout-seconds", str(seconds)],
                                            stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                self.wait_for_sample(run, observer, child=child)
                yield run, app, observer, worker
            finally:
                if observer is not None:
                    if observer.poll() is None:
                        observer.terminate()
                    observer.communicate(timeout=5)
                if app.poll() is None:
                    app.terminate()
                app.communicate(timeout=5)
                if owned_pid:
                    try:
                        if sampler.process_identity(owned_pid) == owned_identity:
                            os.kill(owned_pid, signal.SIGTERM)
                    except ProcessLookupError:
                        pass

    def wait_for_sample(self, run, observer, *, child):
        deadline = time.monotonic()+5
        with closing(select.kqueue()) as queue:
            descriptor = os.open(run, os.O_RDONLY)
            try:
                queue.control([select.kevent(descriptor, filter=select.KQ_FILTER_VNODE,
                               flags=select.KQ_EV_ADD | select.KQ_EV_CLEAR, fflags=select.KQ_NOTE_WRITE)], 0, 0)
                while time.monotonic() < deadline:
                    journal = run / "memory-receipt-observations.jsonl"
                    if journal.exists():
                        lines = journal.read_bytes().split(b"\n")[:-1]
                        if lines:
                            row = json.loads(lines[-1])
                            if not child or row["kernel_processes"]["children"]:
                                self.assertEqual(row["memory"]["status"], "ok")
                                self.assertGreater(row["memory"]["rss_bytes"], 0)
                                self.assertGreater(row["memory"]["physical_footprint_bytes"], 0)
                                return
                    if observer.poll() is not None:
                        out, error = observer.communicate()
                        self.fail(out+error)
                    queue.control(None, 1, min(0.02, max(0, deadline-time.monotonic())))
                self.fail("real initial OS sample was not written")
            finally:
                os.close(descriptor)

    def exit_app(self, app, code=0):
        app.stdin.write(str(code)+"\n")
        app.stdin.flush()
        actual = app.wait(timeout=5)
        self.assertEqual(actual, code)
        return actual

    def await_missing(self, observer):
        ready, _, _ = select.select([observer.stderr], [], [], 5)
        self.assertTrue(ready, "collector never reported the real missing main")
        line = observer.stderr.readline()
        self.assertIn("main process disappeared; waiting for actual app.exit", line)

    def assert_terminal_missing(self, run):
        for name in ("kernel-processes.after-exit.json", "app-exit-observed.json"):
            self.assertFalse((run / name).exists(), name)

    def test_delayed_real_normal_exit_marker_completes_without_zero_sample(self):
        with self.fixture() as (run, app, observer, _worker):
            actual = self.exit_app(app)
            self.await_missing(observer)
            self.assertIsNone(observer.poll())
            self.assert_terminal_missing(run)
            (run / "app.exit").write_text(str(actual)+"\n", encoding="utf-8")
            out, error = observer.communicate(timeout=5)
            self.assertEqual(observer.returncode, 0, out+error)
            terminal = json.loads((run / "kernel-processes.after-exit.json").read_text(encoding="utf-8"))
            self.assertFalse(terminal["main_alive"])
            self.assertEqual(terminal["children"], [])
            self.assertEqual(json.loads((run / "app-exit-observed.json").read_text(encoding="utf-8"))["exit_code"], actual)
            for row in map(json.loads, (run / "memory-receipt-observations.jsonl").read_text(encoding="utf-8").splitlines()):
                self.assertEqual(row["memory"]["status"], "ok")
                self.assertGreater(row["memory"]["rss_bytes"], 0)
                self.assertGreater(row["memory"]["physical_footprint_bytes"], 0)

    def test_missing_marker_still_expires_at_existing_collection_deadline(self):
        with self.fixture(seconds=2) as (run, app, observer, _worker):
            self.exit_app(app)
            self.await_missing(observer)
            out, error = observer.communicate(timeout=5)
            self.assertNotEqual(observer.returncode, 0)
            self.assertIn("observation deadline expired", out+error)
            self.assert_terminal_missing(run)

    def test_nonzero_actual_exit_marker_is_rejected(self):
        with self.fixture() as (run, app, observer, _worker):
            actual = self.exit_app(app, 7)
            self.await_missing(observer)
            (run / "app.exit").write_text(str(actual)+"\n", encoding="utf-8")
            out, error = observer.communicate(timeout=5)
            self.assertNotEqual(observer.returncode, 0)
            self.assertIn("abnormal app exit", out+error)
            self.assert_terminal_missing(run)

    def test_known_os_child_remaining_after_main_exit_is_rejected(self):
        with self.fixture(child=True) as (run, app, observer, _worker):
            actual = self.exit_app(app)
            self.await_missing(observer)
            (run / "app.exit").write_text(str(actual)+"\n", encoding="utf-8")
            out, error = observer.communicate(timeout=5)
            self.assertNotEqual(observer.returncode, 0)
            self.assertIn("owned child remains", out+error)
            self.assert_terminal_missing(run)

    def test_changed_worker_file_is_rejected_at_finish(self):
        with self.fixture() as (run, app, observer, worker):
            actual = self.exit_app(app)
            self.await_missing(observer)
            worker.write_bytes(b"changed actual worker bytes\n")
            (run / "app.exit").write_text(str(actual)+"\n", encoding="utf-8")
            out, error = observer.communicate(timeout=5)
            self.assertNotEqual(observer.returncode, 0)
            self.assertIn("actual main/worker inode/stat changed", out+error)
            self.assert_terminal_missing(run)

    def test_zero_marker_cannot_hide_live_main(self):
        with self.fixture() as (run, _app, observer, _worker):
            (run / "app.exit").write_text("0\n", encoding="utf-8")
            out, error = observer.communicate(timeout=5)
            self.assertNotEqual(observer.returncode, 0)
            self.assertIn("normal main exit was not observed", out+error)
            self.assert_terminal_missing(run)

    @unittest.skipIf(hasattr(os, "geteuid") and os.geteuid() == 0, "root bypasses file read permissions")
    def test_actual_permission_error_is_not_treated_as_normal_exit(self):
        with self.fixture() as (run, _app, observer, _worker):
            (run / "resource-receipts/latest.json").chmod(0)
            out, error = observer.communicate(timeout=5)
            self.assertNotEqual(observer.returncode, 0)
            self.assertIn("[Errno 13]", out+error)
            self.assertNotIn("waiting for actual app.exit", error)
            self.assert_terminal_missing(run)


@unittest.skipUnless(platform.system() in ("Darwin", "Linux"), "external PID sampler platform")
class SamplerAndIdentityBoundaryTests(unittest.TestCase):
    @unittest.skipUnless(platform.system() == "Darwin", "Darwin ESRCH process identity")
    def test_wrapped_actual_missing_pid_is_only_raised_when_opted_in(self):
        child = subprocess.Popen([sys.executable, "-B", "-c", "pass"])
        self.assertEqual(child.wait(timeout=5), 0)
        result = sampler.sample(child.pid)
        self.assertEqual(result["status"], "error")
        self.assertIsNone(result["rss_bytes"])
        with self.assertRaises(ProcessLookupError) as context:
            sampler.sample(child.pid, raise_on_missing_process=True)
        self.assertEqual(context.exception.errno, errno.ESRCH)

    def test_unknown_oserror_does_not_enter_exit_transition(self):
        with patch.object(sampler, "process_identity", side_effect=PermissionError(errno.EACCES, "denied")):
            result = sampler.sample(os.getpid(), raise_on_missing_process=True)
        self.assertEqual(result["status"], "error")
        self.assertIsNone(result["rss_bytes"])
        self.assertIn("PermissionError", result["error"])

    def test_identity_mismatch_does_not_enter_exit_transition(self):
        actual = sampler.process_identity(os.getpid())
        result = sampler.sample(os.getpid(), actual | {"start_identity":"not-current"},
                                raise_on_missing_process=True)
        self.assertEqual(result["status"], "error")
        self.assertIn("identity mismatch", result["error"])

    def test_finish_rejects_pid_reuse_instead_of_calling_it_main_absence(self):
        # WHY: OSのPID再利用を強制せず、実current PIDの照合境界だけ単体で差替える。
        actual = sampler.process_identity(os.getpid())
        executable = Path(actual["executable"])
        observer = processes.ProcessObserver(os.getpid(), executable, executable)
        with patch.object(processes, "process_identity", return_value=actual | {"start_identity":"different"}):
            with self.assertRaises(HostEvidenceError):
                observer.finish()

    @unittest.skipUnless(platform.system() == "Darwin", "Darwin ESRCH ps/identity race")
    def test_real_ps_identity_exit_race_remains_typed_and_never_zero(self):
        child = subprocess.Popen([sys.executable, "-B", "-c",
                                  'import sys; print("ready",flush=True); sys.stdin.readline()'],
                                 stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
        try:
            self.assertEqual(child.stdout.readline().strip(), "ready")
            executable = Path(sampler.process_identity(child.pid)["executable"])
            observer = processes.ProcessObserver(child.pid, executable, executable)
            original = subprocess.run

            def ps_then_exit(arguments, **kwargs):
                result = original(arguments, **kwargs)
                self.assertIn(child.pid, processes.process_table(result.stdout))
                child.stdin.write("exit\n")
                child.stdin.flush()
                self.assertEqual(child.wait(timeout=5), 0)
                return result

            with patch.object(processes.subprocess, "run", side_effect=ps_then_exit):
                with self.assertRaises(ProcessLookupError) as context:
                    observer.snapshot()
            self.assertEqual(context.exception.errno, errno.ESRCH)
            self.assertFalse(observer.finish()["main_alive"])
        finally:
            if child.poll() is None:
                child.terminate()
            child.communicate(timeout=5)


if __name__ == "__main__":
    unittest.main()
