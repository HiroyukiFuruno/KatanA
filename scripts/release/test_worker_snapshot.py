"""Verify the immutable worker snapshot entry point with real files."""

import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import tempfile
import unittest

from native_test_run_identity import file_digest
from worker_snapshot import snapshot_worker


ROOT = Path(__file__).resolve().parents[2]
HELPER = ROOT / "scripts/release/worker_snapshot.py"


class WorkerSnapshotTests(unittest.TestCase):
    def test_subprocess_snapshot_survives_source_replacement_and_keeps_exe_name(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name).resolve()
            source = root / "kdv-office-worker.exe"
            destination = root / "tmp" / "worker.exe"
            source.write_bytes(b"original")
            source.chmod(0o755)
            result = subprocess.run(
                [sys.executable, str(HELPER), str(source), "--directory", "tmp"],
                cwd=root, check=False,
                capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            destination = Path(result.stdout.strip())
            self.assertTrue(destination.is_absolute())
            digest = file_digest(destination)
            source.write_bytes(b"coverage replacement")
            self.assertEqual(file_digest(destination), digest)
            self.assertEqual(destination.name, "kdv-office-worker.exe")
            self.assertTrue(os.access(destination, os.X_OK))

    def test_cli_rejects_symlinked_directory_before_mkdtemp(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name).resolve()
            source = root / "worker.exe"
            source.write_bytes(b"worker")
            source.chmod(0o755)
            outside = root / "outside"
            outside.mkdir()
            (root / "linked").symlink_to(outside, target_is_directory=True)
            result = subprocess.run(
                [sys.executable, str(HELPER), str(source), "--directory", str(root / "linked")],
                check=False, capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(list(outside.iterdir()), [])

    def test_alias_nonexec_and_existing_destination_are_rejected(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name).resolve()
            source, destination = root / "worker", root / "snapshot"
            source.write_bytes(b"worker")
            with self.assertRaises(ValueError):
                snapshot_worker(source, destination)
            source.chmod(0o755)
            alias = root / "alias"
            alias.symlink_to(source)
            with self.assertRaises(ValueError):
                snapshot_worker(alias, destination)
            destination.write_bytes(b"existing")
            with self.assertRaises(FileExistsError):
                snapshot_worker(source, destination)

    def test_symlinked_destination_parent_is_rejected(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name).resolve()
            source = root / "worker"
            source.write_bytes(b"worker")
            source.chmod(0o755)
            real_parent = root / "real"
            real_parent.mkdir()
            (root / "linked").symlink_to(real_parent, target_is_directory=True)
            with self.assertRaises(ValueError):
                snapshot_worker(source, root / "linked/snapshot")


class CoverageWiringTests(unittest.TestCase):
    def test_snapshot_is_wired_before_all_four_selectors(self):
        source = (ROOT / "scripts/ci/coverage.sh").read_text()
        build = source.index("cargo build -p katana-ui --bin kdv-office-worker")
        snapshot = source.index("worker_snapshot.py")
        phases = [source.index(f"llvm_cov {name}") for name in
                  ("workspace-lib-bin", "workspace-integration", "ui-parallel", "ui-serial")]
        self.assertLess(build, snapshot)
        self.assertLess(snapshot, min(phases))
        self.assertEqual(source.count("llvm_cov "), 4)
        self.assertIn('--directory "$COVERAGE_TARGET_DIR/tmp"', source)
        self.assertNotIn('--name "$OFFICE_WORKER_NAME"', source)

    @unittest.skipUnless(shutil.which("zsh"), "coverage formal runner requires zsh")
    def test_actual_snapshot_assignment_stops_before_child_marker(self):
        source = (ROOT / "scripts/ci/coverage.sh").read_text()
        start = source.index("OFFICE_WORKER_SOURCE=")
        end = source.index('info "Running workspace lib/bin tests', start)
        snippet = source[start:end]
        self.assertIn("KATANA_KDV_OFFICE_WORKER=$(python3", snippet)
        self.assertNotIn("export KATANA_KDV_OFFICE_WORKER=$(", snippet)
        with tempfile.TemporaryDirectory() as name:
            root = Path(name).resolve()
            marker = root / "marker"
            command = "\n".join((
                "set -euo pipefail",
                f"COVERAGE_TARGET_DIR={shlex.quote(str(root / 'target'))}",
                "OFFICE_WORKER_NAME=kdv-office-worker",
                snippet,
                f"touch {shlex.quote(str(marker))}",
            ))
            result = subprocess.run(["zsh", "-c", command], cwd=ROOT,
                                    check=False, capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("FileNotFoundError", result.stderr)
            self.assertIn("kdv-office-worker", result.stderr)
            self.assertFalse(marker.exists())


if __name__ == "__main__":
    unittest.main()
