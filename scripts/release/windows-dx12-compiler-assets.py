#!/usr/bin/env python3
"""Prepare isolated DXC diagnostic DLLs from a pinned release asset."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import struct
import urllib.error
import urllib.request
import zipfile
from pathlib import Path, PurePosixPath
from typing import Iterable
from windows_dx12_compiler_pins import ASSET_ID, ASSET_SHA256, ASSET_URL, FILES


MAX_ARCHIVE_BYTES = 512 * 1024 * 1024
RECEIPT_NAME = "compiler-receipt.json"


class AssetError(ValueError):
    """Pinned asset validation or extraction failed."""


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def normalized_member(name: str) -> str:
    normalized = name.replace("\\", "/")
    candidate = PurePosixPath(normalized)
    if candidate.is_absolute() or re.match(r"^[A-Za-z]:", normalized):
        raise AssetError(f"absolute archive member rejected: {name!r}")
    if ".." in candidate.parts:
        raise AssetError(f"traversal archive member rejected: {name!r}")
    return candidate.as_posix()


def collect_required_members(members: Iterable[zipfile.ZipInfo]) -> dict[str, zipfile.ZipInfo]:
    selected: dict[str, zipfile.ZipInfo] = {}
    seen: set[str] = set()
    for info in members:
        name = normalized_member(info.filename)
        if name in seen:
            raise AssetError(f"duplicate normalized archive member rejected: {name}")
        seen.add(name)
        if name in FILES:
            selected[name] = info
    if set(selected) != set(FILES):
        raise AssetError("official archive does not contain exactly the required DLL members")
    return selected


def validate_pe_amd64(data: bytes, label: str) -> None:
    if len(data) < 64 or data[:2] != b"MZ":
        raise AssetError(f"{label} is not a PE image")
    pe_offset = struct.unpack_from("<I", data, 0x3C)[0]
    if pe_offset + 6 > len(data) or data[pe_offset:pe_offset + 4] != b"PE\0\0":
        raise AssetError(f"{label} has an invalid PE header")
    machine = struct.unpack_from("<H", data, pe_offset + 4)[0]
    if machine != 0x8664:
        raise AssetError(f"{label} is not AMD64 (machine=0x{machine:04x})")


def validate_receipt(receipt_path: Path, expected_root: Path | None = None) -> dict[str, object]:
    if not receipt_path.is_file() or receipt_path.is_symlink():
        raise AssetError("compiler receipt must be a regular file")
    try:
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise AssetError(f"compiler receipt is invalid: {error}") from error
    if not isinstance(receipt, dict) or receipt.get("asset_url") != ASSET_URL or \
            receipt.get("asset_id") != ASSET_ID or receipt.get("archive_sha256") != ASSET_SHA256:
        raise AssetError("compiler receipt does not match the pinned official asset")
    compiler_dir = Path(str(receipt.get("compiler_bin_path", ""))).resolve()
    if compiler_dir != (receipt_path.parent / "bin" / "x64").resolve():
        raise AssetError("compiler DLL directory is outside the isolated asset destination")
    if expected_root is not None and compiler_dir != (expected_root / "bin" / "x64").resolve():
        raise AssetError("compiler receipt directory differs from the isolated destination")
    files = receipt.get("files")
    if not isinstance(files, dict) or set(files) != set(FILES):
        raise AssetError("compiler receipt file set differs from the pinned DLL pair")
    for relative, expected in FILES.items():
        recorded = files.get(relative)
        path = compiler_dir / Path(relative).name
        if not isinstance(recorded, dict) or recorded.get("sha256") != expected["sha256"] or \
                recorded.get("bytes") != expected["bytes"]:
            raise AssetError(f"compiler receipt metadata differs for {relative}")
        if not path.is_file() or path.is_symlink() or path.stat().st_size != expected["bytes"]:
            raise AssetError(f"pinned compiler DLL is missing or has wrong size: {relative}")
        if sha256_file(path) != expected["sha256"]:
            raise AssetError(f"pinned compiler DLL hash differs: {relative}")
        validate_pe_amd64(path.read_bytes(), relative)
    return receipt


def download_archive(destination: Path) -> Path:
    archive_path = destination / "official-dxc.zip"
    request = urllib.request.Request(ASSET_URL, headers={"User-Agent": "KatanA-Windows-Diagnostic"})
    total = 0
    digest = hashlib.sha256()
    with urllib.request.urlopen(request, timeout=120) as response, archive_path.open("xb") as output:
        while True:
            chunk = response.read(1024 * 1024)
            if not chunk:
                break
            total += len(chunk)
            if total > MAX_ARCHIVE_BYTES:
                raise AssetError("official DXC archive exceeded the bounded download size")
            digest.update(chunk)
            output.write(chunk)
    if digest.hexdigest() != ASSET_SHA256:
        raise AssetError("official DXC archive SHA-256 differs from the pinned asset")
    return archive_path


def extract_pair(archive_path: Path, destination: Path) -> dict[str, dict[str, object]]:
    with zipfile.ZipFile(archive_path) as archive:
        target_members = collect_required_members(archive.infolist())
        compiler_dir = destination / "bin" / "x64"
        compiler_dir.mkdir(parents=True)
        records = {}
        for relative, expected in FILES.items():
            data = archive.read(target_members[relative])
            if len(data) != expected["bytes"] or hashlib.sha256(data).hexdigest() != expected["sha256"]:
                raise AssetError(f"pinned compiler DLL content differs: {relative}")
            validate_pe_amd64(data, relative)
            with (compiler_dir / Path(relative).name).open("xb") as stream:
                stream.write(data)
            records[relative] = {"bytes": len(data), "sha256": expected["sha256"],
                                 "pe_machine": "0x8664 AMD64"}
    return records


def write_receipt(destination: Path, records: dict[str, dict[str, object]]) -> Path:
    receipt = {
        "release": "v1.9.2609",
        "asset_url": ASSET_URL,
        "asset_id": ASSET_ID,
        "archive_sha256": ASSET_SHA256,
        "compiler_bin_path": str((destination / "bin" / "x64").resolve()),
        "files": records,
    }
    receipt_path = destination / RECEIPT_NAME
    receipt_path.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    validate_receipt(receipt_path, destination.resolve())
    return receipt_path


def prepare(destination: Path) -> Path:
    if destination.exists() or destination.is_symlink():
        raise AssetError("DXC destination must be fresh and must not already exist")
    destination.mkdir(parents=True)
    archive_path = destination / "official-dxc.zip"
    try:
        archive_path = download_archive(destination)
        return write_receipt(destination, extract_pair(archive_path, destination))
    except Exception:
        shutil.rmtree(destination, ignore_errors=True)
        raise
    finally:
        archive_path.unlink(missing_ok=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    prepare_parser = subparsers.add_parser("prepare")
    prepare_parser.add_argument("--destination", required=True)
    verify_parser = subparsers.add_parser("verify")
    verify_parser.add_argument("--receipt", required=True)
    verify_parser.add_argument("--expected-root")
    args = parser.parse_args()
    try:
        if args.command == "prepare":
            print(prepare(Path(args.destination)))
        else:
            receipt = validate_receipt(Path(args.receipt),
                                       Path(args.expected_root) if args.expected_root else None)
            print(receipt["compiler_bin_path"])
    except (OSError, zipfile.BadZipFile, AssetError, urllib.error.URLError) as error:
        print(f"ERROR: pinned DXC asset: {error}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
