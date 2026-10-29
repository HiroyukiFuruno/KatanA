"""Regression checks for immutable published product and bounded controller repairs."""

import json
import unittest
import tomllib
from pathlib import Path

from document_host_io import HostEvidenceError
from published_git_source import PublishedGitSourceError
from document_host_publication import verify_generated_provenance, verify_publication_context
from postpublication_controller_scope import (
    CONTROLLER_PATHS, NEW_CLEANUP_INSTALL, OLD_CLEANUP_INSTALL, NEW_DOCUMENT_FILTER, OLD_DOCUMENT_FILTER,
    PINNED_SWEEP, UNPINNED_SWEEP, git, validate_controller_paths,
    REGISTRY_RECEIPT_UPLOAD,
    validate_tool_pin, validate_workflow_prefix, validate_worktree_clean, verify_committed_scope,
)

ROOT = Path(__file__).resolve().parents[2]
RECEIPT = json.loads((ROOT / "evidence/v0.22.42-public-download-40672472894.json").read_text())
PUBLISHED = RECEIPT["source_commit"]
BUILD_WORKFLOW = ".github/workflows/build-and-release.yml"
READINESS_WORKFLOW = ".github/workflows/release-readiness.yml"


class PublishedControllerScopeTests(unittest.TestCase):
    def published(self, path):
        return git(ROOT, "show", f"{PUBLISHED}:{path}")

    def test_current_head_cannot_relabel_product_changes_as_published_controller(self):
        head = git(ROOT, "rev-parse", "HEAD").decode().strip()
        original = tomllib.loads(self.published("Cargo.toml").decode())
        current = tomllib.loads(git(ROOT, "show", f"{head}:Cargo.toml").decode())
        if current["workspace"]["package"]["version"] == original["workspace"]["package"]["version"]:
            self.assertLessEqual(set(verify_committed_scope(ROOT, PUBLISHED, head)), CONTROLLER_PATHS)
        else:
            # WHY: 次版の製品変更を、公開済み版の非配布controller修正として認めない。
            with self.assertRaises(HostEvidenceError):
                verify_committed_scope(ROOT, PUBLISHED, head)

    def test_rejects_every_product_and_unrelated_tool_change(self):
        for path in ("Cargo.lock", "Cargo.toml", "crates/katana-ui/src/main.rs",
                     "vendor/egui-wgpu/src/winit.rs", "assets/font.ttf",
                     "scripts/release/record-generated-artifact-identity.py",
                     "scripts/release/preflight.sh", "scripts/other-controller.py"):
            with self.subTest(path=path), self.assertRaises(HostEvidenceError):
                validate_controller_paths([path])

    def test_only_exact_cleanup_installer_change_is_admitted(self):
        original = self.published(BUILD_WORKFLOW)
        repaired = original.replace(OLD_CLEANUP_INSTALL, NEW_CLEANUP_INSTALL, 1)
        repaired = repaired.replace(OLD_DOCUMENT_FILTER, NEW_DOCUMENT_FILTER, 1)
        self.assertNotEqual(original, repaired)
        validate_workflow_prefix(BUILD_WORKFLOW, original, repaired)
        for mutated in (repaired.replace(b"Start-Sleep -Milliseconds 200", b"Start-Sleep -Milliseconds 201"),
                        repaired.replace(b"--remote", b"--remote --force"),
                        repaired.replace(b"version 0.8.0", b"version 0.7.0")):
            with self.subTest(mutation=mutated[-80:]), self.assertRaises(HostEvidenceError):
                validate_workflow_prefix(BUILD_WORKFLOW, original, mutated)

    def test_readiness_must_preserve_all_prior_required_checks(self):
        original = self.published(READINESS_WORKFLOW)
        validate_workflow_prefix(READINESS_WORKFLOW, original, original)
        for mutated in (original.replace(b"fetch-depth: 0", b"fetch-depth: 1"),
                        original.replace(b"--post-release-automated", b"--pr-bootstrap"),
                        original.replace(b"Verify actual published release artifacts", b"Ignore published artifacts")):
            with self.subTest(mutation=mutated[:80]), self.assertRaises(HostEvidenceError):
                validate_workflow_prefix(READINESS_WORKFLOW, original, mutated)

    def test_registry_receipt_cannot_change_publication_or_upload_other_files(self):
        original = self.published(BUILD_WORKFLOW)
        repaired = original.replace(OLD_CLEANUP_INSTALL, NEW_CLEANUP_INSTALL, 1)
        repaired = repaired.replace(OLD_DOCUMENT_FILTER, NEW_DOCUMENT_FILTER, 1)
        repaired = repaired.replace(b"  cleanup_default:\n",
                                    REGISTRY_RECEIPT_UPLOAD + b"  cleanup_default:\n", 1)
        validate_workflow_prefix(BUILD_WORKFLOW, original, repaired)
        for mutated in (repaired.replace(b"path: release-files/external-registry-sync.json",
                                         b"path: release-files/*"),
                        repaired.replace(b"if: always()", b"if: success()"),
                        repaired.replace(b"continue-on-error: true", b"continue-on-error: false"),
                        repaired.replace(b"gh release create", b"gh release delete")):
            with self.subTest(mutation=mutated[-100:]), self.assertRaises(HostEvidenceError):
                validate_workflow_prefix(BUILD_WORKFLOW, original, mutated)

    def test_ci_tool_pin_cannot_change_any_other_test_or_threshold(self):
        original = self.published(".github/workflows/test-and-build.yml")
        repaired = original.replace(UNPINNED_SWEEP, PINNED_SWEEP, 1)
        self.assertNotEqual(original, repaired)
        validate_tool_pin(original, repaired)
        with self.assertRaises(HostEvidenceError):
            validate_tool_pin(original, repaired.replace(b"JOBS=1 just coverage", b"echo skip"))

    def test_default_provenance_still_rejects_a_different_source(self):
        changed = dict(RECEIPT["provenance"], source_tree_sha256="0" * 64)
        with self.assertRaises(HostEvidenceError):
            verify_generated_provenance(ROOT, changed, None)

    def test_explicit_published_provenance_checks_full_git_objects(self):
        verify_generated_provenance(ROOT, RECEIPT["provenance"], PUBLISHED)
        changed = dict(RECEIPT["provenance"], cargo_lock_sha256="0" * 64)
        with self.assertRaises(PublishedGitSourceError):
            verify_generated_provenance(ROOT, changed, PUBLISHED)

    def test_frozen_source_cannot_be_used_as_a_build_job_mode(self):
        with self.assertRaises(HostEvidenceError):
            verify_publication_context(ROOT, RECEIPT["provenance"], PUBLISHED, "123", "1")
        with self.assertRaises(HostEvidenceError):
            verify_publication_context(ROOT, {"source_commit": "0" * 40}, PUBLISHED, None, None)

    def test_uncommitted_controller_and_product_sources_are_rejected(self):
        validate_worktree_clean(b"", b"")
        for dirty, untracked in ((b"scripts/controller.py\0", b""),
                                 (b"", b"crates/other.rs\0"), (b"changed\0", b"new\0")):
            with self.subTest(dirty=dirty, untracked=untracked), self.assertRaises(HostEvidenceError):
                validate_worktree_clean(dirty, untracked)


if __name__ == "__main__":
    unittest.main()
