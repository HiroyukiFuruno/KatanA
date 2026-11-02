"""v47 の公開後記録だけを公開ソースと証跡へ結び付ける。"""

from pathlib import Path
import copy
import json
import runpy
import shutil
import subprocess
import tempfile
import unittest

from document_host_io import HostEvidenceError
import future_publication_records as records
from generated_asset_identity import file_digest
from generated_identity_fixtures import bundle, git, repository
from published_git_source import git_source_identity
from v047_evidence_paths import HOST_FILES, CANONICAL_FILES, HOST_RECEIPT, HOST_MANIFESTS

ROOT = Path(__file__).resolve().parents[2]
VERSION = "0.22.47"
BRANCH = "release/v0.22.47-public-release-evidence"
CHANGE = "v0-22-47-katana-contained-fixes"
CAPABILITIES = ("tab-group-dialog-interaction", "multi-format-document-preview", "generated-release-identity", "release-ci-integrity", "canonical-diagram-reference", "oss-repository-security", "auto-update", "workspace-shell")


def assert_delta_sync(test, root):
    changes = root / "openspec/changes"
    active = changes / CHANGE
    archives = [path for path in (changes / "archive").glob("*-" + CHANGE)
                if path.is_dir() and records.archive_change((path.relative_to(root) / "tasks.md").as_posix(), VERSION)]
    locations = ([active] if active.is_dir() else []) + archives
    test.assertEqual(len(locations), 1, "exactly one active or archived change is required")
    deltas = {path.parent.name: path for path in (locations[0] / "specs").glob("*/spec.md")}
    test.assertEqual(set(deltas), set(records.POLICIES[VERSION][CHANGE]))
    for capability, path in deltas.items():
        delta = path.read_text().removeprefix("## ADDED Requirements\n").strip()
        main = (root / "openspec/specs" / capability / "spec.md").read_text()
        test.assertEqual(main.count(delta), 1, capability)


