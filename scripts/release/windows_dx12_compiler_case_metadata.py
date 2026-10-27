"""診断caseへ固定DXC receiptを結び付ける。"""

from __future__ import annotations

import hashlib
import importlib.util
from pathlib import Path


def compiler_metadata(case: str, run_label: str, receipt_value: str | None) -> dict[str, object]:
    is_compiler_case = case == "dx12-compiler"
    if is_compiler_case != (receipt_value is not None):
        raise ValueError("pinned DXC receipt must be supplied only for dx12-compiler")
    if not is_compiler_case:
        return {"dx12_compiler": None, "dx12_compiler_receipt_sha256": None}
    if run_label != "dx12-dxc":
        raise ValueError("pinned DXC receipt is only valid for the dx12-dxc run label")
    asset_path = Path(__file__).with_name("windows-dx12-compiler-assets.py")
    spec = importlib.util.spec_from_file_location("windows_dx12_compiler_assets", asset_path)
    if spec is None or spec.loader is None:
        raise ValueError("could not load pinned DXC asset verifier")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    path = Path(receipt_value)
    receipt = module.validate_receipt(path)
    return {
        "dx12_compiler": receipt,
        "dx12_compiler_receipt_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
    }
