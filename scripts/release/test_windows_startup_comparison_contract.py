import hashlib
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/release/prepare-windows-startup-comparison.py"
WORKFLOW = ROOT / ".github/workflows/build-and-release.yml"
EXPECTED_BLOCK_SHA = "726979a6a0eb68c1eb8c64f4b277c4fef923d9f67043b21469caa376b7ac7cd7"


def load_helper():
    spec = importlib.util.spec_from_file_location("windows_startup_comparison", SCRIPT)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def isolated_git_environment() -> dict[str, str]:
    environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
    environment.update({"GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull})
    return environment


def git(root: Path, *arguments: str) -> str:
    result = subprocess.run(["git", "-C", str(root), *arguments], check=True, capture_output=True,
                            text=True, env=isolated_git_environment())
    return result.stdout.strip()


class WindowsStartupComparisonContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.helper = load_helper()
        cls.workflow = WORKFLOW.read_text(encoding="utf-8")

    def make_fixture(self):
        temporary = tempfile.TemporaryDirectory(prefix="katana-windows-comparison-")
        root = Path(temporary.name)
        (root / ".github/workflows").mkdir(parents=True)
        shutil.copy2(WORKFLOW, root / ".github/workflows/build-and-release.yml")
        (root / "marker").write_text("source\n", encoding="utf-8")
        subprocess.run(["git", "init", "-q", "--initial-branch", "main", str(root)], check=True,
                       env=isolated_git_environment())
        subprocess.run(["git", "-C", str(root), "add", "marker", ".github/workflows/build-and-release.yml"], check=True,
                       env=isolated_git_environment())
        subprocess.run(["git", "-C", str(root), "-c", "user.name=test", "-c", "user.email=test@example.invalid",
                        "commit", "-qm", "source"], check=True, env=isolated_git_environment())
        commit = git(root, "rev-parse", "HEAD")
        payload = root / "payload"
        (payload / "tmp").mkdir(parents=True)
        main, worker = b"main-bytes", b"worker-bytes"
        archive = payload / "KatanA-windows-x86_64.zip"
        with zipfile.ZipFile(archive, "w") as bundle:
            bundle.writestr("KatanA.exe", main)
            bundle.writestr("kdv-office-worker.exe", worker)
        manifest = {
            "schema": "katana-generated-asset-identity-v1", "version": "v0.22.42", "platform": "windows",
            "provenance": {"source_commit": commit}, "github_run_id": "123", "github_run_attempt": "1",
            "targets": {"windows-x86_64": {"main_sha256": hashlib.sha256(main).hexdigest(),
                                             "sidecar_sha256": hashlib.sha256(worker).hexdigest()}},
            "assets": {"KatanA-windows-x86_64.zip": {"sha256": hashlib.sha256(archive.read_bytes()).hexdigest(),
                                                        "size": archive.stat().st_size},
                       "KatanA-windows-x86_64.msi": {"sha256": "a" * 64, "size": 1}},
        }
        manifest_path = payload / "tmp/identity-windows.json"
        manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
        return temporary, root, payload, manifest, manifest_path, archive, commit, main, worker

    def test_original_smoke_block_is_extracted_verbatim(self):
        block, body = self.helper.extract_original_smoke_block(WORKFLOW)
        self.assertEqual(hashlib.sha256(block.encode()).hexdigest(), EXPECTED_BLOCK_SHA)
        self.assertIn("Start-Sleep -Milliseconds 200", body)
        self.assertIn("foreach ($attempt in 1..10)", body)
        self.assertIn("if ($peakRss -gt 512MB)", body)
        self.assertIn("if ($fontBytes -gt 134217728)", body)

    def test_successful_cli_preparation_uses_real_git_and_real_zip(self):
        temporary, root, payload, _manifest, _manifest_path, _archive, commit, _main, _worker = self.make_fixture()
        try:
            alternate = root / "alternate-repo"
            alternate.mkdir()
            (alternate / "other").write_text("other\n", encoding="utf-8")
            subprocess.run(["git", "init", "-q", "--initial-branch", "main", str(alternate)], check=True,
                           env=isolated_git_environment())
            subprocess.run(["git", "-C", str(alternate), "add", "other"], check=True,
                           env=isolated_git_environment())
            subprocess.run(["git", "-C", str(alternate), "-c", "user.name=other", "-c", "user.email=other@example.invalid",
                            "commit", "-qm", "other"], check=True, env=isolated_git_environment())
            output_script = root / "out/smoke.ps1"
            output_metadata = root / "out/metadata.json"
            environment = isolated_git_environment()
            environment["GIT_DIR"] = os.fspath(alternate / ".git")
            result = subprocess.run([
                sys.executable, os.fspath(SCRIPT), "--payload-dir", os.fspath(payload), "--workflow",
                os.fspath(root / ".github/workflows/build-and-release.yml"), "--current-root", os.fspath(root),
                "--output-script", os.fspath(output_script), "--output-metadata", os.fspath(output_metadata),
                "--expected-version", "v0.22.42", "--expected-source-commit", commit,
                "--expected-run-id", "123", "--expected-run-attempt", "1",
            ], check=False, capture_output=True, text=True, env=environment)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertEqual(hashlib.sha256(output_script.read_bytes()).hexdigest(),
                             hashlib.sha256(self.helper.extract_original_smoke_block(WORKFLOW)[1].encode()).hexdigest())
            self.assertEqual(json.loads(output_metadata.read_text())["source_commit"], commit)
        finally:
            temporary.cleanup()

    def test_identity_boundaries_reject_source_attempt_archive_main_worker_and_duplicate_entry(self):
        for case in ("source", "attempt", "archive", "main", "worker", "duplicate"):
            with self.subTest(case=case):
                temporary, root, payload, manifest, manifest_path, archive, commit, main, worker = self.make_fixture()
                try:
                    if case == "source":
                        with self.assertRaisesRegex(self.helper.PreparationError, "source commit"):
                            self.helper.validate_payload(payload, expected_version="v0.22.42", expected_source_commit="a" * 40,
                                                         expected_run_id="123", expected_run_attempt="1", current_root=root)
                    elif case == "attempt":
                        with self.assertRaisesRegex(self.helper.PreparationError, "run attempt"):
                            self.helper.validate_payload(payload, expected_version="v0.22.42", expected_source_commit=commit,
                                                         expected_run_id="123", expected_run_attempt="2", current_root=root)
                    elif case == "archive":
                        manifest["assets"]["KatanA-windows-x86_64.zip"]["sha256"] = "b" * 64
                        manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
                        with self.assertRaisesRegex(self.helper.PreparationError, "archive hash"):
                            self.helper.validate_payload(payload, expected_version="v0.22.42", expected_source_commit=commit,
                                                         expected_run_id="123", expected_run_attempt="1", current_root=root)
                    elif case == "main":
                        manifest["targets"]["windows-x86_64"]["main_sha256"] = "b" * 64
                        manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
                        with self.assertRaisesRegex(self.helper.PreparationError, "main hash"):
                            self.helper.validate_payload(payload, expected_version="v0.22.42", expected_source_commit=commit,
                                                         expected_run_id="123", expected_run_attempt="1", current_root=root)
                    elif case == "worker":
                        manifest["targets"]["windows-x86_64"]["sidecar_sha256"] = "b" * 64
                        manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
                        with self.assertRaisesRegex(self.helper.PreparationError, "worker hash"):
                            self.helper.validate_payload(payload, expected_version="v0.22.42", expected_source_commit=commit,
                                                         expected_run_id="123", expected_run_attempt="1", current_root=root)
                    else:
                        with zipfile.ZipFile(archive, "a") as bundle:
                            bundle.writestr("KatanA.exe", main)
                        manifest["assets"]["KatanA-windows-x86_64.zip"] = {
                            "sha256": hashlib.sha256(archive.read_bytes()).hexdigest(), "size": archive.stat().st_size}
                        manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
                        with self.assertRaisesRegex(self.helper.PreparationError, "exactly one KatanA.exe"):
                            self.helper.validate_payload(payload, expected_version="v0.22.42", expected_source_commit=commit,
                                                         expected_run_id="123", expected_run_attempt="1", current_root=root)
                finally:
                    temporary.cleanup()

    def test_original_body_drift_is_rejected(self):
        temporary, root, _payload, _manifest, _manifest_path, _archive, _commit, _main, _worker = self.make_fixture()
        try:
            workflow = root / ".github/workflows/build-and-release.yml"
            source = workflow.read_text(encoding="utf-8")
            workflow.write_text(source.replace("Start-Sleep -Milliseconds 200", "Start-Sleep -Milliseconds 201", 1), encoding="utf-8")
            with self.assertRaisesRegex(self.helper.PreparationError, "smoke block changed"):
                self.helper.extract_original_smoke_block(workflow)
        finally:
            temporary.cleanup()

    def test_comparison_preserves_original_publish_contract_and_captures_success(self):
        publish = self.workflow.split("  publish:", 1)[1]
        self.assertIn("needs: [preflight, build_macos, smoke_macos, build_linux, build_windows]", publish)
        self.assertIn("needs.build_windows.result == 'success'", publish)
        self.assertNotIn("windows_startup_comparison", publish)
        self.assertIn("runs-on: windows-latest", self.workflow)
        self.assertIn("runs-on: windows-2022", self.workflow)
        self.assertIn("timeout-minutes: 10", self.workflow)
        self.assertIn("if: always()", self.workflow[self.workflow.index("windows_startup_comparison:"):])
        self.assertIn("steps.windows_startup_comparison_smoke.outputs.diagnostics_directory", self.workflow)
        self.assertIn("parent_session_id", self.workflow)
        self.assertIn("image_os = $env:ImageOS", self.workflow)
        self.assertIn("image_version = $env:ImageVersion", self.workflow)
        self.assertIn("Start-Sleep -Milliseconds 200", self.workflow)
        self.assertIn("foreach ($attempt in 1..10)", self.workflow)
        self.assertNotIn("continue-on-error", self.workflow[self.workflow.index("Smoke launch packaged Windows app"):])


if __name__ == "__main__":
    unittest.main()