class Publication47Tests(unittest.TestCase):
    def test_all_published_delta_capabilities_are_in_exact_scope_and_synced(self):
        assert_delta_sync(self, ROOT)

    def test_sync_check_accepts_active_or_archive_and_rejects_missing_or_duplicate(self):
        assert_delta_sync(self, self.root)
        active = self.root / "openspec/changes" / CHANGE
        archived = self.root / "openspec/changes/archive" / ("2026-11-02-" + CHANGE)
        archived.parent.mkdir(parents=True)
        active.rename(archived)
        self.commit()
        assert_delta_sync(self, self.root)
        archived.rename(active)
        self.commit()
        (active / "specs" / CAPABILITIES[0] / "spec.md").rename(active / "missing-spec.md")
        with self.assertRaises(AssertionError):
            assert_delta_sync(self, self.root)
        (active / "missing-spec.md").rename(active / "specs" / CAPABILITIES[0] / "spec.md")
        import shutil
        shutil.copytree(active, archived)
        with self.assertRaises(AssertionError):
            assert_delta_sync(self, self.root)
        active.rename(active.with_name("missing-change"))
        archived.rename(archived.with_name("missing-archive"))
        with self.assertRaises(AssertionError):
            assert_delta_sync(self, self.root)

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        repository(self.root, "v0.22.47")
        self.write(f"openspec/changes/{CHANGE}/tasks.md", "- [x] 1.1 Impl\n- [ ] 6.7 Archive\n")
        self.write(f"openspec/changes/{CHANGE}/proposal.md", "Published proposal\n")
        for cap in CAPABILITIES:
            self.write(f"openspec/changes/{CHANGE}/specs/{cap}/spec.md",
                       "## ADDED Requirements\n\n### Requirement: New\n\nNew behavior.\n")
            self.write(f"openspec/specs/{cap}/spec.md",
                       "## Purpose\n\nPurpose.\n\n## Requirements\n\n### Requirement: Old\n\nOld behavior.\n\n### Requirement: New\n\nNew behavior.\n")
        for name in ("check-openspec-task-phase.py", "check-openspec-task-completion.py"):
            self.write("scripts/release/" + name, (ROOT / "scripts/release" / name).read_text())
        self.commit()
        self.source = git(self.root, "rev-parse", "HEAD")
        self.write("evidence/releases/v0.22.47/publication.md", "Published metadata\n")
        self.commit()

    def tearDown(self):
        self.temp.cleanup()

    def write(self, name, text):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def commit(self):
        git(self.root, "add", ".")
        git(self.root, "commit", "-q", "-m", "test")

    def test_exact_route_and_published_source_scope(self):
        records.validate_route(BRANCH, VERSION, records.REPOSITORY)
        for branch, version in ((BRANCH + "-extra", VERSION), (BRANCH, "0.22.46"),
                                ("release/v0.22.45-public-release-evidence", VERSION)):
            with self.assertRaises(HostEvidenceError):
                records.validate_route(branch, version, records.REPOSITORY)
        changed, tasks = records.verify_scope(self.root, self.source, VERSION)
        self.assertEqual(changed, ["evidence/releases/v0.22.47/publication.md"])
        self.assertEqual(set(tasks), {CHANGE})
        self.write("Cargo.lock", "modified source")
        self.commit()
        with self.assertRaisesRegex(HostEvidenceError, "production tree"):
            records.verify_scope(self.root, self.source, VERSION)

    def test_exact_v47_record_paths_survive_archive_and_reject_unknown_names(self):
        allowed = HOST_FILES | CANONICAL_FILES
        self.assertEqual(len(allowed), 15)
        for name in allowed:
            self.assertTrue(records.allowed_path(name, set(), VERSION))
            self.assertFalse(records.allowed_path(name, set(), "0.22.46"))
            self.write(name, "synthetic unit record\n")
        self.commit()
        records.verify_scope(self.root, self.source, VERSION)
        records.verify_v47_record_inventory(self.root)
        active = self.root / "openspec/changes" / CHANGE
        archived = self.root / "openspec/changes/archive" / ("2026-11-02-" + CHANGE)
        archived.parent.mkdir(parents=True)
        active.rename(archived)
        (archived / "tasks.md").write_text("- [x] 1.1 Impl\n- [x] 6.7 Archive\n")
        self.commit()
        records.verify_scope(self.root, self.source, VERSION)
        self.write("evidence/releases/v0.22.47/document-host/unknown.json", "unknown\n")
        self.commit()
        with self.assertRaisesRegex(HostEvidenceError, "outside exact"):
            records.verify_scope(self.root, self.source, VERSION)

    def test_missing_committed_v47_record_is_rejected(self):
        for name in HOST_FILES | CANONICAL_FILES:
            self.write(name, "synthetic unit record\n")
        self.commit()
        records.verify_v47_record_inventory(self.root)
        (self.root / next(iter(HOST_FILES))).unlink()
        git(self.root, "add", "-u")
        git(self.root, "commit", "-q", "-m", "unit remove record")
        with self.assertRaisesRegex(HostEvidenceError, "complete v47 host"):
            records.verify_v47_record_inventory(self.root)

    def test_unsynced_published_main_spec_is_rejected(self):
        path = self.root / f"openspec/specs/{CAPABILITIES[0]}/spec.md"
        path.write_text(path.read_text().replace("### Requirement: New\n\nNew behavior.\n", ""))
        self.commit()
        source = git(self.root, "rev-parse", "HEAD")
        with self.assertRaisesRegex(HostEvidenceError, "already contain its delta"):
            records.verify_scope(self.root, source, VERSION)

    def test_records_paths_are_ignored_by_release_trigger_without_ignoring_assets(self):
        workflow = (ROOT / ".github/workflows/build-and-release.yml").read_text()
        trigger = workflow.split("  pull_request:\n", 1)[1].split("  push:\n", 1)[0]
        self.assertIn("      - 'evidence/**'", trigger)
        self.assertIn("      - 'openspec/**'", trigger)
        self.assertNotIn("      - 'assets/**'", trigger)
        self.assertTrue(all(name.startswith("evidence/") for name in HOST_FILES | CANONICAL_FILES))
        self.assertFalse(records.allowed_path("assets/reference/sample-diagrams-top.png", set(), VERSION))

    def test_staged_change_hidden_by_unstaged_reversal_is_rejected(self):
        path = self.root / "Cargo.lock"
        original = path.read_bytes()
        path.write_bytes(b"unauthorized staged change\n")
        git(self.root, "add", "Cargo.lock")
        path.write_bytes(original)
        with self.assertRaises(HostEvidenceError):
            records.verify_scope(self.root, self.source, VERSION)

    def test_archive_rejects_any_main_spec_edit_including_duplicate_delta(self):
        active = self.root / "openspec/changes" / CHANGE
        archived = self.root / "openspec/changes/archive" / ("2026-11-01-" + CHANGE)
        archived.parent.mkdir(parents=True)
        active.rename(archived)
        (archived / "tasks.md").write_text("- [x] 1.1 Impl\n- [x] 6.7 Archive\n")
        self.commit()
        records.verify_scope(self.root, self.source, VERSION)
        path = self.root / f"openspec/specs/{CAPABILITIES[0]}/spec.md"
        path.write_text(path.read_text() + "\n### Requirement: New\n\nNew behavior.\n")
        self.commit()
        with self.assertRaisesRegex(HostEvidenceError, "main spec must remain unchanged"):
            records.verify_scope(self.root, self.source, VERSION)

    def test_route_selection_is_exact(self):
        selector = ROOT / "scripts/release/select-pr-readiness-mode.sh"
        for branch, version, expected in ((BRANCH, VERSION, "post-release-evidence"),
                (BRANCH + "-extra", VERSION, "pr-bootstrap"), (BRANCH, "0.22.46", "pr-bootstrap"),
                ("release/v0.22.47", VERSION, "pr-bootstrap")):
            result = subprocess.run(["bash", str(selector), branch, version, "pr-bootstrap"],
                                    check=True, capture_output=True, text=True)
            self.assertEqual(expected, result.stdout.strip())

    def test_workflow_preserves_normal_release_and_exact_record_routes(self):
        helper = runpy.run_path(str(ROOT / "scripts/release/test-release-flow-contract.py"))
        workflow = (ROOT / ".github/workflows/release-readiness.yml").read_text()
        route = helper["run_readiness_routing"]
        self.assertEqual([VERSION, "--post-release-evidence"], route(workflow, BRANCH))
        for branch in ("release/v0.22.47", BRANCH + "-extra"):
            self.assertEqual([VERSION, "--pr-bootstrap"], route(workflow, branch))

    def test_other_branches_and_post_release_modes_cannot_bypass_scope_guard(self):
        source = (ROOT / "scripts/release/check-pr-ready.sh").read_text()
        guard = source.split("# 5. Version increment or published-release guard\n", 1)[1].split(
            'if [[ "$TASK_GATE_MODE" == "post-release-evidence" ||', 1)[0]
        for branch, mode, accepted in ((BRANCH, "post-release-evidence", True),
                                       (BRANCH + "-extra", "post-release-evidence", False),
                                       (BRANCH, "post-release-host", False),
                                       (BRANCH, "post-release-automated", False),
                                       ("release/v0.22.47", "pr-bootstrap", True)):
            script = (f'TARGET_VERSION="{VERSION}"\nCURRENT_BRANCH="{branch}"\n'
                      f'TASK_GATE_MODE="{mode}"\nerror() {{ :; }}\n' + guard)
            result = subprocess.run(["bash", "-c", script], capture_output=True, text=True)
            self.assertEqual(accepted, result.returncode == 0, (branch, mode))



