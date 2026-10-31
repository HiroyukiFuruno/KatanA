"""v45記録経路の実Git・公開API契約回帰。fixtureは実公開の証跡ではない。"""

import copy
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

from document_host_io import HostEvidenceError
import future_publication_records as records
from generated_asset_identity import file_digest
from generated_identity_fixtures import bundle, git, repository
from published_git_source import git_source_identity

ROOT = Path(__file__).resolve().parents[2]


class RecordsScopeTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        repository(self.root)
        self.originals = {}
        for change, capability in records.CHANGES.items():
            last = "3.6" if capability == "update-storage-preflight" else "3.5"
            content = f"- [x] 1.1 Implement exact behavior\n- [ ] {last} Archive and cleanup\n"
            self.originals[change] = content
            self.write(f"openspec/changes/{change}/tasks.md", content)
            self.write(f"openspec/changes/{change}/proposal.md", "Published requirement\n")
            self.write(f"openspec/changes/{change}/specs/{capability}/spec.md",
                       "## ADDED Requirements\n\n### Requirement: Exact behavior\n\nThe product SHALL retain its contract.\n")
        for name in ("check-openspec-task-phase.py", "check-openspec-task-completion.py"):
            self.write("scripts/release/" + name, (ROOT / "scripts/release" / name).read_text(encoding="utf-8"))
        self.commit()
        self.source = git(self.root, "rev-parse", "HEAD")
        self.write(records.EVIDENCE + "publication.md", "Synthetic unit metadata only\n")
        self.commit()

    def tearDown(self):
        self.temporary.cleanup()

    def write(self, name, text):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def commit(self):
        git(self.root, "add", ".")
        git(self.root, "commit", "-q", "-m", "Synthetic records unit change")

    def test_exact_route_and_active_phase_do_not_require_guessed_source(self):
        records.validate_route(records.BRANCH, records.VERSION, records.REPOSITORY)
        for branch, version, repository_name in ((records.BRANCH + "-extra", records.VERSION, records.REPOSITORY),
                (records.BRANCH, "0.22.44", records.REPOSITORY), (records.BRANCH, records.VERSION, "other/repo")):
            with self.assertRaises(HostEvidenceError):
                records.validate_route(branch, version, repository_name)
        changed, tasks = records.verify_scope(self.root, self.source)
        self.assertEqual(changed, [records.EVIDENCE + "publication.md"])
        self.assertEqual(set(tasks), set(records.CHANGES))
        selector = ROOT / "scripts/release/select-pr-readiness-mode.sh"
        for requested, expected in (("pr-bootstrap", "post-release-evidence"), ("", "post-release-evidence"),
                                     ("strict", "strict")):
            result = subprocess.run(["bash", str(selector), records.BRANCH, records.VERSION, requested],
                                    check=True, capture_output=True, text=True)
            self.assertEqual(expected, result.stdout.strip())
        result = subprocess.run(["bash", str(selector), "release/v0.22.45", records.VERSION, "pr-bootstrap"],
                                check=True, capture_output=True, text=True)
        self.assertEqual("pr-bootstrap", result.stdout.strip())

    def test_archive_and_spec_sync_preserve_original_file_and_task_text(self):
        for change, capability in records.CHANGES.items():
            active = self.root / "openspec/changes" / change
            archive = self.root / "openspec/changes/archive" / ("2026-11-01-" + change)
            archive.parent.mkdir(parents=True, exist_ok=True)
            active.rename(archive)
            (archive / "tasks.md").write_text(self.originals[change].replace("[ ]", "[x]"))
            delta = (archive / "specs" / capability / "spec.md").read_text()
            self.write(f"openspec/specs/{capability}/spec.md",
                       "## Purpose\n\nPublished requirement.\n\n## Requirements\n" + delta.removeprefix("## ADDED Requirements\n"))
        self.commit()
        _, tasks = records.verify_scope(self.root, self.source)
        self.assertTrue(all("/archive/2026-11-01-" in path for path in tasks.values()))
        path = next(iter(tasks.values()))
        self.write(path, (self.root / path).read_text().replace("exact behavior", "different behavior"))
        self.commit()
        with self.assertRaisesRegex(HostEvidenceError, "ID/text"):
            records.verify_scope(self.root, self.source)

    def test_pending_task_content_edit_duplicate_archive_and_bad_date_are_rejected(self):
        change = next(iter(records.CHANGES))
        active = f"openspec/changes/{change}/"
        for content in (self.originals[change].replace("[x] 1.1", "[ ] 1.1"),
                        self.originals[change] + "- [x] 9.9 Added task\n"):
            self.write(active + "tasks.md", content)
            self.commit()
            with self.assertRaises((HostEvidenceError, subprocess.CalledProcessError)):
                records.verify_scope(self.root, self.source)
        self.write(active + "tasks.md", self.originals[change])
        self.write(f"openspec/changes/archive/2026-11-01-{change}/tasks.md", self.originals[change])
        self.commit()
        with self.assertRaisesRegex(HostEvidenceError, "exactly one"):
            records.verify_scope(self.root, self.source)
        with self.assertRaises(ValueError):
            records.archive_change(f"openspec/changes/archive/2026-02-30-{change}/tasks.md")

    def test_production_metadata_scope_and_dirty_untracked_files_are_rejected(self):
        self.write("Cargo.lock", "changed product input\n")
        self.commit()
        with self.assertRaisesRegex(HostEvidenceError, "production tree"):
            records.verify_scope(self.root, self.source)
        git(self.root, "restore", "--source", self.source, "--", "Cargo.lock")
        self.commit()
        self.write("untracked.txt", "unresolved\n")
        with self.assertRaisesRegex(HostEvidenceError, "committed"):
            records.verify_scope(self.root, self.source)
        (self.root / "untracked.txt").unlink()
        self.write(records.EVIDENCE + "arbitrary.json", "{}\n")
        self.commit()
        with self.assertRaisesRegex(HostEvidenceError, "exact v45"):
            records.verify_scope(self.root, self.source)

    def test_nonregular_evidence_and_changed_published_spec_are_rejected(self):
        path = self.root / records.EVIDENCE / "public-release.json"
        path.symlink_to("publication.md")
        self.commit()
        with self.assertRaisesRegex(HostEvidenceError, "nonregular"):
            records.verify_scope(self.root, self.source)

    def test_unrelated_source_changed_requirements_and_change_content_are_rejected(self):
        tree = git(self.root, "rev-parse", "HEAD^{tree}")
        unrelated = git(self.root, "commit-tree", tree, "-m", "Synthetic unrelated source")
        with self.assertRaises(subprocess.CalledProcessError):
            records.verify_scope(self.root, unrelated)
        capability = next(iter(records.CHANGES.values()))
        self.write(f"openspec/specs/{capability}/spec.md", "## Purpose\n\nSynthetic.\n\n## Requirements\n\nAltered requirement\n")
        self.commit()
        with self.assertRaisesRegex(HostEvidenceError, "spec sync"):
            records.verify_scope(self.root, self.source)


class RecordsReceiptTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.source, assets, manifests, self.raw, _ = bundle(self.root)
        self.canonical = self.root / "target/canonical"
        shutil.copytree(manifests, self.canonical / "manifests")
        public_assets = []
        downloaded = {}
        for index, path in enumerate(sorted(assets.iterdir())):
            digest = file_digest(path)
            url = f"https://github.com/{records.REPOSITORY}/releases/download/{records.TAG}/{path.name}"
            public_assets.append({"id": index + 1, "name": path.name, "size": path.stat().st_size,
                                  "digest": "sha256:" + digest, "browser_download_url": url,
                                  "updated_at": "2026-10-30T02:00:00Z"})
            downloaded[path.name] = {"asset_id": index + 1, "size": path.stat().st_size, "sha256": digest,
                                     "downloaded_at": "2026-10-30T03:00:00Z", "download_url": url}
        self.release = {"id": 10, "tag_name": records.TAG, "draft": False, "prerelease": False,
                        "published_at": "2026-10-30T01:00:00Z",
                        "html_url": f"https://github.com/{records.REPOSITORY}/releases/tag/{records.TAG}",
                        "assets": public_assets}
        self.receipt = {"schema": "katana-published-asset-identity-v1", "version": records.TAG,
                        "scope": "asset-identity-only", "document_qa": "not_evaluated",
                        "github_run_id": "123", "github_run_attempt": "2",
                        "provenance": git_source_identity(self.root, self.source), "release_id": 10,
                        "assets": downloaded, "controller_commit": self.source,
                        "build_manifests": {platform: {"path": filename, "sha256": file_digest(self.canonical / filename)}
                                            for platform, filename in records.MANIFESTS.items()}}

    def tearDown(self):
        self.temporary.cleanup()

    def verify(self, receipt=None, release=None):
        records.verify_receipt(self.root, self.receipt if receipt is None else receipt,
                               self.release if release is None else release, self.source, "123", "2", self.canonical)

    def test_actual_api_digest_and_each_raw_build_manifest_bind_identity(self):
        self.verify()
        for key, value in (("version", "v0.22.44"), ("github_run_id", "124"), ("github_run_attempt", "3"),
                           ("scope", "host-qa"), ("document_qa", "passed"), ("provenance", {}), ("release_id", 11)):
            receipt = copy.deepcopy(self.receipt)
            receipt[key] = value
            with self.subTest(field=key), self.assertRaises(HostEvidenceError):
                self.verify(receipt)
        for item in self.release["assets"]:
            for field, value in (("id", 999), ("size", 999), ("digest", "sha256:" + "0" * 64),
                                 ("browser_download_url", "https://example.invalid/asset")):
                release = copy.deepcopy(self.release)
                next(asset for asset in release["assets"] if asset["name"] == item["name"])[field] = value
                with self.subTest(asset=item["name"], field=field), self.assertRaises(HostEvidenceError):
                    self.verify(release=release)
        for platform, filename in records.MANIFESTS.items():
            receipt = copy.deepcopy(self.receipt)
            receipt["build_manifests"][platform]["path"] = "../../other.json"
            with self.subTest(platform=platform), self.assertRaises(HostEvidenceError):
                self.verify(receipt)
            path = self.canonical / filename
            raw = path.read_bytes()
            path.unlink()
            with self.assertRaises(HostEvidenceError):
                self.verify()
            path.write_bytes(raw)

    def test_raw_self_claim_and_different_production_controller_cannot_be_reused(self):
        filename = records.MANIFESTS["linux"]
        record = copy.deepcopy(self.raw["linux"])
        record["github_run_attempt"] = "1"
        (self.canonical / filename).write_text(json.dumps(record))
        receipt = copy.deepcopy(self.receipt)
        receipt["build_manifests"]["linux"]["sha256"] = file_digest(self.canonical / filename)
        with self.assertRaisesRegex(HostEvidenceError, "run/attempt"):
            self.verify(receipt)
        (self.root / "Cargo.toml").write_text('[workspace]\n[workspace.package]\nversion="0.22.46"\n')
        git(self.root, "add", "Cargo.toml")
        git(self.root, "commit", "-q", "-m", "Synthetic different production")
        receipt["controller_commit"] = git(self.root, "rev-parse", "HEAD")
        with self.assertRaisesRegex(HostEvidenceError, "different production"):
            self.verify(receipt)

    def test_successful_same_repo_readiness_artifact_is_required(self):
        metadata = {"id": 456, "run_attempt": 1, "path": records.READINESS_WORKFLOW,
                    "event": "workflow_run", "status": "completed", "conclusion": "success", "head_branch": "master",
                    "head_sha": self.source, "repository": {"full_name": records.REPOSITORY},
                    "head_repository": {"full_name": records.REPOSITORY}}
        for field, value in (("path", ".github/workflows/untrusted.yml"), ("event", "pull_request"),
                             ("conclusion", "failure"), ("head_repository", {"full_name": "other/repo"})):
            run = metadata | {field: value}
            def reader(endpoint):
                self.assertIn("release-readiness.yml/runs?event=workflow_run", endpoint)
                return {"workflow_runs": [run]}
            with self.subTest(field=field), self.assertRaisesRegex(HostEvidenceError, "canonical"):
                records.canonical_identity(self.root, self.source, self.canonical, reader)
        def no_artifact(endpoint):
            if "/workflows/" in endpoint:
                return {"workflow_runs": [metadata]}
            if "/jobs?" in endpoint:
                self.assertIn("/runs/456/attempts/1/jobs?", endpoint)
                return {"total_count": 1, "jobs": [{"name": "Verify published asset identity", "status": "completed", "conclusion": "success",
                                                    "run_id": 456, "run_attempt": 1, "head_sha": self.source}]}
            self.assertIn("/runs/456/artifacts?", endpoint)
            return {"total_count": 0, "artifacts": []}
        with self.assertRaisesRegex(HostEvidenceError, "canonical"):
            records.canonical_identity(self.root, self.source, self.canonical, no_artifact)

    def test_canonical_api_contract_downloads_only_trusted_identity_artifact(self):
        metadata = {"id": 456, "run_attempt": 2, "path": records.READINESS_WORKFLOW,
                    "event": "workflow_run", "status": "completed", "conclusion": "success", "head_branch": "master",
                    "head_sha": self.source, "repository": {"full_name": records.REPOSITORY},
                    "head_repository": {"full_name": records.REPOSITORY}}
        job = {"name": "Verify published asset identity", "status": "completed", "conclusion": "success",
               "run_id": 456, "run_attempt": 2, "head_sha": self.source}
        artifact = {"name": "published-asset-identity", "expired": False,
                    "workflow_run": {"id": 456, "head_sha": self.source}}
        def reader(endpoint):
            if "/workflows/" in endpoint:
                return {"workflow_runs": [metadata]}
            if "/jobs?" in endpoint:
                self.assertIn("/attempts/2/jobs?", endpoint)
                return {"total_count": 1, "jobs": [job]}
            return {"total_count": 1, "artifacts": [artifact]}
        (self.canonical / "publication-identity.json").write_text(json.dumps(self.receipt))
        binary = self.root / "target/bin/gh"
        binary.parent.mkdir()
        binary.write_text(f"#!{os.sys.executable}\nimport shutil,sys\n"
                          f"assert sys.argv[1:8] == ['run','download','456','--repo',{records.REPOSITORY!r},'--name','published-asset-identity']\n"
                          f"assert sys.argv[8] == '--dir'\nshutil.copytree({str(self.canonical)!r},sys.argv[9])\n")
        binary.chmod(0o755)
        old_path = os.environ["PATH"]
        try:
            os.environ["PATH"] = str(binary.parent) + os.pathsep + old_path
            destination = self.root / "target/downloaded-canonical"
            self.assertEqual(self.receipt, records.canonical_identity(self.root, self.source, destination, reader))
            job["run_attempt"] = 1
            with self.assertRaisesRegex(HostEvidenceError, "run/attempt/source"):
                records.canonical_identity(self.root, self.source, self.root / "target/rejected", reader)
        finally:
            os.environ["PATH"] = old_path


if __name__ == "__main__":
    unittest.main()
