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
from types import SimpleNamespace
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


class WorkerSnapshotBoundaryTests(unittest.TestCase):
    """OS読み取り境界を制御するunit。actual OS lifecycle回帰とは区別する。"""

    def observer(self):
        observer = object.__new__(processes.ProcessObserver)
        observer.main_pid = 1
        observer.main = Path("/main")
        observer.worker = Path("/worker")
        observer.main_identity = self.identity(1)
        observer.hashes = {observer.worker: "fixed-worker-hash"}
        observer.owned = {}
        observer.unchanged = lambda: None
        return observer

    def identity(self, pid):
        return {"pid": pid, "executable": str(Path("/main" if pid == 1 else "/worker")),
                "start_identity": f"actual-start-{pid}"}

    def measurement(self, pid, identity, **_options):
        return {"status": "ok", "pid": pid, "identity": identity,
                "observed_at": "2026-11-03T00:00:00+00:00", "rss_bytes": 4096}

    def verified(self, observer, pid):
        identity = self.identity(pid)
        observer.owned[pid] = identity
        if not hasattr(observer, "_verified_workers"):
            observer._verified_workers = {}
        observer._verified_workers[pid] = {"identity": identity,
            "executable_sha256": observer.hashes[observer.worker], "observed_at": "prior-real-sample"}

    def ps(self, *pids):
        return SimpleNamespace(stdout="1 0 0.0\n"+"".join(f"{pid} 1 0.0\n" for pid in pids))

    def test_known_worker_esrch_requires_prior_success_and_fresh_absence(self):
        observer = self.observer()
        self.verified(observer, 2)
        def identity(pid):
            if pid == 2:
                raise ProcessLookupError(errno.ESRCH, "gone")
            return self.identity(pid)
        with patch.object(processes.subprocess, "run", side_effect=[self.ps(2), self.ps()]), \
                patch.object(processes, "process_identity", side_effect=identity):
            result = observer.snapshot()
        self.assertEqual(result["children"], [])
        retired = result["terminated_children"][0]
        self.assertEqual(retired["pid"], 2)
        self.assertTrue(retired["pid_absent"])
        self.assertIn("2 1", retired["before_ps"])
        self.assertNotIn("2 1", retired["after_ps"])
        self.assertEqual(retired["previous_verified"], observer._verified_workers[2])

    def test_refresh_observes_new_descendant_and_retained_owned(self):
        observer = self.observer()
        self.verified(observer, 2)
        self.verified(observer, 4)
        def identity(pid):
            if pid == 2:
                raise ProcessLookupError(errno.ESRCH, "gone")
            return self.identity(pid)
        fresh = SimpleNamespace(stdout="1 0 0\n3 1 0\n4 99 0\n")
        with patch.object(processes.subprocess, "run", side_effect=[self.ps(2), fresh]), \
                patch.object(processes, "process_identity", side_effect=identity), \
                patch.object(processes, "sample", side_effect=self.measurement):
            result = observer.snapshot()
        self.assertEqual([child["pid"] for child in result["children"]], [3, 4])

    def test_typed_memory_esrch_uses_same_proof_only_for_verified_worker(self):
        for known, remains, error in ((True, False, ProcessLookupError(errno.ESRCH, "gone")),
                                     (True, True, ProcessLookupError(errno.ESRCH, "live")),
                                     (True, False, ProcessLookupError(errno.EIO, "not ESRCH")),
                                     (False, False, None)):
            with self.subTest(known=known, remains=remains, error=error):
                observer = self.observer()
                if known:
                    self.verified(observer, 2)
                def measurement(pid, identity, **options):
                    self.assertEqual(options.get("raise_on_missing_process", False), known)
                    if known:
                        raise error
                    return self.measurement(pid, identity) | {"status": "error", "rss_bytes": None,
                                                             "error": "ProcessLookupError: untrusted text"}
                with patch.object(processes.subprocess, "run", side_effect=[self.ps(2), self.ps(2) if remains else self.ps()]), \
                        patch.object(processes, "process_identity", side_effect=self.identity), \
                        patch.object(processes, "sample", side_effect=measurement):
                    if known and (remains or error.errno != errno.ESRCH):
                        with self.assertRaisesRegex(HostEvidenceError, "without proven exit"):
                            observer.snapshot()
                    else:
                        result = observer.snapshot()
                        if known:
                            self.assertEqual(result["children"], [])
                            self.assertEqual(result["terminated_children"][0]["observation_stage"], "memory")
                        else:
                            self.assertEqual(result["children"][0]["status"], "memory_unobserved")
                            self.assertNotIn("terminated_children", result)

    def test_other_invalid_child_cannot_be_erased_by_memory_esrch_refresh(self):
        for retiring, invalid in ((2, 3), (3, 2)):
            for failure in ("unknown_esrch", "permission", "foreign", "memory", "memory_identity"):
                with self.subTest(retiring=retiring, invalid=invalid, failure=failure):
                    observer = self.observer()
                    self.verified(observer, retiring)
                    def identity(pid):
                        if pid == invalid and failure in ("unknown_esrch", "permission"):
                            raise (ProcessLookupError(errno.ESRCH, "gone") if failure == "unknown_esrch"
                                   else PermissionError(errno.EACCES, "denied"))
                        value = self.identity(pid)
                        return value | {"executable": "/foreign"} if pid == invalid and failure == "foreign" else value
                    def measurement(pid, value, **_options):
                        if pid == retiring:
                            raise ProcessLookupError(errno.ESRCH, "gone during memory")
                        result = self.measurement(pid, value)
                        if failure == "memory":
                            return result | {"status": "error", "rss_bytes": None, "error": "PermissionError"}
                        if failure == "memory_identity":
                            return result | {"identity": value | {"start_identity": "wrong"}}
                        return result
                    with patch.object(processes.subprocess, "run", side_effect=[self.ps(2, 3), self.ps()]), \
                            patch.object(processes, "process_identity", side_effect=identity), \
                            patch.object(processes, "sample", side_effect=measurement):
                        with self.assertRaisesRegex(HostEvidenceError, "cannot be discarded"):
                            observer.snapshot()

    def test_non_esrch_memory_errors_never_refresh(self):
        for failure in ("PermissionError", "OSError", "RuntimeError"):
            with self.subTest(failure=failure):
                observer = self.observer()
                self.verified(observer, 2)
                measurement = self.measurement(2, self.identity(2)) | {"status": "error", "rss_bytes": None,
                                                                       "error": failure}
                with patch.object(processes.subprocess, "run", return_value=self.ps(2)) as table_read, \
                        patch.object(processes, "process_identity", side_effect=self.identity), \
                        patch.object(processes, "sample", return_value=measurement):
                    result = observer.snapshot()
                self.assertEqual(result["children"][0]["status"], "memory_unobserved")
                self.assertNotIn("terminated_children", result)
                self.assertEqual(table_read.call_count, 1)

    def test_unverified_or_live_worker_esrch_is_not_retired(self):
        for known, remains, error in [(False, False, ProcessLookupError(errno.ESRCH, "gone")),
                                     (True, True, ProcessLookupError(errno.ESRCH, "still live")),
                                     (True, False, PermissionError(errno.EACCES, "denied")),
                                     (True, False, OSError(errno.EIO, "I/O failure"))]:
            with self.subTest(known=known, remains=remains, error=type(error).__name__):
                observer = self.observer()
                observer.owned[2] = self.identity(2)
                if known:
                    self.verified(observer, 2)
                def identity(pid):
                    if pid == 2:
                        raise error
                    return self.identity(pid)
                with patch.object(processes.subprocess, "run", side_effect=[self.ps(2), self.ps(2) if remains else self.ps()]), \
                        patch.object(processes, "process_identity", side_effect=identity):
                    result = observer.snapshot()
                self.assertEqual(result["children"][0]["status"], "identity_unobserved")
                self.assertNotIn("terminated_children", result)

    def test_pid_reuse_and_main_identity_change_are_rejected(self):
        for changed_pid in (1, 2):
            with self.subTest(pid=changed_pid):
                observer = self.observer()
                self.verified(observer, 2)
                def identity(pid):
                    result = self.identity(pid)
                    return result | {"start_identity": "reused"} if pid == changed_pid else result
                with patch.object(processes.subprocess, "run", return_value=self.ps(2)), \
                        patch.object(processes, "process_identity", side_effect=identity):
                    with self.assertRaises(HostEvidenceError):
                        observer.snapshot()

    def test_failed_memory_cannot_establish_verified_worker_history(self):
        observer = self.observer()
        with patch.object(processes.subprocess, "run", return_value=self.ps(2)), \
                patch.object(processes, "process_identity", side_effect=self.identity), \
                patch.object(processes, "sample", return_value={"status": "error", "rss_bytes": None}):
            first = observer.snapshot()
        self.assertEqual(first["children"][0]["status"], "memory_unobserved")
        self.assertIn(2, observer.owned)
        self.assertNotIn(2, getattr(observer, "_verified_workers", {}))
        def identity(pid):
            if pid == 2:
                raise ProcessLookupError(errno.ESRCH, "gone")
            return self.identity(pid)
        with patch.object(processes.subprocess, "run", return_value=self.ps(2)) as table_read, \
                patch.object(processes, "process_identity", side_effect=identity):
            second = observer.snapshot()
        self.assertEqual(second["children"][0]["status"], "identity_unobserved")
        self.assertNotIn("terminated_children", second)
        self.assertEqual(table_read.call_count, 1)

    def test_invalid_other_child_cannot_disappear_across_refresh_in_either_pid_order(self):
        for retiring, invalid in ((2, 3), (3, 2)):
            for failure in ("unknown_esrch", "permission", "foreign", "memory", "memory_identity"):
                with self.subTest(retiring=retiring, invalid=invalid, failure=failure):
                    observer = self.observer()
                    self.verified(observer, retiring)
                    inspected = []
                    def identity(pid):
                        inspected.append(pid)
                        if pid == retiring or (pid == invalid and failure == "unknown_esrch"):
                            raise ProcessLookupError(errno.ESRCH, "gone")
                        if pid == invalid and failure == "permission":
                            raise PermissionError(errno.EACCES, "denied")
                        value = self.identity(pid)
                        return value | {"executable": "/foreign"} if pid == invalid and failure == "foreign" else value
                    def measurement(pid, value, **_options):
                        result = self.measurement(pid, value)
                        if pid == invalid and failure == "memory":
                            return result | {"status": "error", "rss_bytes": None}
                        if pid == invalid and failure == "memory_identity":
                            return result | {"identity": value | {"start_identity": "wrong"}}
                        return result
                    with patch.object(processes.subprocess, "run", side_effect=[self.ps(2, 3), self.ps()]), \
                            patch.object(processes, "process_identity", side_effect=identity), \
                            patch.object(processes, "sample", side_effect=measurement):
                        with self.assertRaisesRegex(HostEvidenceError, "cannot be discarded"):
                            observer.snapshot()
                    self.assertIn(invalid, inspected)

    def test_refresh_is_bounded_and_later_invalid_child_still_fails(self):
        for final_invalid in (False, True):
            with self.subTest(final_invalid=final_invalid):
                observer = self.observer()
                for pid in (2, 3, 4):
                    self.verified(observer, pid)
                def identity(pid):
                    if pid != 1:
                        raise ProcessLookupError(errno.ESRCH, "gone")
                    return self.identity(pid)
                tables = [self.ps(2), self.ps(3), self.ps(4, 5) if final_invalid else self.ps(4), self.ps()]
                with patch.object(processes.subprocess, "run", side_effect=tables) as table_read, \
                        patch.object(processes, "process_identity", side_effect=identity):
                    with self.assertRaisesRegex(HostEvidenceError, "cannot be discarded" if final_invalid else "three attempts"):
                        observer.snapshot()
                self.assertEqual(table_read.call_count, 4)