class Publication47ReceiptTests(unittest.TestCase):
    def test_host_binding_requires_admitted_release_assets_and_all_manifests(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, assets, manifests, builds, _ = bundle(root, "v0.22.47")
            canonical = root / "canonical"
            shutil.copytree(manifests, canonical / "manifests")
            asset_records = {path.name: {"asset_id": index + 1, "size": path.stat().st_size,
                             "sha256": file_digest(path), "download_url": "https://example.invalid/" + path.name}
                             for index, path in enumerate(sorted(assets.iterdir())) if path.name != "checksums.txt"}
            identity = {"release_id": 10, "assets": asset_records}
            (canonical / "publication-identity.json").write_text(json.dumps(identity))
            host = {"source_commit": source, "release_id": 10, "assets": asset_records}
            (root / HOST_RECEIPT).parent.mkdir(parents=True)
            (root / HOST_RECEIPT).write_text(json.dumps(host))
            for platform, name in HOST_MANIFESTS.items():
                (root / name).write_text(json.dumps(builds[platform]))
            records.verify_v47_host_binding(root, {"id": 10}, source, "123", "2", canonical)
            with self.assertRaisesRegex(HostEvidenceError, "run or attempt"):
                records.verify_v47_host_binding(root, {"id": 10}, source, "123", "1", canonical)
            changed_host = copy.deepcopy(host)
            changed_host["assets"][next(iter(changed_host["assets"]))]["sha256"] = "0" * 64
            (root / HOST_RECEIPT).write_text(json.dumps(changed_host))
            with self.assertRaisesRegex(HostEvidenceError, "public asset"):
                records.verify_v47_host_binding(root, {"id": 10}, source, "123", "2", canonical)
            (root / HOST_RECEIPT).write_text(json.dumps(host))
            changed_manifest = copy.deepcopy(builds["windows"])
            changed_manifest["provenance"]["source_commit"] = "0" * 40
            (root / HOST_MANIFESTS["windows"]).write_text(json.dumps(changed_manifest))
            with self.assertRaisesRegex(HostEvidenceError, "build manifest"):
                records.verify_v47_host_binding(root, {"id": 10}, source, "123", "2", canonical)
            (root / HOST_MANIFESTS["windows"]).write_text(json.dumps(builds["windows"]))
            host["release_id"] = 11
            (root / HOST_RECEIPT).write_text(json.dumps(host))
            with self.assertRaisesRegex(HostEvidenceError, "public release"):
                records.verify_v47_host_binding(root, {"id": 10}, source, "123", "2", canonical)

    def test_all_three_platform_manifests_bind_to_v47_public_assets(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, assets, manifests, _, _ = bundle(root, "v0.22.47")
            canonical = root / "canonical"
            shutil.copytree(manifests, canonical / "manifests")
            public, downloaded = [], {}
            for index, path in enumerate(sorted(assets.iterdir())):
                digest = file_digest(path)
                url = f"https://github.com/{records.REPOSITORY}/releases/download/v0.22.47/{path.name}"
                public.append({"id": index + 1, "name": path.name, "size": path.stat().st_size,
                               "digest": "sha256:" + digest, "browser_download_url": url})
                downloaded[path.name] = {"asset_id": index + 1, "size": path.stat().st_size,
                                         "sha256": digest, "downloaded_at": "2026-11-01T03:00:00Z",
                                         "download_url": url}
            release = {"id": 10, "tag_name": "v0.22.47", "draft": False, "prerelease": False,
                       "published_at": "2026-11-01T01:00:00Z",
                       "html_url": f"https://github.com/{records.REPOSITORY}/releases/tag/v0.22.47",
                       "assets": public}
            receipt = {"schema": "katana-published-asset-identity-v1", "version": "v0.22.47",
                       "scope": "asset-identity-only", "document_qa": "not_evaluated",
                       "github_run_id": "123", "github_run_attempt": "2",
                       "provenance": git_source_identity(root, source), "release_id": 10,
                       "assets": downloaded, "controller_commit": source,
                       "build_manifests": {platform: {"path": filename, "sha256": file_digest(canonical / filename)}
                                           for platform, filename in records.MANIFESTS.items()}}
            records.verify_receipt(root, receipt, release, source, "123", "2", canonical, VERSION)
            with self.assertRaises(HostEvidenceError):
                records.verify_receipt(root, receipt, release, source, "123", "2", canonical, "0.22.45")
            for platform in records.MANIFESTS:
                altered = copy.deepcopy(receipt)
                altered["build_manifests"][platform]["sha256"] = "0" * 64
                with self.subTest(platform=platform), self.assertRaises(HostEvidenceError):
                    records.verify_receipt(root, altered, release, source, "123", "2", canonical, VERSION)


if __name__ == "__main__":
    unittest.main()
