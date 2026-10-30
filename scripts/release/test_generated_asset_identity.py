"""Real Git/archive unit regressions; synthetic records never prove a release."""

import copy
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from document_host_io import HostEvidenceError
import document_host_publication as frozen
from generated_asset_identity import assets_for, make_manifest, manifest_metadata, module
from generated_asset_verification import verify, verify_records
from generated_identity_fixtures import bundle, checksums, write_record

ROOT = Path(__file__).resolve().parents[2]
PRODUCER = ROOT / "scripts/release/record-generated-artifact-identity.py"


class GeneratedIdentityTests(unittest.TestCase):
    def test_future_versions_bind_real_three_platform_archives_and_five_assets(self):
        for version in ("v0.22.45", "v1.2.3"):
            with self.subTest(version=version), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                commit, assets, manifests, records, _ = bundle(root, version)
                result = verify_records(root, assets, manifests, version, commit, "123", "2")
                self.assertEqual(set(result["manifests"]), {"macos", "linux", "windows"})
                self.assertEqual(set().union(*(set(record["assets"]) for record in records.values())),
                                 set().union(*assets_for(version).values()))
                identity = module("_unit_real_portable", "check-published-artifact-identity.py")
                identity.check_portable(assets, result["targets"])
                # WHY: synthetic installerは本物の受入成功にせず、完全検証が実installerを要求することも確認する。
                with self.assertRaises((ValueError, RuntimeError, OSError, subprocess.SubprocessError)):
                    verify(root, assets, manifests, version, commit, "123", "2")

    def test_fixed_42_adapter_rejects_future_manifest_without_broadening_approval(self):
        self.assertEqual(frozen.ASSETS, assets_for("v0.22.42"))
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            _, _, _, records, _ = bundle(root)
            for platform, record in records.items():
                with self.assertRaises(HostEvidenceError):
                    frozen.manifest_metadata(platform, record, record["provenance"])

    def test_missing_mixed_or_modified_records_fail_closed(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            commit, assets, manifests, records, _ = bundle(root)
            for field, value in (("version", "v0.22.44"), ("github_run_id", "124"),
                                 ("github_run_attempt", "1"), ("github_run_attempt", True),
                                 ("platform", "windows"), ("provenance", {}), ("targets", {}), ("assets", {})):
                changed = copy.deepcopy(records["macos"])
                changed[field] = value
                write_record(manifests, "macos", changed)
                with self.subTest(field=field), self.assertRaises(HostEvidenceError):
                    verify_records(root, assets, manifests, "v0.22.45", commit, "123", "2")
            write_record(manifests, "macos", records["macos"])
            for platform in records:
                path = manifests / f"artifacts-identity-{platform}/identity.json"
                data = path.read_bytes()
                path.unlink()
                with self.subTest(platform=platform), self.assertRaises((HostEvidenceError, OSError)):
                    verify_records(root, assets, manifests, "v0.22.45", commit, "123", "2")
                path.write_bytes(data)

    def test_exact_source_and_each_asset_hash_cannot_be_relabelled(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            commit, assets, manifests, records, _ = bundle(root)
            for field in ("source_commit", "source_tree_sha256", "cargo_lock_sha256", "published_dependencies"):
                changed = copy.deepcopy(records["linux"])
                changed["provenance"][field] = {} if field == "published_dependencies" else "0" * len(changed["provenance"][field])
                write_record(manifests, "linux", changed)
                with self.subTest(field=field), self.assertRaises(HostEvidenceError):
                    verify_records(root, assets, manifests, "v0.22.45", commit, "123", "2")
            write_record(manifests, "linux", records["linux"])
            for name in set().union(*assets_for("v0.22.45").values()):
                path = assets / name
                original = path.read_bytes()
                path.write_bytes(original + b"modified")
                checksums(assets)
                with self.subTest(asset=name), self.assertRaises(HostEvidenceError):
                    verify_records(root, assets, manifests, "v0.22.45", commit, "123", "2")
                path.write_bytes(original)
            checksums(assets)
            (assets / "unknown.zip").write_bytes(b"extra")
            with self.assertRaises(HostEvidenceError):
                verify_records(root, assets, manifests, "v0.22.45", commit, "123", "2")

    def test_invalid_version_and_commit_version_mismatch_are_rejected(self):
        for version in ("0.22.45", "v00.22.45", "v0.22.45-extra", "v0.22.45/../42"):
            with self.subTest(version=version), self.assertRaises(ValueError):
                assets_for(version)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            commit, assets, manifests, _, _ = bundle(root)
            with self.assertRaisesRegex(HostEvidenceError, "committed Cargo.toml"):
                verify_records(root, assets, manifests, "v0.22.44", commit, "123", "2")

    def test_real_producer_requires_actions_clean_source_version_and_new_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            _, assets, _, records, binaries = bundle(root)
            environment = dict(os.environ, GITHUB_ACTIONS="true", GITHUB_RUN_ID="123", GITHUB_RUN_ATTEMPT="2")
            for platform, (main, worker) in binaries.items():
                output = root / "target" / f"generated-{platform}.json"
                command = [sys.executable, str(PRODUCER), "--root", str(root), "--version", "v0.22.45",
                           "--platform", platform, "--main", str(main), "--worker", str(worker), "--output", str(output)]
                for name in records[platform]["assets"]:
                    command.extend(("--asset", str(assets / name)))
                result = subprocess.run(command, env=environment, capture_output=True, text=True, timeout=60)
                self.assertEqual(0, result.returncode, result.stderr)
                for modified in (command, ["v0.22.44" if item == "v0.22.45" else item for item in command]):
                    rejected = subprocess.run(modified, env=environment, capture_output=True, text=True, timeout=60)
                    self.assertNotEqual(0, rejected.returncode)
            rejected = subprocess.run(command, env={**environment, "GITHUB_ACTIONS": "false"},
                                      capture_output=True, text=True, timeout=60)
            self.assertNotEqual(0, rejected.returncode)
            (root / "crates").mkdir()
            (root / "crates/dirty.rs").write_text("// SYNTHETIC UNIT SOURCE\n")
            rejected = subprocess.run(command, env=environment, capture_output=True, text=True, timeout=60)
            self.assertNotEqual(0, rejected.returncode)
            self.assertIn("uncommitted production", rejected.stderr)


if __name__ == "__main__":
    unittest.main()