@unittest.skipUnless(platform.system() == "Darwin", "actual Darwin worker identity/libproc lifecycle")
class ActualWorkerSnapshotTests(unittest.TestCase):
    def test_real_worker_exits_between_ps_and_identity_then_fresh_child_is_observed(self):
        self.actual_worker_exit("identity")

    def test_real_worker_exits_before_memory_read_then_fresh_memory_is_observed(self):
        self.actual_worker_exit("memory_before")

    def test_real_worker_exits_after_memory_read_then_fresh_memory_is_observed(self):
        self.actual_worker_exit("memory_after")

    def actual_worker_exit(self, stage):
        # WHY: unitの境界hookで終了順序だけ決め、PID/identity/RSS/再取得psは実OSを使う。
        script = '''
import subprocess, sys
def spawn():
    child = subprocess.Popen([sys.executable,"-B","-c","import sys; sys.stdin.readline()"],stdin=subprocess.PIPE)
    print(child.pid,flush=True)
    return child
child=spawn()
for command in sys.stdin:
    if command.strip()=="replace":
        child.stdin.write(b"exit\\n"); child.stdin.flush()
        print(child.wait(timeout=5),flush=True)
        child=spawn()
    else:
        child.stdin.write(b"exit\\n"); child.stdin.flush()
        child.wait(timeout=5)
        break
'''
        app = subprocess.Popen([sys.executable, "-B", "-c", script], stdin=subprocess.PIPE,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, encoding="utf-8")
        try:
            worker_pid = int(app.stdout.readline())
            executable = Path(sampler.process_identity(app.pid)["executable"])
            observer = processes.ProcessObserver(app.pid, executable, executable)
            initial = observer.snapshot()
            self.assertEqual([row["pid"] for row in initial["children"]], [worker_pid])
            self.assertEqual(initial["children"][0]["memory"]["status"], "ok")
            original_identity = processes.process_identity
            original_read = sampler._read_memory
            replaced = []
            def replace_worker():
                if not replaced:
                    app.stdin.write("replace\n"); app.stdin.flush()
                    self.assertEqual(app.stdout.readline().strip(), "0")
                    replaced.append(int(app.stdout.readline()))
            def identity(pid):
                if pid == worker_pid:
                    replace_worker()
                return original_identity(pid)
            def read_memory(result, pid, system):
                if pid == worker_pid and stage == "memory_before":
                    replace_worker()
                original_read(result, pid, system)
                if pid == worker_pid and stage == "memory_after":
                    replace_worker()
            boundary = (patch.object(processes, "process_identity", side_effect=identity) if stage == "identity"
                        else patch.object(sampler, "_read_memory", side_effect=read_memory))
            with boundary:
                result = observer.snapshot()
            self.assertEqual([row["pid"] for row in result["children"]], replaced)
            self.assertGreater(result["children"][0]["memory"]["rss_bytes"], 0)
            self.assertGreater(result["children"][0]["memory"]["physical_footprint_bytes"], 0)
            self.assertTrue(result["terminated_children"][0]["pid_absent"])
            self.assertEqual(result["terminated_children"][0]["pid"], worker_pid)
            self.assertEqual(result["terminated_children"][0]["observation_stage"],
                             "identity" if stage == "identity" else "memory")
            app.stdin.write("quit\n"); app.stdin.flush()
            self.assertEqual(app.wait(timeout=5), 0)
            self.assertEqual(observer.finish()["children"], [])
        finally:
            if app.poll() is None:
                app.terminate()
            app.communicate(timeout=5)


if __name__ == "__main__":
    unittest.main()
