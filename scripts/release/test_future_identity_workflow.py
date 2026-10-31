"""Protect actual workflow wiring, historical policy, and read-only run trust."""

from pathlib import Path
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]


class FutureIdentityWorkflowTests(unittest.TestCase):
    def test_future_entrypoints_import_without_optional_site_packages(self):
        code = (f"import sys; sys.path.insert(0,{str(ROOT / 'scripts/release')!r}); "
                "import generated_asset_identity, generated_asset_verification, future_publication_identity; "
                "assert 'PIL' not in sys.modules")
        result = subprocess.run([sys.executable, "-S", "-B", "-c", code],
                                capture_output=True, text=True, timeout=60)
        self.assertEqual(0, result.returncode, result.stderr)
        for name in ("record-generated-artifact-identity.py", "check-generated-artifact-identity.py"):
            result = subprocess.run([sys.executable, "-S", "-B", str(ROOT / "scripts/release" / name), "--help"],
                                    capture_output=True, text=True, timeout=60)
            self.assertEqual(0, result.returncode, result.stderr)

    def test_all_builds_record_versioned_raw_identity_and_fail_missing_upload(self):
        workflow = (ROOT / ".github/workflows/build-and-release.yml").read_text(encoding="utf-8")
        for label, platform in (("macOS", "macos"), ("Linux", "linux"), ("Windows", "windows")):
            producer = workflow.split(f"      - name: Record generated {label} payload identity\n", 1)[1]
            producer = producer.split("      - name:", 1)[0]
            self.assertNotIn("if:", producer)
            self.assertIn("RELEASE_VERSION: ${{ needs.preflight.outputs.version }}", producer)
            self.assertIn('--version "$RELEASE_VERSION"', producer)
            self.assertIn(f"--platform {platform}", producer)
            upload = workflow.split(f"      - name: Upload generated {label} payload identity\n", 1)[1]
            upload = upload.split("      - name:", 1)[0]
            self.assertNotIn("version_bare ==", upload)
            self.assertIn(f"name: artifacts-identity-{platform}", upload)
            self.assertIn("if-no-files-found: error", upload)
        self.assertIn('--asset "target/release/KatanA-Desktop-$RELEASE_VERSION_BARE.dmg"', workflow)

    def test_all_identity_checks_precede_public_mutation_and_42_approval_stays(self):
        workflow = (ROOT / ".github/workflows/build-and-release.yml").read_text(encoding="utf-8")
        publish = workflow.split("  publish:\n", 1)[1]
        self.assertLess(publish.index("Normalize generated payload identity filenames"), publish.index("collect-artifacts.sh"))
        self.assertLess(publish.index("collect-artifacts.sh"), publish.index("check-generated-artifact-identity.py"))
        self.assertLess(publish.index("check-generated-artifact-identity.py"), publish.index("Create or Update GitHub Release"))
        old = publish.split("      - name: Verify downloaded publication executable identities\n", 1)[1]
        old = old.split("      - name:", 1)[0]
        self.assertIn("if: needs.preflight.outputs.version_bare == '0.22.42'", old)
        self.assertIn("check-document-fidelity-release-gate.py", old)
        self.assertIn("--scope automated-release", old)

    def test_finished_run_verification_does_not_relax_draft_or_required_build_jobs(self):
        readiness = (ROOT / ".github/workflows/release-readiness.yml").read_text(encoding="utf-8")
        self.assertIn("if: github.event_name == 'pull_request' && !github.event.pull_request.draft", readiness)
        self.assertIn("workflows: [Release]\n    types: [completed]", readiness)
        job = readiness.split("  published-asset-identity:\n", 1)[1]
        self.assertIn("head_repository.full_name == github.repository", job)
        self.assertIn("github.event.workflow_run.event == 'pull_request'", job)
        self.assertIn("github.event.workflow_run.head_branch == 'master'", job)
        self.assertNotIn("contents: write", job)
        self.assertNotIn("actions: write", job)
        self.assertNotIn("workflow_run.conclusion", job)
        self.assertIn("--phase postpublication", job)
        self.assertIn('--run-id "$BUILD_RUN" --run-attempt "$BUILD_ATTEMPT"', job)
        self.assertNotIn("/assets/", job.split("path: |", 1)[1])
        from postpublication_run_admission import REQUIRED_JOBS
        self.assertEqual(REQUIRED_JOBS, {"Preflight Check", "Build Linux", "Build macOS", "Build Windows",
                                         "Smoke macOS (x86_64)", "Smoke macOS (arm64)", "Publish Release"})


if __name__ == "__main__":
    unittest.main()
