#!/usr/bin/env python3
"""検証済みsmoke本文からの派生ファイル作成と上書き防止を検査する。"""

import hashlib
import importlib.util
import json
import tempfile
from pathlib import Path
from types import SimpleNamespace
import unittest


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


class WindowsStartupDiagnosticPreparationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.helper = load_module("windows_startup_diagnostics", HELPER_PATH)
        cls.comparison = load_module("windows_startup_comparison", COMPARISON_PATH)
        cls.body = cls.comparison.extract_original_smoke_block(WORKFLOW)[1]

    def test_preparation_requires_original_body_hash_and_run_binding(self):
        with tempfile.TemporaryDirectory(prefix="katana-windows-diagnostic-") as name:
            root = Path(name)
            source, metadata = root / "source.ps1", root / "source.json"
            source.write_text(self.body, encoding="utf-8")
            metadata.write_text(json.dumps({"powershell_body_sha256": "0" * 64,
                "version": "v0.22.42", "source_commit": "a" * 40,
                "github_run_id": "123", "github_run_attempt": "1"}), encoding="utf-8")
            args = SimpleNamespace(source_script=str(source), source_metadata=str(metadata),
                output_script=str(root / "out.ps1"), output_metadata=str(root / "out.json"),
                expected_source_commit="a" * 40, expected_run_id="123", expected_run_attempt="1",
                case="original", run_label="original", dx12_value=None)
            with self.assertRaisesRegex(self.helper.PreparationError, "source body hash"):
                self.helper.prepare(args)
            self.assertFalse((root / "out.ps1").exists())

    def test_successful_preparation_records_variant_body_hash(self):
        with tempfile.TemporaryDirectory(prefix="katana-windows-diagnostic-") as name:
            root = Path(name)
            source, metadata = root / "source.ps1", root / "source.json"
            source.write_text(self.body, encoding="utf-8")
            metadata.write_text(json.dumps({"powershell_body_sha256": hashlib.sha256(
                self.body.encode()).hexdigest(), "version": "v0.22.42", "source_commit": "a" * 40,
                "github_run_id": "123", "github_run_attempt": "1", "main_sha256": "b" * 64}),
                encoding="utf-8")
            output, output_metadata = root / "out.ps1", root / "out.json"
            args = SimpleNamespace(source_script=str(source), source_metadata=str(metadata),
                output_script=str(output), output_metadata=str(output_metadata),
                expected_source_commit="a" * 40, expected_run_id="123", expected_run_attempt="1",
                case="reduced-native-trace", run_label="reduced-native-trace", dx12_value=None)
            self.helper.prepare(args)
            self.assertEqual(output.read_text(encoding="utf-8"), self.helper.diagnostic_variant(
                self.body, "reduced-native-trace"))
            result = json.loads(output_metadata.read_text(encoding="utf-8"))
            self.assertEqual(result["diagnostic_body_sha256"], hashlib.sha256(output.read_bytes()).hexdigest())
            self.assertEqual(result["run_label"], "reduced-native-trace")
            self.assertIsNone(result["dx12_environment_value"])
            self.assertEqual(source.read_text(encoding="utf-8"), self.body)

    def test_dx12_preparation_records_case_semantics_and_selected_value(self):
        with tempfile.TemporaryDirectory(prefix="katana-windows-dx12-diagnostic-") as name:
            root = Path(name)
            source, metadata = root / "source.ps1", root / "source.json"
            source.write_text(self.body, encoding="utf-8")
            body_hash = hashlib.sha256(self.body.encode()).hexdigest()
            metadata.write_text(json.dumps({"powershell_body_sha256": body_hash,
                "version": "v0.22.42", "source_commit": "a" * 40,
                "github_run_id": "123", "github_run_attempt": "1"}), encoding="utf-8")
            output, output_metadata = root / "dx12.ps1", root / "dx12.json"
            args = SimpleNamespace(source_script=str(source), source_metadata=str(metadata),
                output_script=str(output), output_metadata=str(output_metadata),
                expected_source_commit="a" * 40, expected_run_id="123", expected_run_attempt="1",
                case="dx12-waiting", run_label="dx12-dont-wait", dx12_value="DontWait")
            self.helper.prepare(args)
            result = json.loads(output_metadata.read_text(encoding="utf-8"))
            self.assertEqual((result["diagnostic_case"], result["run_label"],
                result["dx12_environment_value"]), ("dx12-waiting", "dx12-dont-wait", "DontWait"))

    def test_outputs_cannot_overwrite_input_files(self):
        with tempfile.TemporaryDirectory(prefix="katana-windows-diagnostic-") as name:
            root = Path(name)
            source, metadata = root / "source.ps1", root / "source.json"
            original_metadata = json.dumps({"powershell_body_sha256": hashlib.sha256(
                self.body.encode()).hexdigest(), "version": "v0.22.42", "source_commit": "a" * 40,
                "github_run_id": "123", "github_run_attempt": "1"})
            source.write_text(self.body, encoding="utf-8")
            metadata.write_text(original_metadata, encoding="utf-8")
            args = SimpleNamespace(source_script=str(source), source_metadata=str(metadata),
                output_script=str(source), output_metadata=str(root / "out.json"),
                expected_source_commit="a" * 40, expected_run_id="123", expected_run_attempt="1",
                case="original", run_label="original", dx12_value=None)
            with self.assertRaisesRegex(self.helper.PreparationError, "must not overwrite"):
                self.helper.prepare(args)
            args.output_script, args.output_metadata = str(root / "out.ps1"), str(metadata)
            with self.assertRaisesRegex(self.helper.PreparationError, "must not overwrite"):
                self.helper.prepare(args)
            self.assertEqual(source.read_text(encoding="utf-8"), self.body)
            self.assertEqual(metadata.read_text(encoding="utf-8"), original_metadata)

    def test_existing_output_files_are_not_overwritten(self):
        with tempfile.TemporaryDirectory(prefix="katana-windows-diagnostic-") as name:
            root = Path(name)
            source, metadata = root / "source.ps1", root / "source.json"
            body_hash = hashlib.sha256(self.body.encode()).hexdigest()
            source.write_text(self.body, encoding="utf-8")
            metadata.write_text(json.dumps({"powershell_body_sha256": body_hash,
                "version": "v0.22.42", "source_commit": "a" * 40,
                "github_run_id": "123", "github_run_attempt": "1"}), encoding="utf-8")
            output, output_metadata = root / "out.ps1", root / "out.json"
            args = SimpleNamespace(source_script=str(source), source_metadata=str(metadata),
                output_script=str(output), output_metadata=str(output_metadata),
                expected_source_commit="a" * 40, expected_run_id="123", expected_run_attempt="1",
                case="original", run_label="original", dx12_value=None)
            output.write_text("preserve script\n", encoding="utf-8")
            with self.assertRaisesRegex(self.helper.PreparationError, "already exists"):
                self.helper.prepare(args)
            self.assertEqual(output.read_text(encoding="utf-8"), "preserve script\n")
            output.unlink()
            output_metadata.write_text("preserve metadata\n", encoding="utf-8")
            with self.assertRaisesRegex(self.helper.PreparationError, "already exists"):
                self.helper.prepare(args)
            self.assertEqual(source.read_text(encoding="utf-8"), self.body)
            self.assertEqual(output_metadata.read_text(encoding="utf-8"), "preserve metadata\n")


if __name__ == "__main__":
    unittest.main()
