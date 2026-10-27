"""固定DXC診断assetの拒否境界を、ネットワークなしで検証する。"""

import importlib.util
import json
from pathlib import Path
import struct
import tempfile
import unittest
import zipfile


SOURCE = Path(__file__).with_name("windows-dx12-compiler-assets.py")
SPEC = importlib.util.spec_from_file_location("dxc_asset_contract", SOURCE)
assert SPEC is not None and SPEC.loader is not None
ASSETS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ASSETS)
ROOT = Path(__file__).resolve().parents[2]


def load_helper(name):
    path = SOURCE.with_name(name)
    spec = importlib.util.spec_from_file_location(path.stem.replace("-", "_"), path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class DxcAssetAdmissionTests(unittest.TestCase):
    def test_official_asset_and_dll_pins_match_the_independently_checked_release(self):
        self.assertEqual(ASSETS.ASSET_ID, 596913261)
        self.assertEqual(ASSETS.ASSET_SHA256,
                         "ad31b1fc8443175d204f77a611fdb3ef2ec42759bdc2f1167368de24a4a7e7f1")
        self.assertEqual(ASSETS.FILES["bin/x64/dxcompiler.dll"]["sha256"],
                         "0689e2fee0a1c9df8662a59680469f24e2a3ad7ce6bbbcc9284f7a83172bfc59")
        self.assertEqual(ASSETS.FILES["bin/x64/dxil.dll"]["sha256"],
                         "64be2368211d257a038a7d4ea6bb3e24c878eb8d588c7b8414fccf759b69c2ac")

    def test_compiler_dispatch_preserves_the_validated_smoke(self):
        helper = load_helper("prepare-windows-startup-diagnostics.py")
        comparison = load_helper("prepare-windows-startup-comparison.py")
        body = comparison.extract_original_smoke_block(ROOT / ".github/workflows/build-and-release.yml")[1]
        self.assertEqual(helper.CASE_LABELS["dx12-compiler"],
                         ("original-before", "dx12-dxc", "original-after"))
        self.assertEqual(helper.diagnostic_variant(body, "dx12-compiler"),
                         body.replace(helper.SMOKE_DIRECTORY_LINE, helper.ISOLATED_DIRECTORY_LINE, 1))
        environment = SOURCE.with_name("windows-dx12-compiler-environment.ps1").read_text(encoding="utf-8")
        runner = SOURCE.with_name("run-windows-startup-diagnostic-cases.ps1").read_text(encoding="utf-8")
        preparer = SOURCE.with_name("prepare-windows-startup-diagnostic-cases.ps1").read_text(encoding="utf-8")
        self.assertIn("DynamicDxc", environment)
        self.assertIn("verify --receipt $receiptPath --expected-root", environment)
        self.assertIn("windows-dx12-compiler-assets.py prepare", preparer)
        self.assertIn("Enable-WindowsDx12Compiler", runner)
        self.assertIn("Restore-WindowsDx12CompilerEnvironment", runner)
        self.assertIn("-CaseLabel $caseLabel", runner)
        self.assertIn("comparison_mode = $env:WINDOWS_COMPARISON", runner)
        workflow = (ROOT / ".github/workflows/html-failure-diagnostics.yml").read_text(encoding="utf-8")
        self.assertIn("- dx12-compiler", workflow)
        capture = SOURCE.with_name("capture-windows-startup-wait-stacks.ps1").read_text(encoding="utf-8")
        self.assertIn("dx12-dxc", capture)
        profile = SOURCE.with_name("windows-startup-wait-stacks.wprp").read_text(encoding="utf-8")
        self.assertIn("Loader", profile)

    def receipt(self, root):
        return {
            "asset_url": ASSETS.ASSET_URL,
            "asset_id": ASSETS.ASSET_ID,
            "archive_sha256": ASSETS.ASSET_SHA256,
            "compiler_bin_path": str(root / "bin" / "x64"),
        }

    def test_existing_destination_is_rejected_without_touching_its_contents(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            marker = root / "keep.txt"
            marker.write_text("preserve", encoding="utf-8")
            with self.assertRaisesRegex(ASSETS.AssetError, "fresh"):
                ASSETS.prepare(root)
            self.assertEqual(marker.read_text(encoding="utf-8"), "preserve")

    def test_non_object_and_changed_official_identity_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "receipt.json"
            values = [None, [], "invalid"]
            for key in ("asset_url", "asset_id", "archive_sha256"):
                value = self.receipt(root)
                value[key] = "different"
                values.append(value)
            for value in values:
                with self.subTest(value=value):
                    path.write_text(json.dumps(value), encoding="utf-8")
                    with self.assertRaises(ASSETS.AssetError):
                        ASSETS.validate_receipt(path, root)

    def test_receipt_cannot_select_a_dll_directory_outside_its_destination(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            receipt = self.receipt(root)
            receipt["compiler_bin_path"] = str(root / "outside")
            path = root / "receipt.json"
            path.write_text(json.dumps(receipt), encoding="utf-8")
            with self.assertRaisesRegex(ASSETS.AssetError, "outside"):
                ASSETS.validate_receipt(path, root)

    def test_backslash_members_are_selected_but_normalized_aliases_are_rejected(self):
        members = [zipfile.ZipInfo(name.replace("/", "\\")) for name in ASSETS.FILES]
        self.assertEqual(set(ASSETS.collect_required_members(members)), set(ASSETS.FILES))
        duplicate = zipfile.ZipInfo(next(iter(ASSETS.FILES)))
        with self.assertRaisesRegex(ASSETS.AssetError, "duplicate"):
            ASSETS.collect_required_members([*members, duplicate])
        with self.assertRaisesRegex(ASSETS.AssetError, "required DLL"):
            ASSETS.collect_required_members(members[:1])

    def test_unsafe_archive_paths_are_rejected(self):
        for name in ("../dxcompiler.dll", "bin/../../x.dll", "C:\\x.dll", "/x.dll"):
            with self.subTest(name=name), self.assertRaises(ASSETS.AssetError):
                ASSETS.normalized_member(name)

    def test_malformed_and_wrong_architecture_dlls_are_rejected(self):
        data = bytearray(128)
        data[:2] = b"MZ"
        struct.pack_into("<I", data, 0x3C, 64)
        data[64:68] = b"PE\0\0"
        struct.pack_into("<H", data, 68, 0x8664)
        ASSETS.validate_pe_amd64(bytes(data), "x64.dll")
        for machine in (0x014C, 0xAA64):
            struct.pack_into("<H", data, 68, machine)
            with self.subTest(machine=machine), self.assertRaises(ASSETS.AssetError):
                ASSETS.validate_pe_amd64(bytes(data), "other.dll")
        for malformed in (b"MZ", bytes(128), b"MZ" + bytes(126)):
            with self.subTest(data=malformed[:4]), self.assertRaises(ASSETS.AssetError):
                ASSETS.validate_pe_amd64(malformed, "broken.dll")


if __name__ == "__main__":
    unittest.main()
