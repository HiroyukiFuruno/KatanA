"""実プロセスを使いhost runnerの隔離と残存処理を確認する。"""

import json
import os
import platform
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path

import html_host_process_group as process_group


HELPER = Path(process_group.__file__).resolve()


class ProcessGroupTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        root = Path(self.temp.name)
        self.receipt = root / "receipt.json"
        self.observations = root / "observations.jsonl"

    def invoke(self, action, *args, check=True):
        result = subprocess.run(
            [sys.executable, str(HELPER), action, str(self.receipt), str(self.observations), *map(str, args)],
            capture_output=True, text=True, timeout=5,
        )
        if check:
            self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        return result

    def wait_for(self, condition, timeout=3):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if condition():
                return
            time.sleep(0.02)
        self.fail("process lifecycle condition was not reached")

    def launch(self, code):
        return subprocess.Popen(
            [sys.executable, str(HELPER), "launch", str(self.receipt), str(self.observations), "--",
             sys.executable, "-c", code],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        )

    @staticmethod
    def stop_process(process):
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=2)

    def test_orphaned_owned_grandchild_fails_and_is_cleaned(self):
        pid_file = Path(self.temp.name) / "grandchild.pid"
        continue_file = Path(self.temp.name) / "continue"
        code = (
            "import subprocess,sys; "
            f"p=subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)'],"
            "stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL); "
            f"open({str(pid_file)!r},'w').write(str(p.pid)); "
            f"exec(\"import os,time\\nwhile not os.path.exists({str(continue_file)!r}): time.sleep(0.01)\")"
        )
        runner = self.launch(code)
        self.wait_for(pid_file.exists)
        grandchild_pid = int(pid_file.read_text())
        self.invoke("observe")
        continue_file.touch()
        self.assertEqual(runner.wait(timeout=4), 0)
        self.invoke("observe")
        observed = json.loads(self.observations.read_text().splitlines()[-1])
        self.assertIn(grandchild_pid, [member["pid"] for member in observed["members"]])

        result = self.invoke("finish", check=False)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn(grandchild_pid, json.loads(result.stdout)["residual_observed"])
        pgid = json.loads(self.receipt.read_text())["pgid"]
        self.wait_for(lambda: not any(
            p["pid"] == grandchild_pid and not p["state"].startswith("Z")
            for p in process_group._ps_snapshot(pgid)
        ))

    def test_unwitnessed_group_member_fails_closed_without_signal(self):
        pid_file = Path(self.temp.name) / "unwitnessed.pid"
        code = (
            "import subprocess,sys; "
            "p=subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)'],"
            "stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL); "
            f"open({str(pid_file)!r},'w').write(str(p.pid))"
        )
        runner = self.launch(code)
        self.wait_for(pid_file.exists)
        grandchild_pid = int(pid_file.read_text())
        self.assertEqual(runner.wait(timeout=4), 0)
        result = self.invoke("finish", check=False)
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
        os.kill(grandchild_pid, 0)

        os.kill(grandchild_pid, process_group.signal.SIGTERM)
        pgid = json.loads(self.receipt.read_text())["pgid"]
        self.wait_for(lambda: not any(
            p["pid"] == grandchild_pid and not p["state"].startswith("Z")
            for p in process_group._ps_snapshot(pgid)
        ))

    def test_unrelated_process_survives_owned_cleanup(self):
        unrelated = subprocess.Popen(
            [sys.executable, "-c", "import time; time.sleep(30)"],
            stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
            start_new_session=True,
        )
        self.addCleanup(self.stop_process, unrelated)
        runner = self.launch("pass")
        self.assertEqual(runner.wait(timeout=4), 0)
        self.invoke("observe")
        result = self.invoke("finish", check=False)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIsNone(unrelated.poll())
        os.kill(unrelated.pid, 0)

    def test_successful_runner_has_empty_group(self):
        runner = self.launch("pass")
        self.assertEqual(runner.wait(timeout=4), 0)
        result = self.invoke("observe")
        self.assertEqual(json.loads(result.stdout)["member_count"], 0)
        finish = self.invoke("finish")
        self.assertEqual(json.loads(finish.stdout)["residual_observed"], [])


class ReceiptContractTests(unittest.TestCase):
    def test_boolean_pid_is_rejected_as_invalid_identity(self):
        with tempfile.TemporaryDirectory() as directory:
            receipt_path = Path(directory) / "receipt.json"
            receipt_path.write_text(json.dumps({
                "platform": platform.system(), "leader_pid": True, "pgid": True,
                "launch_parent_pid": os.getpid(), "leader_starttime": "123",
            }), encoding="utf-8")
            with self.assertRaisesRegex(process_group.InspectionError, "invalid process receipt"):
                process_group._load_receipt(receipt_path)

    def test_unwitnessed_group_member_cannot_anchor_ownership(self):
        receipt = {"leader_pid": 100, "leader_starttime": "leader-start"}
        live_member = {"pid": 200, "pgid": 100, "starttime": "new-member-start"}
        self.assertFalse(process_group._has_group_anchor(receipt, [live_member], {}))

    def test_receipt_is_bound_to_original_caller(self):
        with self.assertRaisesRegex(process_group.InspectionError, "launch parent"):
            process_group._validate_launch_parent(os.getpid(), os.getpid() + 1)


class UnsupportedPlatformTests(unittest.TestCase):
    def test_launch_is_explicitly_unsupported(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaisesRegex(process_group.InspectionError, "unsupported platform"):
                process_group.launch(root / "receipt.json", root / "observations.jsonl", [sys.executable])


def load_tests(loader, tests, pattern):
    # POSIXでは実プロセスライフサイクルを検査し、他OSでは明示的な拒否契約を検査する。
    selected = ProcessGroupTests if os.name == "posix" else UnsupportedPlatformTests
    suite = unittest.TestSuite()
    suite.addTests(loader.loadTestsFromTestCase(selected))
    suite.addTests(loader.loadTestsFromTestCase(ReceiptContractTests))
    return suite


if __name__ == "__main__":
    unittest.main()
