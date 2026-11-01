"""Exercise test-profile isolation with real child processes and real Cargo."""

import hashlib
import os
from pathlib import Path
import queue
import signal
import subprocess
import sys
import tempfile
import threading
import unittest

ROOT = Path(__file__).resolve().parents[2]
WRAPPER = ROOT / "scripts/ci/isolated-test-config.py"


class IsolatedTestConfig(unittest.TestCase):
    def run_child(self, original, code, *args):
        environment = os.environ.copy() | {"KATANA_CONFIG_DIR": str(original)}
        return subprocess.run([sys.executable, str(WRAPPER), "--", sys.executable, "-c", code, *args],
                              cwd=ROOT, env=environment, capture_output=True, text=True, timeout=60)

    def test_explicit_existing_profile_is_not_used_and_exit_is_preserved(self):
        with tempfile.TemporaryDirectory() as directory:
            original = Path(directory) / "original"
            original.mkdir()
            sentinel = original / "workspace.json"
            sentinel.write_text('{"sentinel":"unchanged"}')
            before = hashlib.sha256(sentinel.read_bytes()).digest()
            result = self.run_child(original,
                                    "import os,pathlib,sys; p=pathlib.Path(os.environ['KATANA_CONFIG_DIR']); "
                                    "(p/'workspace.json').write_text('test'); print(p); sys.exit(7)")
            self.assertEqual(7, result.returncode, result.stderr)
            isolated = Path(result.stdout.strip())
            self.assertNotEqual(original, isolated)
            self.assertEqual(before, hashlib.sha256(sentinel.read_bytes()).digest())
            if os.name == "posix":
                self.assertFalse(isolated.exists())
            else:
                self.assertTrue(isolated.is_dir())

    @unittest.skipUnless(os.name == "posix", "POSIX process-group lifetime")
    def test_descendant_keeps_profile_until_exit(self):
        with tempfile.TemporaryDirectory() as directory:
            original = Path(directory) / "original"
            original.mkdir()
            code = ("import os,subprocess,sys; "
                    "subprocess.Popen([sys.executable,'-c',"
                    "\"import os,sys; print(os.environ['KATANA_CONFIG_DIR'],flush=True); sys.stdin.readline()\"])")
            env = os.environ.copy() | {"KATANA_CONFIG_DIR": str(original)}
            process = subprocess.Popen([sys.executable, str(WRAPPER), "--", sys.executable, "-c", code],
                                       cwd=ROOT, env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                       stderr=subprocess.PIPE, text=True)
            lines = queue.Queue()
            threading.Thread(target=lambda: lines.put(process.stdout.readline()), daemon=True).start()
            try:
                isolated = Path(lines.get(timeout=20).strip())
                self.assertTrue(isolated.is_dir())
                self.assertIsNone(process.poll())
                process.stdin.write("continue\n")
                process.stdin.flush()
                out, err = process.communicate(timeout=20)
                self.assertEqual(0, process.returncode, out + err)
                self.assertFalse(isolated.exists())
            finally:
                if process.poll() is None:
                    process.terminate()
                    process.communicate(timeout=20)

    @unittest.skipUnless(os.name == "posix", "POSIX signal forwarding")
    def test_signal_reaps_group_before_profile_cleanup(self):
        with tempfile.TemporaryDirectory() as directory:
            env = os.environ.copy() | {"KATANA_CONFIG_DIR": directory}
            code = ("import os,signal,sys; "
                    "signal.signal(signal.SIGTERM,lambda *_:sys.exit(0)); "
                    "print(os.environ['KATANA_CONFIG_DIR'],flush=True); signal.pause()")
            process = subprocess.Popen([sys.executable, str(WRAPPER), "--", sys.executable, "-c", code],
                                       cwd=ROOT, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            try:
                isolated = Path(process.stdout.readline().strip())
                self.assertTrue(isolated.is_dir())
                process.send_signal(signal.SIGTERM)
                out, err = process.communicate(timeout=20)
                self.assertEqual(128 + signal.SIGTERM, process.returncode, out + err)
                self.assertFalse(isolated.exists())
            finally:
                if process.poll() is None:
                    process.terminate()
                    process.communicate(timeout=20)

    @unittest.skipUnless(os.name == "posix", "POSIX child signal status")
    def test_child_signal_exit_uses_shell_status(self):
        with tempfile.TemporaryDirectory() as directory:
            result = self.run_child(Path(directory),
                                    "import os,signal; os.kill(os.getpid(),signal.SIGTERM)")
            self.assertEqual(128 + signal.SIGTERM, result.returncode, result.stderr)

    def test_quality_entrypoints_use_isolation_without_changing_test_scope(self):
        just = (ROOT / "just/tests.just").read_text()
        for name in ("test-ui", "test-commonmark-backend", "test-custom-cursor-cache", "test-pdf-theme",
                     "test-verbose", "test-specific", "test-native-html-diagnostics", "check-light", "nextest-shadow"):
            section = just.split(f"\n{name}:", 1)[1].split("\n\n", 1)[0]
            self.assertIn("scripts/ci/isolated-test-config.py", section, name)
        for file in ("scripts/ci/coverage.sh", "scripts/ci/with-office-test-worker.sh",
                     "scripts/ci/document-fixture-acceptance.sh", ".github/workflows/test-and-build.yml"):
            self.assertIn("scripts/ci/isolated-test-config.py", (ROOT / file).read_text(), file)
        self.assertIn("scripts/ci/isolated-test-config.py", just.split("\ncheck-linux:", 1)[1].split("\n\n", 1)[0])
        self.assertIn("scripts/ci/isolated-test-config.py", (ROOT / "just/setup.just").read_text().split("\nwatch:", 1)[1].split("\n\n", 1)[0])
        workflow = (ROOT / ".github/workflows/test-and-build.yml").read_text()
        for name in ("Measure cold real-font lookup (Windows)", "Test CommonMark cache contracts",
                     "Test patched custom cursor cache", "Test screenshot harness contracts"):
            block = workflow.split(f"      - name: {name}", 1)[1].split("      - name:", 1)[0]
            self.assertIn("scripts/ci/isolated-test-config.py -- cargo test", block, name)
        for name in ("Run tests", "Trace failed HTML startup (macOS)"):
            block = workflow.split(f"      - name: {name}\n", 1)[1].split("      - name:", 1)[0]
            self.assertIn("scripts/ci/with-office-test-worker.sh", block, name)
        worker = (ROOT / "scripts/ci/with-office-test-worker.sh").read_text()
        self.assertIn('isolated-test-config.py -- bash -c "$1"', worker)
        self.assertIn("isolated-test-config.py -- python3 scripts/release/record-test-invocation.py", worker)
        document = (ROOT / "scripts/ci/document-fixture-acceptance.sh").read_text()
        self.assertEqual(document.count("isolated-test-config.py -- cargo test"), 3)
        coverage = (ROOT / "scripts/ci/coverage.sh").read_text()
        self.assertEqual(coverage.count("scripts/ci/isolated-test-config.py"), 2)


if __name__ == "__main__":
    unittest.main()
