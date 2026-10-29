"""Small real-Docker lifecycle check; touches only katana-issue376-* fixtures."""
import hashlib
from datetime import datetime, timedelta, timezone
import json
import os
from pathlib import Path
import select
import subprocess
import sys
import tempfile
import unittest
import uuid

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/docker/volume-lifecycle.py"
IMAGE = "alpine:latest"
OWNER_LABEL = "io.katana.owner"
PURPOSE_LABEL = "io.katana.purpose"
RUN_LABEL = "io.katana.run-id"
OWNER = "HiroyukiFuruno/KatanA"
STREAMING_EVIDENCE_SIZE_MB = 24


def docker(*args, check=True):
    return subprocess.run(["docker", *args], capture_output=True, text=True, check=check)


class RealDockerVolumeLifecycleTests(unittest.TestCase):
    def setUp(self):
        self.before_volumes = set(docker("volume", "ls", "-q").stdout.splitlines())
        self.temp = tempfile.TemporaryDirectory(prefix="issue376-root-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.run_ids = [f"docker-ui-functional.issue376-{uuid.uuid4().hex[:8]}" for _ in range(8)]
        self.volumes = []
        self.containers = []

    def tearDown(self):
        for container in self.containers:
            state = docker("inspect", "-f", "{{.State.Running}}", container, check=False)
            if state.returncode == 0 and state.stdout.strip() == "true":
                docker("stop", "-t", "2", container, check=False)
            docker("rm", container, check=False)
        for volume in self.volumes:
            docker("volume", "rm", volume, check=False)
        after_volumes = set(docker("volume", "ls", "-q").stdout.splitlines())
        self.assertEqual(after_volumes, self.before_volumes, "fixture cleanup changed a pre-existing volume")

    def make_run(self, run_id, *, exported, old=False):
        x11 = f"katana-{run_id}-x11"
        evidence = f"katana-{run_id}-evidence"
        for name, purpose in ((x11, "ui-x11"), (evidence, "ui-evidence")):
            docker("volume", "create", "--label", f"{OWNER_LABEL}={OWNER}", "--label", f"{PURPOSE_LABEL}={purpose}",
                   "--label", f"{RUN_LABEL}={run_id}", name)
            self.volumes.append(name)
        run_dir = self.root / "tmp" / run_id
        evidence_dir = run_dir / "evidence"
        evidence_dir.mkdir(parents=True)
        (evidence_dir / "diagnostic.txt").write_text("few-byte fixture evidence\n", encoding="utf-8")
        if exported:
            record = subprocess.run(
                [sys.executable, str(SCRIPT), "--root", str(self.root), "record-export", run_id,
                 "--x11-volume", x11, "--evidence-volume", evidence],
                capture_output=True, text=True, check=False,
            )
            self.assertEqual(record.returncode, 0, record.stdout + record.stderr)
            receipt_path = run_dir / "evidence-export-receipt.json"
            receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
            if old:
                receipt["exported_at"] = (datetime.now(timezone.utc) - timedelta(days=31)).isoformat()
                receipt_path.write_text(json.dumps(receipt), encoding="utf-8")
        return x11, evidence

    def inspect(self, run_id):
        result = subprocess.run(
            [sys.executable, str(SCRIPT), "--root", str(self.root), "preview"],
            capture_output=True, text=True, check=False,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        preview = json.loads(result.stdout)
        return next((run for run in preview["ui_runs"] if run["run_id"] == run_id), None)


    def create_labeled_volume(self, name, purpose="ci-cache", owner=OWNER, run_id=None):
        labels = ["--label", f"{OWNER_LABEL}={owner}", "--label", f"{PURPOSE_LABEL}={purpose}"]
        if run_id:
            labels.extend(["--label", f"{RUN_LABEL}={run_id}"])
        docker("volume", "create", *labels, name)
        self.volumes.append(name)
        return name

    def manager(self, *args, input_text=None):
        return subprocess.run(
            [sys.executable, str(SCRIPT), "--root", str(self.root), *args],
            input=input_text, capture_output=True, text=True, check=False,
        )

    def make_producer_run(self, run_id, *, app_running=False, block_export=False,
                          evidence_size_mb=0, remove_observer_tar=False):
        x11, evidence = f"katana-{run_id}-x11", f"katana-{run_id}-evidence"
        for name, purpose in ((x11, "ui-x11"), (evidence, "ui-evidence")):
            self.create_labeled_volume(name, purpose, run_id=run_id)
        run_dir = self.root / "tmp" / run_id
        run_dir.mkdir(parents=True)
        app_name, observer_name = f"katana-{run_id}-app", f"katana-{run_id}-observer"
        mounts = ["--mount", f"type=volume,src={x11},dst=/tmp/.X11-unix",
                  "--mount", f"type=volume,src={evidence},dst=/evidence"]
        observer_id = docker(
            "run", "-d", "--name", observer_name,
            "--label", f"{OWNER_LABEL}={OWNER}", "--label", f"{RUN_LABEL}={run_id}",
            *mounts, IMAGE, "sh", "-c", "echo fixture-evidence > /evidence/diagnostic.txt; sleep 60",
        ).stdout.strip()
        self.containers.append(observer_name)
        if evidence_size_mb:
            docker("exec", observer_id, "dd", "if=/dev/zero", "of=/evidence/large.bin",
                   "bs=1M", f"count={evidence_size_mb}")
        if remove_observer_tar:
            docker("exec", observer_id, "mv", "/bin/tar", "/bin/tar.fixture-disabled")
        app_command = "echo fixture-app > /evidence/app.txt; sleep 60" if app_running else "echo fixture-app > /evidence/app.txt"
        app_id = docker(
            "run", "-d", "--name", app_name,
            "--label", f"{OWNER_LABEL}={OWNER}", "--label", f"{RUN_LABEL}={run_id}",
            *mounts, IMAGE, "sh", "-c", app_command,
        ).stdout.strip()
        self.containers.append(app_name)
        if not app_running:
            self.assertEqual(docker("wait", app_id).stdout.strip(), "0")
        (run_dir / "app.id").write_text(app_id + "\n", encoding="utf-8")
        (run_dir / "observer.id").write_text(observer_id + "\n", encoding="utf-8")
        if block_export:
            (run_dir / "evidence").write_text("block export destination\n", encoding="utf-8")
        return {
            "run_id": run_id, "run_dir": run_dir, "x11": x11, "evidence": evidence,
            "app": app_name, "observer": observer_name,
        }

    def cleanup_producer(self, fixture, initial_status):
        return subprocess.run([
            sys.executable, str(ROOT / "scripts/docker/ui-functional-cleanup.py"),
            "--root", str(self.root), "--run-dir", str(fixture["run_dir"]),
            "--run-id", fixture["run_id"], "--app-name", fixture["app"],
            "--observer-name", fixture["observer"],
            "--app-id-file", str(fixture["run_dir"] / "app.id"),
            "--observer-id-file", str(fixture["run_dir"] / "observer.id"),
            "--x11-volume", fixture["x11"], "--evidence-volume", fixture["evidence"],
            "--initial-status", str(initial_status),
        ], capture_output=True, text=True, check=False)

    def test_cache_preview_and_real_targeted_delete_guards(self):
        target = self.create_labeled_volume(f"katana-issue376-{uuid.uuid4().hex[:8]}-cache")
        shared = self.create_labeled_volume(f"katana-issue376-{uuid.uuid4().hex[:8]}-shared", "shared-ci-cache")
        foreign = self.create_labeled_volume(f"katana-issue376-{uuid.uuid4().hex[:8]}-foreign", owner="Other/Repo")
        preview = self.manager("preview")
        self.assertEqual(preview.returncode, 0, preview.stdout + preview.stderr)
        caches = {item["volume"]: item for item in json.loads(preview.stdout)["ci_caches"]}
        self.assertIn(target, caches)
        self.assertEqual(caches[target]["purpose"], "ci-cache")
        self.assertIn(caches[target]["size_status"], {"available", "unknown"})
        self.assertIn(caches[target]["reference_status"], {"known", "unknown"})
        self.assertEqual(caches[shared]["deletion"], "not_supported")
        self.assertNotIn(foreign, caches)

        shared_delete = self.manager("delete-cache", shared, input_text=shared + "\n")
        self.assertEqual(shared_delete.returncode, 2)
        self.assertEqual(docker("volume", "inspect", shared, check=False).returncode, 0)
        foreign_delete = self.manager("delete-cache", foreign, input_text=foreign + "\n")
        self.assertEqual(foreign_delete.returncode, 2)
        self.assertEqual(docker("volume", "inspect", foreign, check=False).returncode, 0)

        container = f"katana-issue376-{uuid.uuid4().hex[:8]}-cache-ref"
        docker("create", "--name", container, "--mount", f"type=volume,src={target},dst=/cache",
               IMAGE, "sh", "-c", "sleep 60")
        self.containers.append(container)
        blocked = self.manager("delete-cache", target, input_text=target + "\n")
        self.assertEqual(blocked.returncode, 2)
        self.assertEqual(docker("volume", "inspect", target, check=False).returncode, 0)
        docker("rm", container)
        self.containers.remove(container)

        confirmation = subprocess.Popen(
            [sys.executable, str(SCRIPT), "--root", str(self.root), "delete-cache", target],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
        )
        ready, _, _ = select.select([confirmation.stdout], [], [], 10)
        if not ready:
            confirmation.kill()
            confirmation.communicate()
            self.fail("cache deletion prompt did not arrive")
        prompt = os.read(confirmation.stdout.fileno(), 4096).decode()
        self.assertIn(target, prompt)
        race_container = f"katana-issue376-{uuid.uuid4().hex[:8]}-cache-race"
        docker("create", "--name", race_container, "--mount", f"type=volume,src={target},dst=/cache",
               IMAGE, "sh", "-c", "sleep 60")
        self.containers.append(race_container)
        confirmation.stdin.write(target + "\n")
        confirmation.stdin.flush()
        stdout, stderr = confirmation.communicate(timeout=15)
        self.assertEqual(confirmation.returncode, 2, stdout + stderr)
        self.assertIn("container_reference", stdout)
        self.assertEqual(docker("volume", "inspect", target, check=False).returncode, 0)
        docker("rm", race_container)
        self.containers.remove(race_container)

        deleted = self.manager("delete-cache", target, input_text=target + "\n")
        self.assertEqual(deleted.returncode, 0, deleted.stdout + deleted.stderr)
        self.volumes.remove(target)
        self.assertEqual(docker("volume", "inspect", target, check=False).returncode, 1)

    def test_actual_producer_cleanup_success_and_blocked_exit(self):
        for index, status in enumerate((0, 2)):
            fixture = self.make_producer_run(self.run_ids[3 + index])
            cleanup = self.cleanup_producer(fixture, status)
            self.assertEqual(cleanup.returncode, status, cleanup.stdout + cleanup.stderr)
            self.assertEqual((fixture["run_dir"] / "driver-exit.txt").read_text().strip(), str(status))
            self.assertTrue((fixture["run_dir"] / "evidence-export-receipt.json").is_file())
            self.assertTrue((fixture["run_dir"] / "evidence/diagnostic.txt").is_file())
            self.assertEqual(docker("volume", "inspect", fixture["x11"], check=False).returncode, 1)
            self.assertEqual(docker("volume", "inspect", fixture["evidence"], check=False).returncode, 1)

    def test_actual_producer_export_streams_moderate_file_with_exact_hash(self):
        fixture = self.make_producer_run(
            self.run_ids[7], evidence_size_mb=STREAMING_EVIDENCE_SIZE_MB
        )
        cleanup = self.cleanup_producer(fixture, 0)
        self.assertEqual(cleanup.returncode, 0, cleanup.stdout + cleanup.stderr)
        exported_file = fixture["run_dir"] / "evidence" / "large.bin"
        self.assertEqual(exported_file.stat().st_size, STREAMING_EVIDENCE_SIZE_MB * 1024 * 1024)
        actual_hash = hashlib.sha256()
        with exported_file.open("rb") as source:
            for chunk in iter(lambda: source.read(1024 * 1024), b""):
                actual_hash.update(chunk)
        expected = hashlib.sha256()
        for _ in range(STREAMING_EVIDENCE_SIZE_MB):
            expected.update(b"\0" * (1024 * 1024))
        self.assertEqual(actual_hash.digest(), expected.digest())

    def test_actual_docker_archive_producer_failure_preserves_resources(self):
        fixture = self.make_producer_run(self.run_ids[0], remove_observer_tar=True)
        cleanup = self.cleanup_producer(fixture, 0)
        self.assertEqual(cleanup.returncode, 1, cleanup.stdout + cleanup.stderr)
        self.assertIn("functional evidence export failed", cleanup.stderr)
        self.assertEqual((fixture["run_dir"] / "driver-exit.txt").read_text().strip(), "1")
        self.assertFalse((fixture["run_dir"] / "evidence-export-receipt.json").exists())
        self.assertEqual(docker("volume", "inspect", fixture["x11"], check=False).returncode, 0)
        self.assertEqual(docker("volume", "inspect", fixture["evidence"], check=False).returncode, 0)

    def test_actual_producer_forced_stop_preserves_resources_and_diagnostics(self):
        fixture = self.make_producer_run(self.run_ids[5], app_running=True)
        cleanup = self.cleanup_producer(fixture, 0)
        self.assertEqual(cleanup.returncode, 1, cleanup.stdout + cleanup.stderr)
        self.assertTrue((fixture["run_dir"] / "forced-cleanup.txt").is_file())
        self.assertEqual((fixture["run_dir"] / "driver-exit.txt").read_text().strip(), "1")
        self.assertTrue((fixture["run_dir"] / "evidence-export-receipt.json").is_file())
        self.assertEqual(docker("volume", "inspect", fixture["x11"], check=False).returncode, 0)
        self.assertEqual(docker("volume", "inspect", fixture["evidence"], check=False).returncode, 0)
        self.assertEqual(docker("inspect", "-f", "{{.State.Running}}", fixture["app"]).stdout.strip(), "false")

    def test_actual_producer_export_failure_preserves_unexported_evidence(self):
        fixture = self.make_producer_run(self.run_ids[6], block_export=True)
        cleanup = self.cleanup_producer(fixture, 0)
        self.assertEqual(cleanup.returncode, 1, cleanup.stdout + cleanup.stderr)
        self.assertEqual((fixture["run_dir"] / "driver-exit.txt").read_text().strip(), "1")
        self.assertFalse((fixture["run_dir"] / "evidence-export-receipt.json").exists())
        self.assertEqual(docker("volume", "inspect", fixture["x11"], check=False).returncode, 0)
        self.assertEqual(docker("volume", "inspect", fixture["evidence"], check=False).returncode, 0)
        self.assertEqual(docker("inspect", "-f", "{{.State.Running}}", fixture["observer"]).stdout.strip(), "false")

    def test_stopped_running_export_failure_and_rerun_safety(self):
        failed_id, exported_id, rerun_id = self.run_ids[:3]
        self.make_run(failed_id, exported=False)
        self.make_run(exported_id, exported=True, old=True)
        self.make_run(rerun_id, exported=True, old=False)

        failed = self.inspect(failed_id)
        self.assertEqual((failed["status"], failed["reason"]), ("retained", "evidence_not_exported"))
        fresh = self.inspect(rerun_id)
        self.assertEqual((fresh["status"], fresh["reason"]), ("retained", "within_evidence_retention"))

        x11, evidence = f"katana-{exported_id}-x11", f"katana-{exported_id}-evidence"
        container = f"katana-issue376-{uuid.uuid4().hex[:8]}-reference"
        docker("create", "--name", container, "--label", f"{OWNER_LABEL}={OWNER}",
               "--mount", f"type=volume,source={x11},target=/x11",
               "--mount", f"type=volume,source={evidence},target=/evidence",
               IMAGE, "sh", "-c", "sleep 60")
        self.containers.append(container)
        stopped = self.inspect(exported_id)
        self.assertEqual((stopped["status"], stopped["reason"]), ("retained", "container_reference"))
        docker("start", container)
        running = self.inspect(exported_id)
        self.assertEqual((running["status"], running["reason"]), ("retained", "container_reference"))
        docker("stop", "-t", "2", container)
        stopped_again = self.inspect(exported_id)
        self.assertEqual((stopped_again["status"], stopped_again["reason"]), ("retained", "container_reference"))
        docker("rm", container)
        self.containers.remove(container)

        eligible = self.inspect(exported_id)
        self.assertEqual((eligible["status"], eligible["reason"]), ("eligible", "exported_and_retention_elapsed"))
        delete = subprocess.run(
            [sys.executable, str(SCRIPT), "--root", str(self.root), "delete-ui-run", exported_id],
            capture_output=True, text=True, check=False,
        )
        self.assertEqual(delete.returncode, 0, delete.stdout + delete.stderr)
        self.volumes.remove(x11)
        self.volumes.remove(evidence)
        self.assertEqual(docker("volume", "inspect", x11, check=False).returncode, 1)
        self.assertEqual(docker("volume", "inspect", evidence, check=False).returncode, 1)
        self.assertIsNotNone(self.inspect(failed_id), "failed unexported evidence was removed")
        self.assertIsNotNone(self.inspect(rerun_id), "new rerun resources were removed")


if __name__ == "__main__":
    unittest.main()
