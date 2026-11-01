"""v46 の公開後記録だけを公開ソースと証跡へ結び付ける。"""

from pathlib import Path
import copy
import os
import runpy
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from document_host_io import HostEvidenceError
import future_publication_records as records
from generated_asset_identity import file_digest
from generated_identity_fixtures import bundle, git, repository
from published_git_source import git_source_identity

ROOT = Path(__file__).resolve().parents[2]
VERSION = "0.22.46"
BRANCH = "release/v0.22.46-public-release-evidence"
CHANGE = "v0-22-46-startup-png-regex"
CAPABILITIES = ("local-asset-preview", "workspace-file-filter")


class Publication46Tests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        repository(self.root, "v0.22.46")
        self.write(f"openspec/changes/{CHANGE}/tasks.md", "- [x] 1.1 Impl\n- [ ] 5.6 Archive\n")
        self.write(f"openspec/changes/{CHANGE}/proposal.md", "Published proposal\n")
        for cap in CAPABILITIES:
            self.write(f"openspec/changes/{CHANGE}/specs/{cap}/spec.md",
                       "## ADDED Requirements\n\n### Requirement: New\n\nNew behavior.\n")
            self.write(f"openspec/specs/{cap}/spec.md",
                       "## Purpose\n\nPurpose.\n\n## Requirements\n\n### Requirement: Old\n\nOld behavior.\n")
        for name in ("check-openspec-task-phase.py", "check-openspec-task-completion.py"):
            self.write("scripts/release/" + name, (ROOT / "scripts/release" / name).read_text())
        self.commit()
        self.source = git(self.root, "rev-parse", "HEAD")
        self.write("evidence/releases/v0.22.46/publication.md", "Published metadata\n")
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
        for branch, version in ((BRANCH + "-extra", VERSION), (BRANCH, "0.22.47"),
                                ("release/v0.22.45-public-release-evidence", VERSION)):
            with self.assertRaises(HostEvidenceError):
                records.validate_route(branch, version, records.REPOSITORY)
        changed, tasks = records.verify_scope(self.root, self.source, VERSION)
        self.assertEqual(changed, ["evidence/releases/v0.22.46/publication.md"])
        self.assertEqual(set(tasks), {CHANGE})
        self.write("Cargo.lock", "modified source")
        self.commit()
        with self.assertRaisesRegex(HostEvidenceError, "production tree"):
            records.verify_scope(self.root, self.source, VERSION)

    def test_archival_sync_preserves_both_existing_and_added_requirements(self):
        active = self.root / "openspec/changes" / CHANGE
        archived = self.root / "openspec/changes/archive" / ("2026-11-01-" + CHANGE)
        archived.parent.mkdir(parents=True)
        active.rename(archived)
        (archived / "tasks.md").write_text("- [x] 1.1 Impl\n- [x] 5.6 Archive\n")
        for cap in CAPABILITIES:
            path = self.root / f"openspec/specs/{cap}/spec.md"
            path.write_text(path.read_text().rstrip() + "\n\n### Requirement: New\n\nNew behavior.\n")
        self.commit()
        records.verify_scope(self.root, self.source, VERSION)
        path = self.root / f"openspec/specs/{CAPABILITIES[0]}/spec.md"
        path.write_text(path.read_text().replace("Old behavior.", "Altered behavior."))
        self.commit()
        with self.assertRaisesRegex(HostEvidenceError, "spec sync"):
            records.verify_scope(self.root, self.source, VERSION)

    def test_route_selection_is_exact(self):
        selector = ROOT / "scripts/release/select-pr-readiness-mode.sh"
        for branch, version, expected in ((BRANCH, VERSION, "post-release-evidence"),
                (BRANCH + "-extra", VERSION, "pr-bootstrap"), (BRANCH, "0.22.47", "pr-bootstrap"),
                ("release/v0.22.46", VERSION, "pr-bootstrap")):
            result = subprocess.run(["bash", str(selector), branch, version, "pr-bootstrap"],
                                    check=True, capture_output=True, text=True)
            self.assertEqual(expected, result.stdout.strip())

    def test_workflow_preserves_normal_release_and_exact_record_routes(self):
        helper = runpy.run_path(str(ROOT / "scripts/release/test-release-flow-contract.py"))
        workflow = (ROOT / ".github/workflows/release-readiness.yml").read_text()
        route = helper["run_readiness_routing"]
        self.assertEqual([VERSION, "--post-release-evidence"], route(workflow, BRANCH))
        for branch in ("release/v0.22.46", BRANCH + "-extra"):
            self.assertEqual([VERSION, "--pr-bootstrap"], route(workflow, branch))

    def test_other_branches_and_post_release_modes_cannot_bypass_scope_guard(self):
        source = (ROOT / "scripts/release/check-pr-ready.sh").read_text()
        guard = source.split("# 5. Version increment or published-release guard\n", 1)[1].split(
            'if [[ "$TASK_GATE_MODE" == "post-release-evidence" ||', 1)[0]
        for branch, mode, accepted in ((BRANCH, "post-release-evidence", True),
                                       (BRANCH + "-extra", "post-release-evidence", False),
                                       (BRANCH, "post-release-host", False),
                                       (BRANCH, "post-release-automated", False),
                                       ("release/v0.22.46", "pr-bootstrap", True)):
            script = (f'TARGET_VERSION="{VERSION}"\nCURRENT_BRANCH="{branch}"\n'
                      f'TASK_GATE_MODE="{mode}"\nerror() {{ :; }}\n' + guard)
            result = subprocess.run(["bash", "-c", script], capture_output=True, text=True)
            self.assertEqual(accepted, result.returncode == 0, (branch, mode))

    def test_unpublished_or_draft_release_is_rejected_before_scope_admission(self):
        with patch.dict(os.environ, {"GITHUB_HEAD_REF": BRANCH}), patch.object(
            records, "api", return_value={"tag_name": "v0.22.46", "draft": True, "prerelease": False}
        ):
            with self.assertRaises(ValueError):
                records.verify(self.root, BRANCH, VERSION, records.REPOSITORY)


class Publication46ReceiptTests(unittest.TestCase):
    def test_all_three_platform_manifests_bind_to_v46_public_assets(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, assets, manifests, _, _ = bundle(root, "v0.22.46")
            canonical = root / "canonical"
            shutil.copytree(manifests, canonical / "manifests")
            public, downloaded = [], {}
            for index, path in enumerate(sorted(assets.iterdir())):
                digest = file_digest(path)
                url = f"https://github.com/{records.REPOSITORY}/releases/download/v0.22.46/{path.name}"
                public.append({"id": index + 1, "name": path.name, "size": path.stat().st_size,
                               "digest": "sha256:" + digest, "browser_download_url": url})
                downloaded[path.name] = {"asset_id": index + 1, "size": path.stat().st_size,
                                         "sha256": digest, "downloaded_at": "2026-11-01T03:00:00Z",
                                         "download_url": url}
            release = {"id": 10, "tag_name": "v0.22.46", "draft": False, "prerelease": False,
                       "published_at": "2026-11-01T01:00:00Z",
                       "html_url": f"https://github.com/{records.REPOSITORY}/releases/tag/v0.22.46",
                       "assets": public}
            receipt = {"schema": "katana-published-asset-identity-v1", "version": "v0.22.46",
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
