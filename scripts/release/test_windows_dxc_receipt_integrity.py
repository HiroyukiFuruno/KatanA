"""成功する小型PE fixtureから、receiptとDLL改変の拒否を検証する。"""

from contextlib import contextmanager
import hashlib
import importlib.util
import json
from pathlib import Path
import struct
import tempfile
import unittest
from unittest.mock import patch


SPEC = importlib.util.spec_from_file_location(
    "dxc_receipt_integrity", Path(__file__).with_name("windows-dx12-compiler-assets.py"))
assert SPEC is not None and SPEC.loader is not None
ASSETS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ASSETS)


@contextmanager
def valid_fixture():
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        compiler = root / "bin" / "x64"
        compiler.mkdir(parents=True)
        data = bytearray(128)
        data[:2] = b"MZ"
        struct.pack_into("<I", data, 0x3C, 64)
        data[64:68] = b"PE\0\0"
        struct.pack_into("<H", data, 68, 0x8664)
        pair = {name: {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}
                for name in ASSETS.FILES}
        for name in pair:
            (compiler / Path(name).name).write_bytes(data)
        receipt = {"asset_url": ASSETS.ASSET_URL, "asset_id": ASSETS.ASSET_ID,
                   "archive_sha256": ASSETS.ASSET_SHA256, "files": json.loads(json.dumps(pair)),
                   "compiler_bin_path": str(compiler.resolve())}
        path = root / "compiler-receipt.json"
        path.write_text(json.dumps(receipt), encoding="utf-8")
        with patch.dict(ASSETS.FILES, pair, clear=True):
            ASSETS.validate_receipt(path, root)
            yield root, path, receipt, compiler / "dxcompiler.dll"


class DxcReceiptIntegrityTests(unittest.TestCase):
    def test_same_size_dll_tampering_is_rejected_after_a_valid_receipt(self):
        with valid_fixture() as (root, receipt, _, dll):
            data = bytearray(dll.read_bytes())
            data[-1] ^= 1
            dll.write_bytes(data)
            with self.assertRaisesRegex(ASSETS.AssetError, "hash differs"):
                ASSETS.validate_receipt(receipt, root)

    def test_missing_dll_is_rejected_after_a_valid_receipt(self):
        with valid_fixture() as (root, receipt, _, dll):
            dll.rename(dll.with_suffix(".moved"))
            with self.assertRaisesRegex(ASSETS.AssetError, "missing or has wrong size"):
                ASSETS.validate_receipt(receipt, root)

    def test_truncated_dll_is_rejected_after_a_valid_receipt(self):
        with valid_fixture() as (root, receipt, _, dll):
            dll.write_bytes(dll.read_bytes()[:-1])
            with self.assertRaisesRegex(ASSETS.AssetError, "missing or has wrong size"):
                ASSETS.validate_receipt(receipt, root)

    def test_changed_receipt_file_metadata_is_rejected(self):
        with valid_fixture() as (root, receipt, value, _):
            value["files"]["bin/x64/dxcompiler.dll"]["sha256"] = "0" * 64
            receipt.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaisesRegex(ASSETS.AssetError, "metadata differs"):
                ASSETS.validate_receipt(receipt, root)


if __name__ == "__main__":
    unittest.main()
