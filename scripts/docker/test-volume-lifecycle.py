"""Unit contracts for the isolated Issue #376 volume lifecycle draft."""
from datetime import datetime, timedelta, timezone
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/docker/volume-lifecycle.py"
spec = importlib.util.spec_from_file_location("volume_lifecycle", SCRIPT)
manager = importlib.util.module_from_spec(spec)
spec.loader.exec_module(manager)


class VolumeLifecycleUnitTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.run_id = "docker-ui-functional.issue376-fixture"
        self.x11 = f"katana-{self.run_id}-x11"
        self.evidence = f"katana-{self.run_id}-evidence"
        self.labels = {
            self.x11: {manager.OWNER_LABEL: manager.OWNER, manager.PURPOSE_LABEL: "ui-x11", manager.RUN_LABEL: self.run_id},
            self.evidence: {manager.OWNER_LABEL: manager.OWNER, manager.PURPOSE_LABEL: "ui-evidence", manager.RUN_LABEL: self.run_id},
        }
        self.now = datetime.now(timezone.utc)
        self.inspections = [
            {"Name": name, "Labels": labels}
            for name, labels in self.labels.items()
        ]

    def mock_inspect(self, argv, **_kwargs):
        name = argv[-1]
        value = next((item for item in self.inspections if item["Name"] == name), None)
        if value is None:
            return subprocess.CompletedProcess(argv, 1, "", "missing")
        return subprocess.CompletedProcess(argv, 0, json.dumps([value]), "")

    def make_receipt(self, exported_at: datetime) -> None:
        run_dir = self.root / "tmp" / self.run_id
        evidence_dir = run_dir / "evidence"
        evidence_dir.mkdir(parents=True)
        (evidence_dir / "diagnostic.txt").write_text("fixture-only evidence\n", encoding="utf-8")
        receipt = {
            "schema_version": 1,
            "owner": manager.OWNER,
            "run_id": self.run_id,
            "x11_volume": self.x11,
            "evidence_volume": self.evidence,
            "exported_at": exported_at.isoformat(),
            "evidence_tree_sha256": manager.evidence_tree_sha256(evidence_dir),
        }
        (run_dir / "evidence-export-receipt.json").write_text(json.dumps(receipt), encoding="utf-8")

    def inspect(self, refs=None):
        with patch.object(manager.subprocess, "run", side_effect=self.mock_inspect), \
             patch.object(manager, "volume_refs", return_value=refs or []):
            return manager.inspect_ui_run(self.root, self.run_id, self.now)

    def test_unexported_failure_is_retained(self):
        state = self.inspect()
        self.assertEqual((state["status"], state["reason"]), ("retained", "evidence_not_exported"))

    def test_modified_export_is_retained(self):
        self.make_receipt(self.now - timedelta(days=31))
        evidence_path = self.root / "tmp" / self.run_id / "evidence/diagnostic.txt"
        evidence_path.write_text("changed after export\n", encoding="utf-8")
        state = self.inspect()
        self.assertEqual((state["status"], state["reason"]), ("retained", "export_hash_mismatch"))

    def test_malformed_receipt_timestamp_is_retained(self):
        self.make_receipt(self.now - timedelta(days=31))
        receipt_path = self.root / "tmp" / self.run_id / "evidence-export-receipt.json"
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        receipt["exported_at"] = 123
        receipt_path.write_text(json.dumps(receipt), encoding="utf-8")
        state = self.inspect()
        self.assertEqual((state["status"], state["reason"]), ("retained", "invalid_export_receipt"))

    def test_evidence_within_retention_is_retained(self):
        self.make_receipt(self.now - timedelta(days=29))
        state = self.inspect()
        self.assertEqual((state["status"], state["reason"]), ("retained", "within_evidence_retention"))

    def test_expired_export_is_candidate_only_without_running_or_stopped_refs(self):
        self.make_receipt(self.now - timedelta(days=31))
        blocked = self.inspect(refs=["fixture-container-id"])
        self.assertEqual((blocked["status"], blocked["reason"]), ("retained", "container_reference"))
        eligible = self.inspect()
        self.assertEqual((eligible["status"], eligible["reason"]), ("eligible", "exported_and_retention_elapsed"))

    def test_label_mismatch_never_becomes_a_candidate(self):
        self.inspections[0]["Labels"][manager.OWNER_LABEL] = "another/repository"
        state = self.inspect()
        self.assertEqual((state["status"], state["reason"]), ("retained", "ownership_or_purpose_label_mismatch"))

    def test_invalid_run_id_is_rejected_before_docker_access(self):
        for run_id in ("../outside", "issue376-fixture", "docker-ui-functional../outside"):
            with self.subTest(run_id=run_id):
                state = manager.inspect_ui_run(self.root, run_id, self.now)
                self.assertEqual((state["status"], state["reason"]), ("retained", "invalid_run_id"))

    def test_receipt_uses_the_producer_run_directory_without_duplicating_prefix(self):
        self.make_receipt(self.now - timedelta(days=31))
        producer_directory = self.root / "tmp" / self.run_id
        self.assertTrue((producer_directory / "evidence-export-receipt.json").is_file())
        self.assertFalse((self.root / "tmp" / f"docker-ui-functional.{self.run_id}").exists())
        self.assertEqual(self.inspect()["status"], "eligible")

    def test_cache_preview_exposes_size_references_and_shared_no_delete_policy(self):
        cache = {
            "Name": "katana-cache-fixture",
            "Labels": {manager.PURPOSE_LABEL: "ci-cache"},
            "UsageData": {"Size": 4096, "RefCount": 1},
        }
        preview = manager.cache_preview(cache, [{"id": "fixture-id", "state": "exited"}])
        self.assertEqual(preview["size_bytes"], 4096)
        self.assertEqual(preview["size_status"], "available")
        self.assertEqual(preview["engine_reference_count"], 1)
        self.assertEqual(preview["reference_status"], "known")
        self.assertEqual(preview["referenced_containers"][0]["state"], "exited")

        shared = {
            "Name": "katana-shared-fixture",
            "Labels": {manager.PURPOSE_LABEL: "shared-ci-cache"},
            "UsageData": None,
        }
        unknown = manager.cache_preview(shared, None)
        self.assertIsNone(unknown["size_bytes"])
        self.assertEqual(unknown["size_status"], "unknown")
        self.assertEqual(unknown["reference_status"], "unknown")
        self.assertEqual(unknown["deletion"], "not_supported")

    def test_shared_cache_is_never_a_delete_candidate(self):
        with patch.object(manager, "inspect_volume", return_value={"Name": "shared", "Labels": {
            manager.OWNER_LABEL: manager.OWNER, manager.PURPOSE_LABEL: "shared-ci-cache"
        }}):
            self.assertEqual(manager.cache_candidate("shared"), (False, "ownership_or_cache_label_mismatch"))

    def test_unknown_or_referenced_cache_is_never_deleted(self):
        with patch.object(manager, "inspect_volume", return_value={"Name": "ci-target", "Labels": {}}):
            self.assertEqual(manager.cache_candidate("ci-target"), (False, "ownership_or_cache_label_mismatch"))
        with patch.object(manager, "inspect_volume", return_value={"Name": "ci-target", "Labels": {manager.OWNER_LABEL: manager.OWNER, manager.PURPOSE_LABEL: "ci-cache"}}), \
             patch.object(manager, "volume_refs", return_value=["stopped-container-id"]):
            self.assertEqual(manager.cache_candidate("ci-target"), (False, "container_reference"))


if __name__ == "__main__":
    unittest.main()
