#!/usr/bin/env python3
"""診断派生scriptが必須条件と入力証跡を保つことを検査する。"""

import hashlib
import importlib.util
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
HELPER_PATH = ROOT / "scripts/release/prepare-windows-startup-diagnostics.py"
COMPARISON_PATH = ROOT / "scripts/release/prepare-windows-startup-comparison.py"
WORKFLOW = ROOT / ".github/workflows/build-and-release.yml"
DIAGNOSTIC_WORKFLOW = ROOT / ".github/workflows/html-failure-diagnostics.yml"
PREPARE_CASES = ROOT / "scripts/release/prepare-windows-startup-diagnostic-cases.ps1"
RUN_CASES = ROOT / "scripts/release/run-windows-startup-diagnostic-cases.ps1"
PREPARATION_TESTS = ROOT / "scripts/release/test-windows-startup-diagnostic-preparation.py"


def load_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise AssertionError(f"could not load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def load_tests(loader, tests, pattern):
    preparation_tests = load_module("windows_startup_diagnostic_preparation_tests", PREPARATION_TESTS)
    tests.addTests(loader.loadTestsFromModule(preparation_tests))
    return tests


class WindowsStartupDiagnosticTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.helper = load_module("windows_startup_diagnostics", HELPER_PATH)
        cls.comparison = load_module("windows_startup_comparison", COMPARISON_PATH)
        cls.body = cls.comparison.extract_original_smoke_block(WORKFLOW)[1]
        cls.diagnostics_workflow = DIAGNOSTIC_WORKFLOW.read_text(encoding="utf-8")
        cls.prepare_cases = PREPARE_CASES.read_text(encoding="utf-8")
        cls.run_cases = RUN_CASES.read_text(encoding="utf-8")

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

    def test_dx12_wait_cases_change_only_the_selected_wait_mode(self):
        for value in self.helper.SUPPORTED_DX12_VALUES:
            with self.subTest(value=value):
                result = self.helper.diagnostic_variant(self.body, "dx12-waiting", value)
                expected = self.body.replace(self.helper.SMOKE_DIRECTORY_LINE,
                                              self.helper.ISOLATED_DIRECTORY_LINE, 1)
                expected = expected.replace(self.helper.DEBUG_LINE,
                    self.helper.DEBUG_LINE + self.helper.DX12_ENVIRONMENT_LINE.format(value=value), 1)
                self.assertEqual(result, expected)
                self.assert_protected_contract(result)

    def test_dx12_presentation_case_only_sets_visual_system(self):
        result = self.helper.diagnostic_variant(
            self.body, "dx12-presentation", presentation_system="DxgiFromVisual")
        expected = self.body.replace(self.helper.SMOKE_DIRECTORY_LINE,
                                     self.helper.ISOLATED_DIRECTORY_LINE, 1)
        expected = expected.replace(self.helper.DEBUG_LINE, self.helper.DEBUG_LINE +
                                    self.helper.PRESENTATION_ENVIRONMENT_LINE.format(
                                        value="DxgiFromVisual"), 1)
        self.assertEqual(result, expected)
        self.assert_protected_contract(result)
        with self.assertRaisesRegex(self.helper.PreparationError, "unsupported DX12 presentation system"):
            self.helper.diagnostic_variant(self.body, "dx12-presentation", presentation_system="Visual")

    def test_dx12_run_labels_separate_semantics_and_restore_process_environment(self):
        windows = self.diagnostics_workflow.split("  windows-startup-diagnostics:", 1)[1]
        for label in ("original-before", "dx12-dont-wait", "dx12-none", "original-after"):
            self.assertIn(label, self.prepare_cases)
        self.assertIn("finally {", self.run_cases)
        self.assertIn("$originalSetting", self.run_cases)
        self.assertIn("dx12_environment_value = $caseMetadata.dx12_environment_value", self.run_cases)

    def test_dx12_mode_and_run_label_must_match(self):
        for case, value, message in (
            ("dx12-waiting", "Wait", "unsupported DX12 waitable-object value"),
            ("dx12-waiting", None, "unsupported DX12 waitable-object value"),
            ("original", "None", "DX12 environment value is only valid"),
        ):
            with self.subTest(case=case, value=value), self.assertRaisesRegex(
                    self.helper.PreparationError, message):
                self.helper.diagnostic_variant(self.body, case, value)
        self.assertNotIn("original", self.helper.CASE_LABELS["dx12-waiting"])
        with self.assertRaisesRegex(self.helper.PreparationError, "only valid for dx12-presentation"):
            self.helper.diagnostic_variant(self.body, "original", presentation_system="DxgiFromVisual")

    def test_unknown_control_cases_are_rejected(self):
        for case in ("no-redirection-bitmap", "control-frame-change"):
            with self.subTest(case=case), self.assertRaisesRegex(self.helper.PreparationError,
                                                                "unsupported diagnostic case"):
                self.helper.diagnostic_variant(self.body, case)

    def test_wait_stacks_keeps_original_smoke_and_separate_owned_trace(self):
        self.assertIn("- wait-stacks", self.diagnostics_workflow)
        self.assertIn("$env:WINDOWS_COMPARISON -eq 'wait-stacks'", self.prepare_cases)
        self.assertIn("$env:WINDOWS_COMPARISON -eq 'wait-stacks'", self.run_cases)
        self.assertIn("capture-windows-startup-wait-stacks.ps1", self.run_cases)
        self.assertIn("wait_stack_trace_error = $traceFailure", self.run_cases)
        capture = (ROOT / "scripts/release/capture-windows-startup-wait-stacks.ps1").read_text(encoding="utf-8")
        self.assertIn("Get-FileHash", capture)
        self.assertIn("$identity.main_sha256", capture)
        self.assertIn("-WaitStacks", capture)
        self.assertLess(capture.index("Get-FileHash"), capture.index("-WaitStacks"))
        self.assert_protected_contract(self.helper.diagnostic_variant(self.body, "original"))

    def test_presentation_mode_traces_default_and_visual_cases_separately(self):
        self.assertIn("- dx12-presentation", self.diagnostics_workflow)
        self.assertIn("dx12-visual", self.prepare_cases)
        self.assertIn("dx12-presentation", self.run_cases)
        self.assertIn("WGPU_DX12_PRESENTATION_SYSTEM", self.run_cases)
        self.assertIn("-CaseLabel $caseLabel", self.run_cases)
        self.assertIn("presentation_wait_stack_trace_errors", self.run_cases)
        self.assertIn("$caseLabel -in @('original-before', 'dx12-visual')", self.run_cases)
        self.assertIn("$traceFailures.Count -gt 0", self.run_cases)
        capture = (ROOT / "scripts/release/capture-windows-startup-wait-stacks.ps1").read_text(encoding="utf-8")
        self.assertIn("ValidateSet('original', 'original-before', 'dx12-visual', 'original-after')", capture)
        self.assertIn("DxgiFromVisual", capture)
        self.assertIn("finally {", capture)

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
        self.assertIn("windows_comparison:\n        description: Windows diagnostic comparison mode\n        required: false\n        type: choice", self.diagnostics_workflow)
        self.assertIn("default: logging", self.diagnostics_workflow)
        self.assertIn("- dx12-presentation", self.diagnostics_workflow)
        self.assertIn("inputs.windows_run_attempt != '')", windows.split("runs-on:", 1)[0])
        self.assertNotIn("inputs.windows_comparison", windows.split("runs-on:", 1)[0])

    def test_unexpected_or_duplicated_source_lines_are_rejected(self):
        for body in (self.body.replace(self.helper.SMOKE_DIRECTORY_LINE, "", 1),
                     self.body + self.helper.RUN_LOG_LINE,
                     self.body.replace(self.helper.RUN_LOG_LINE, "", 1)):
            with self.subTest(body_hash=hashlib.sha256(body.encode()).hexdigest()), \
                    self.assertRaises(self.helper.PreparationError):
                self.helper.diagnostic_variant(body, "reduced-native-trace")



if __name__ == "__main__":
    unittest.main()
