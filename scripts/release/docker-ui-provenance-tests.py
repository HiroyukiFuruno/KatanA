#!/usr/bin/env python3
"""Docker-only integration checks using real Linux processes, without process mocks."""

import hashlib
import importlib.util
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("docker_ui_functional", Path(__file__).with_name("docker-ui-functional.py"))
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class ProvenanceTests(unittest.TestCase):
    def setUp(self):
        retained = ROOT / "tmp" / "trash"
        retained.mkdir(parents=True, exist_ok=True)
        self.root = Path(tempfile.mkdtemp(prefix="docker-ui-provenance.", dir=retained))
        package = self.root / "package"
        package.mkdir()
        self.binary = package / "KatanA"
        shutil.copy2(shutil.which("sleep"), self.binary)
        self.digest = hashlib.sha256(self.binary.read_bytes()).hexdigest()
        self.child = subprocess.Popen([str(self.binary), "30"])
        (self.root / "app.pid").write_text(str(self.child.pid))
        MODULE.until("real executable mapping", self.mapped)

    def mapped(self):
        return str(self.binary) in Path(f"/proc/{self.child.pid}/maps").read_text()

    def tearDown(self):
        if self.child.poll() is None:
            self.child.terminate()
        self.child.wait(timeout=5)

    def test_real_mapped_file_and_command_are_verified(self):
        acceptance = MODULE.DesktopAcceptance(self.root, self.digest)
        acceptance.provenance()
        self.assertEqual(acceptance.report["steps"][0]["guest_sha256"], self.digest)
        self.assertFalse(acceptance.report["native_performance"])

    def test_wrong_expected_hash_is_rejected(self):
        acceptance = MODULE.DesktopAcceptance(self.root, "0" * 64)
        with self.assertRaisesRegex(AssertionError, "differs from the input archive"):
            acceptance.provenance()

    def test_same_prefix_does_not_accept_a_different_mapped_path(self):
        self.binary.rename(self.binary.with_name("KatanA-other"))
        shutil.copy2(shutil.which("sleep"), self.binary)
        acceptance = MODULE.DesktopAcceptance(self.root, self.digest)
        with self.assertRaisesRegex(AssertionError, "not mapped"):
            acceptance.provenance()

    def test_mapped_file_without_expected_command_is_rejected(self):
        self.child.terminate()
        self.child.wait(timeout=5)
        self.child = subprocess.Popen([
            "bash", "-c", 'exec -a misleading "$1" 30', "bash", str(self.binary),
        ])
        (self.root / "app.pid").write_text(str(self.child.pid))
        MODULE.until("real renamed argv mapping", self.mapped)
        acceptance = MODULE.DesktopAcceptance(self.root, self.digest)
        with self.assertRaisesRegex(AssertionError, "not in the launch command"):
            acceptance.provenance()


if __name__ == "__main__":
    unittest.main()
