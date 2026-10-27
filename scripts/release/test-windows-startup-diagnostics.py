#!/usr/bin/env python3
"""派生診断scriptが必須smokeの条件と入力証跡を保つことを検査する。"""

import hashlib
import importlib.util
import json
import tempfile
from types import SimpleNamespace
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
HELPER_PATH = ROOT / "scripts/release/prepare-windows-startup-diagnostics.py"
COMPARISON_PATH = ROOT / "scripts/release/prepare-windows-startup-comparison.py"
WORKFLOW = ROOT / ".github/workflows/build-and-release.yml"


def load_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise AssertionError(f"could not load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class WindowsStartupDiagnosticTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.helper = load_module("windows_startup_diagnostics", HELPER_PATH)
        cls.comparison = load_module("windows_startup_comparison", COMPARISON_PATH)
        cls.body = cls.comparison.extract_original_smoke_block(WORKFLOW)[1]
        cls.diagnostics_workflow = (ROOT / ".github/workflows/html-failure-diagnostics.yml").read_text(encoding="utf-8")

    def assert_protected_contract(self, body: str) -> None:
        for contract in ("Start-Sleep -Milliseconds 200", "foreach ($attempt in 1..10)",
                         "if ($peakRss -gt 512MB)", "if ($fontBytes -gt 134217728)",
                         "Office worker is live", "Assert-PackagedProcessIdentity", "finally {"):
            self.assertEqual(body.count(contract), self.body.count(contract), contract)

    def test_original_case_only_isolates_its_extraction_directory(self):
        result = self.helper.diagnostic_variant(self.body, "original")
        self.assertEqual(result, self.body.replace(self.helper.SMOKE_DIRECTORY_LINE,
                                                   self.helper.ISOLATED_DIRECTORY_LINE, 1))
        self.assertIn(self.helper.RUN_LOG_LINE, result)
        self.assertIn('$env:DEBUG = "true"', result)
        self.assert_protected_contract(result)

    def test_reduced_trace_case_changes_only_directory_and_native_trace_levels(self):
        result = self.helper.diagnostic_variant(self.body, "reduced-native-trace")
        expected = self.body.replace(self.helper.SMOKE_DIRECTORY_LINE,
                                      self.helper.ISOLATED_DIRECTORY_LINE, 1)
        expected = expected.replace(self.helper.RUN_LOG_LINE, self.helper.REDUCED_LOG_LINE, 1)
        self.assertEqual(result, expected)
        self.assertIn('$env:DEBUG = "true"', result)
        self.assertIn("katana_ui=info,katana_core=info,KatanA=info", result)
        self.assertNotIn("eframe::native::run=trace", result)
        self.assert_protected_contract(result)

    def test_unknown_control_cases_are_rejected(self):
        for case in ("no-redirection-bitmap", "control-frame-change"):
            with self.subTest(case=case), self.assertRaisesRegex(self.helper.PreparationError,
                                                                "unsupported diagnostic case"):
                self.helper.diagnostic_variant(self.body, case)

    def test_macos_sampler_defaults_remain_separate_from_windows_dispatch(self):
        native = self.diagnostics_workflow.split("  native-sample:", 1)[1].split("\n  windows-startup-diagnostics:", 1)[0]
        windows = self.diagnostics_workflow.split("  windows-startup-diagnostics:", 1)[1]
        self.assertIn("runs-on: macos-latest", native)
        self.assertIn("github.event_name == 'pull_request' && !github.event.pull_request.draft", native)
        self.assertIn("inputs.windows_run_id == ''", native)
        self.assertIn("inputs.windows_source_commit == ''", native)
        self.assertIn("inputs.windows_run_attempt == ''", native)
        self.assertIn("runs-on: windows-latest", windows)
        self.assertIn("actions: read", windows)

    def test_unexpected_or_duplicated_source_lines_are_rejected(self):
        for body in (self.body.replace(self.helper.SMOKE_DIRECTORY_LINE, "", 1),
                     self.body + self.helper.RUN_LOG_LINE,
                     self.body.replace(self.helper.RUN_LOG_LINE, "", 1)):
            with self.subTest(body_hash=hashlib.sha256(body.encode()).hexdigest()), \
                    self.assertRaises(self.helper.PreparationError):
                self.helper.diagnostic_variant(body, "reduced-native-trace")

    def test_preparation_requires_original_body_hash_and_run_binding(self):
        with tempfile.TemporaryDirectory(prefix="katana-windows-diagnostic-") as name:
            root = Path(name)
            source = root / "source.ps1"
            metadata = root / "source.json"
            source.write_text(self.body, encoding="utf-8")
            metadata.write_text(json.dumps({"powershell_body_sha256": "0" * 64,
                                            "version": "v0.22.42", "source_commit": "a" * 40,
                                            "github_run_id": "123", "github_run_attempt": "1"}),
                                encoding="utf-8")
            args = SimpleNamespace(
                source_script=str(source), source_metadata=str(metadata),
                output_script=str(root / "out.ps1"), output_metadata=str(root / "out.json"),
                expected_source_commit="a" * 40, expected_run_id="123",
                expected_run_attempt="1", case="original")
            with self.assertRaisesRegex(self.helper.PreparationError, "source body hash"):
                self.helper.prepare(args)
            self.assertFalse((root / "out.ps1").exists())

    def test_successful_preparation_records_variant_body_hash(self):
        with tempfile.TemporaryDirectory(prefix="katana-windows-diagnostic-") as name:
            root = Path(name)
            source = root / "source.ps1"
            metadata = root / "source.json"
            source.write_text(self.body, encoding="utf-8")
            metadata.write_text(json.dumps({"powershell_body_sha256": hashlib.sha256(
                self.body.encode()).hexdigest(), "version": "v0.22.42", "source_commit": "a" * 40,
                "github_run_id": "123", "github_run_attempt": "1", "main_sha256": "b" * 64}),
                                encoding="utf-8")
            output = root / "out.ps1"
            output_metadata = root / "out.json"
            args = SimpleNamespace(source_script=str(source), source_metadata=str(metadata),
                                   output_script=str(output), output_metadata=str(output_metadata),
                                   expected_source_commit="a" * 40, expected_run_id="123",
                                   expected_run_attempt="1", case="reduced-native-trace")
            self.helper.prepare(args)
            self.assertEqual(output.read_text(encoding="utf-8"),
                             self.helper.diagnostic_variant(self.body, "reduced-native-trace"))
            result = json.loads(output_metadata.read_text(encoding="utf-8"))
            actual_hash = hashlib.sha256(output.read_bytes()).hexdigest()
            self.assertEqual(result["diagnostic_body_sha256"], actual_hash)
            self.assertEqual(source.read_text(encoding="utf-8"), self.body)

    def test_outputs_cannot_overwrite_input_files(self):
        with tempfile.TemporaryDirectory(prefix="katana-windows-diagnostic-") as name:
            root = Path(name)
            source = root / "source.ps1"
            metadata = root / "source.json"
            original_metadata = json.dumps({"powershell_body_sha256": hashlib.sha256(
                self.body.encode()).hexdigest(), "version": "v0.22.42", "source_commit": "a" * 40,
                "github_run_id": "123", "github_run_attempt": "1"})
            source.write_text(self.body, encoding="utf-8")
            metadata.write_text(original_metadata, encoding="utf-8")
            args = SimpleNamespace(source_script=str(source), source_metadata=str(metadata),
                                   output_script=str(source), output_metadata=str(root / "out.json"),
                                   expected_source_commit="a" * 40, expected_run_id="123",
                                   expected_run_attempt="1", case="original")
            with self.assertRaisesRegex(self.helper.PreparationError, "must not overwrite"):
                self.helper.prepare(args)
            args.output_script = str(root / "out.ps1")
            args.output_metadata = str(metadata)
            with self.assertRaisesRegex(self.helper.PreparationError, "must not overwrite"):
                self.helper.prepare(args)
            self.assertEqual(source.read_text(encoding="utf-8"), self.body)
            self.assertEqual(metadata.read_text(encoding="utf-8"), original_metadata)

    def test_existing_output_files_are_not_overwritten(self):
        with tempfile.TemporaryDirectory(prefix="katana-windows-diagnostic-") as name:
            root = Path(name)
            source = root / "source.ps1"
            metadata = root / "source.json"
            body_hash = hashlib.sha256(self.body.encode()).hexdigest()
            source.write_text(self.body, encoding="utf-8")
            metadata.write_text(json.dumps({"powershell_body_sha256": body_hash,
                                            "version": "v0.22.42", "source_commit": "a" * 40,
                                            "github_run_id": "123", "github_run_attempt": "1"}),
                                encoding="utf-8")
            args = SimpleNamespace(source_script=str(source), source_metadata=str(metadata),
                                   output_script=str(root / "out.ps1"),
                                   output_metadata=str(root / "out.json"),
                                   expected_source_commit="a" * 40, expected_run_id="123",
                                   expected_run_attempt="1", case="original")
            output_script = Path(args.output_script)
            output_metadata = Path(args.output_metadata)
            output_script.write_text("preserve script\n", encoding="utf-8")
            with self.assertRaisesRegex(self.helper.PreparationError, "already exists"):
                self.helper.prepare(args)
            self.assertEqual(output_script.read_text(encoding="utf-8"), "preserve script\n")
            output_script.unlink()
            output_metadata.write_text("preserve metadata\n", encoding="utf-8")
            with self.assertRaisesRegex(self.helper.PreparationError, "already exists"):
                self.helper.prepare(args)
            self.assertEqual(source.read_text(encoding="utf-8"), self.body)
            self.assertEqual(output_metadata.read_text(encoding="utf-8"), "preserve metadata\n")


if __name__ == "__main__":
    unittest.main()
