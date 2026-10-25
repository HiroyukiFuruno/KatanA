#!/usr/bin/env python3
"""型/metadataの合成UT専用。実配布受入データや実Release成功には使用しない。"""

import copy
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

from document_host_acceptance import provenance
from document_host_io import HostEvidenceError, digest, legacy, write_new
from document_host_publication import ASSETS, TARGETS, manifest_metadata, module, verify_download_receipt


def unit_sha(label):
    return digest(("SYNTHETIC-UNIT-TEST-ONLY:"+label).encode())


class CheckoutSourceBytesTests(unittest.TestCase):
    def test_actual_git_crlf_checkout_changes_source_identity_and_disabled_conversion_preserves_it(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            source = base / "source"
            source.mkdir()
            environment = legacy().git_environment()

            def git(directory, *args):
                return subprocess.run(["git", "-C", str(directory), "-c", "user.name=Host Evidence Unit Test",
                                       "-c", "user.email=unit-test@example.invalid",
                                       "-c", f"core.attributesFile={os.devnull}", *args],
                                      check=True, capture_output=True, text=True, env=environment)

            (source / "Cargo.toml").write_bytes(b'[workspace]\n')
            (source / "Cargo.lock").write_bytes(b'version = 4\n')
            git(source, "init", "-q")
            git(source, "config", "core.autocrlf", "false")
            git(source, "add", "Cargo.toml", "Cargo.lock")
            git(source, "commit", "-q", "-m", "Synthetic checkout bytes fixture")
            source_hash = legacy().source_tree_sha256(source)
            lock_hash = digest((source / "Cargo.lock").read_bytes())
            for convert in (True, False):
                target = base / str(convert)
                git(base, "clone", "--no-checkout", "--quiet", str(source), str(target))
                git(target, "config", "core.autocrlf", str(convert).lower())
                git(target, "read-tree", "--reset", "-u", "HEAD")
                self.assertEqual(git(target, "status", "--porcelain=v1").stdout, "")
                self.assertEqual(legacy().source_tree_sha256(target) == source_hash, not convert)
                self.assertEqual(digest((target / "Cargo.lock").read_bytes()) == lock_hash, not convert)

    def test_windows_release_sets_byte_policy_before_checkout(self):
        workflow = (Path(__file__).resolve().parents[2] / ".github/workflows/build-and-release.yml").read_text()
        windows = workflow.split("  build_windows:\n", 1)[1].split("\n  publish:", 1)[0]
        self.assertLess(windows.index("git config --global core.autocrlf false"), windows.index("uses: actions/checkout@"))


class ActualSourceCommitBoundaryTests(unittest.TestCase):
    def test_unavailable_object_fails_and_genuine_same_source_docs_successor_passes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            environment = legacy().git_environment()
            def git(*args):
                return subprocess.run(["git", "-C", str(root), "-c", "user.name=Host Evidence Unit Test",
                                       "-c", "user.email=unit-test@example.invalid", *args],
                                      check=True, capture_output=True, text=True, env=environment).stdout.strip()
            (root / "Cargo.toml").write_text('[workspace]\n[workspace.package]\nversion="0.22.42"\n')
            lock = ""
            for name in sorted(legacy().RENDERER_PACKAGES):
                lock += f'[[package]]\nname="{name}"\nversion="0.1.0"\nsource="{legacy().CRATES_IO_SOURCE}"\nchecksum="{unit_sha(name)}"\n\n'
            (root / "Cargo.lock").write_text(lock)
            git("init", "-q")
            git("add", "Cargo.toml", "Cargo.lock")
            git("commit", "-q", "-m", "Synthetic unit fixture only")
            before = module("_unit_actual_source_builder", "record-generated-artifact-identity.py").source_identity(root)
            with self.assertRaisesRegex(HostEvidenceError, "commit object unavailable"):
                provenance(root, dict(before, source_commit="1"*40))
            (root / "docs").mkdir()
            (root / "docs/unit-note.md").write_text("synthetic unit fixture only\n")
            git("add", "docs/unit-note.md")
            git("commit", "-q", "-m", "Synthetic docs-only successor")
            self.assertNotEqual(git("rev-parse", "HEAD"), before["source_commit"])
            provenance(root, before)
            (root / "crates").mkdir()
            (root / "crates/source.rs").write_text("// synthetic unit fixture only\n")
            git("add", "crates/source.rs")
            git("commit", "-q", "-m", "Synthetic production-source successor")
            with self.assertRaises(HostEvidenceError):
                provenance(root, before)


class PublicationMetadataBoundaryTests(unittest.TestCase):
    def fixture(self, root):
        source = {"source_commit": "a"*40, "source_tree_sha256": unit_sha("source"), "cargo_lock_sha256": unit_sha("lock"),
                  "published_dependencies": {}}
        assets = {name: {"asset_id": index+1, "size": 10, "sha256": unit_sha(name),
                         "downloaded_at": "2026-10-25T01:00:00+00:00",
                         "download_url": f"https://github.com/HiroyukiFuruno/KatanA/releases/download/v0.22.42/{name}"}
                  for index, name in enumerate(sorted(set().union(*ASSETS.values())))}
        producer = root / "scripts/release/record-published-artifact-download.py"
        producer.parent.mkdir(parents=True)
        producer.write_bytes(Path(__file__).with_name(producer.name).read_bytes())
        evidence_dir = root / "openspec/changes/unit-only/evidence"
        refs, records = {}, {}
        for platform in ASSETS:
            value = {"schema": "katana-generated-asset-identity-v1", "version": "v0.22.42", "platform": platform,
                     "provenance": source, "github_run_id": "123", "github_run_attempt": "1",
                     "targets": {target: {"main_sha256": unit_sha("main"), "sidecar_sha256": unit_sha("worker")} for target in TARGETS[platform]},
                     "assets": {name: {key: assets[name][key] for key in ("sha256", "size")} for name in ASSETS[platform]}}
            path = evidence_dir / (platform+".json")
            ref = write_new(path, value)
            ref["path"] = path.relative_to(root).as_posix()
            refs[platform], records[platform] = ref, value
        receipt = {"schema": "katana-release-download-v1", "release_url": "https://github.com/HiroyukiFuruno/KatanA/releases/tag/v0.22.42",
                   "release_id": 123, "tag": "v0.22.42", "draft": False, "published_at": "2026-10-25T00:00:00+00:00",
                   "source_commit": source["source_commit"], "provenance": source, "assets": assets,
                   "downloaded_macos": "KatanA-macOS.zip", "main_sha256": unit_sha("main"), "sidecar_sha256": unit_sha("worker"),
                   "build_manifests": refs, "producer_sha256": digest(producer.read_bytes())}
        bundle = {"main_sha256": unit_sha("main"), "sidecar_sha256": unit_sha("worker"), "archive_sha256": assets["KatanA-macOS.zip"]["sha256"]}
        return receipt, bundle, source, records, evidence_dir / "receipt.json"

    def validate(self, root, receipt, bundle, source, path):
        raw = (json.dumps(receipt)+"\n").encode()
        path.write_bytes(raw)
        verify_download_receipt(root, {"status": "published_macos_host_accepted",
                                      "receipt": {"path": path.relative_to(root).as_posix(), "sha256": digest(raw)}}, source, bundle)

    def test_canonical_KatanA_urls_pass_and_owner_repo_substitution_rejects(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            receipt, bundle, source, _, path = self.fixture(root)
            self.validate(root, receipt, bundle, source, path)
            for replacement in ("SomeoneElse/KatanA", "HiroyukiFuruno/OtherRepo", "HiroyukiFuruno/katana"):
                changed = copy.deepcopy(receipt)
                changed["release_url"] = changed["release_url"].replace("HiroyukiFuruno/KatanA", replacement)
                with self.assertRaises(HostEvidenceError):
                    self.validate(root, changed, bundle, source, path)
                changed = copy.deepcopy(receipt)
                changed["assets"]["KatanA-macOS.zip"]["download_url"] = changed["assets"]["KatanA-macOS.zip"]["download_url"].replace("HiroyukiFuruno/KatanA", replacement)
                with self.assertRaises(HostEvidenceError):
                    self.validate(root, changed, bundle, source, path)

    def test_each_of_five_assets_and_each_platform_manifest_is_required(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            receipt, bundle, source, records, path = self.fixture(root)
            for name in receipt["assets"]:
                changed = copy.deepcopy(receipt)
                changed["assets"].pop(name)
                with self.subTest(asset=name), self.assertRaises(HostEvidenceError):
                    self.validate(root, changed, bundle, source, path)
            for platform in ASSETS:
                changed = copy.deepcopy(receipt)
                changed["build_manifests"].pop(platform)
                with self.assertRaises(HostEvidenceError):
                    self.validate(root, changed, bundle, source, path)
                for name in ASSETS[platform]:
                    changed = copy.deepcopy(records[platform])
                    changed["assets"].pop(name)
                    with self.assertRaises(HostEvidenceError):
                        manifest_metadata(platform, changed, source)

    def test_candidate_wrong_source_or_payload_cannot_satisfy_final_release(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            receipt, bundle, source, _, path = self.fixture(root)
            mutations = [lambda value: value.update(draft=True), lambda value: value.update(tag="v0.22.43"),
                         lambda value: value.update(source_commit="b"*40),
                         lambda value: value["assets"]["KatanA-macOS.zip"].update(sha256=unit_sha("candidate-not-published")),
                         lambda value: value["assets"]["KatanA-macOS.zip"].update(downloaded_at="2026-10-24T00:00:00+00:00"),
                         lambda value: value.update(main_sha256=unit_sha("different-main"))]
            for mutate in mutations:
                changed = copy.deepcopy(receipt)
                mutate(changed)
                with self.assertRaises(HostEvidenceError):
                    self.validate(root, changed, bundle, source, path)


if __name__ == "__main__":
    unittest.main()
