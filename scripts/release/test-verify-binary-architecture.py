#!/usr/bin/env python3

import importlib.util
import io
import plistlib
import struct
import sys
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("verify-binary-architecture.py")
SPEC = importlib.util.spec_from_file_location("verify_binary_architecture", MODULE_PATH)
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


def elf_x86_64() -> bytes:
    data = bytearray(64)
    data[:6] = b"\x7fELF\x02\x01"
    struct.pack_into("<H", data, 18, 0x003E)
    return bytes(data)


def pe_x86_64() -> bytes:
    data = bytearray(256)
    data[:2] = b"MZ"
    struct.pack_into("<I", data, 0x3C, 128)
    data[128:132] = b"PE\0\0"
    struct.pack_into("<H", data, 132, 0x8664)
    return bytes(data)


def fat_macho() -> bytes:
    return b"\xca\xfe\xba\xbe" + struct.pack(
        ">I5I5I", 2, 0x0100000C, 0, 0, 0, 0, 0x01000007, 0, 0, 0, 0
    )


def thin_macho(cpu_type: int, major: int, minor: int = 0) -> bytes:
    command = struct.pack("<6I", 0x32, 24, 1, major << 16 | minor << 8, 0, 0)
    header = struct.pack("<8I", 0xFEEDFACF, cpu_type, 0, 2, 1, len(command), 0, 0)
    return header + command


def fat_macho_with_minimum_version(major: int, minor: int = 0) -> bytes:
    slices = [
        (0x0100000C, thin_macho(0x0100000C, major, minor)),
        (0x01000007, thin_macho(0x01000007, major, minor)),
    ]
    header_size = 8 + 20 * len(slices)
    entries = []
    payload = bytearray()
    offset = header_size
    for cpu_type, thin in slices:
        entries.append(struct.pack(">5I", cpu_type, 0, offset, len(thin), 0))
        payload.extend(thin)
        offset += len(thin)
    return b"\xca\xfe\xba\xbe" + struct.pack(">I", len(slices)) + b"".join(entries) + payload


class ArchitectureContractTests(unittest.TestCase):
    def test_detects_declared_architectures(self):
        self.assertEqual(MODULE.detect_architectures(elf_x86_64()), {"x86_64"})
        self.assertEqual(MODULE.detect_architectures(pe_x86_64()), {"x86_64"})
        self.assertEqual(MODULE.detect_architectures(fat_macho()), {"arm64", "x86_64"})

    def test_linux_archive_requires_executable_main_and_worker(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "linux.tar.gz"
            with tarfile.open(path, "w:gz") as archive:
                for name in MODULE.EXPECTED_MEMBERS["linux"]:
                    info = tarfile.TarInfo(name)
                    info.mode = 0o755
                    payload = elf_x86_64()
                    info.size = len(payload)
                    archive.addfile(info, io.BytesIO(payload))
            self.assertEqual(len(MODULE.verify("linux", path)), 2)

    def test_macos_archive_rejects_thin_binary(self):
        thin = b"\xcf\xfa\xed\xfe" + struct.pack("<I", 0x0100000C) + bytes(56)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "macos.zip"
            with zipfile.ZipFile(path, "w") as archive:
                for name in MODULE.EXPECTED_MEMBERS["macos"]:
                    info = zipfile.ZipInfo(name)
                    info.external_attr = 0o100755 << 16
                    archive.writestr(info, thin)
            with self.assertRaisesRegex(MODULE.ContractError, "required=.*x86_64"):
                MODULE.verify("macos", path)

    def test_detects_minimum_version_for_every_macho_slice(self):
        self.assertEqual(
            MODULE.detect_macos_minimum_versions(fat_macho_with_minimum_version(13)),
            {"arm64": (13, 0, 0), "x86_64": (13, 0, 0)},
        )

    def test_macos_archive_requires_matching_binary_and_plist_minimum(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "macos.zip"
            with zipfile.ZipFile(path, "w") as archive:
                for name in MODULE.EXPECTED_MEMBERS["macos"]:
                    info = zipfile.ZipInfo(name)
                    info.external_attr = 0o100755 << 16
                    archive.writestr(info, fat_macho_with_minimum_version(13))
                archive.writestr(
                    "KatanA Desktop.app/Contents/Info.plist",
                    plistlib.dumps({"LSMinimumSystemVersion": "13.0"}),
                )
            messages = MODULE.verify("macos", path)
            self.assertEqual(len(messages), 2)
            self.assertTrue(all("minos=13.0.0" in message for message in messages))

    def test_macos_archive_rejects_binary_below_declared_minimum(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "macos.zip"
            with zipfile.ZipFile(path, "w") as archive:
                for name in MODULE.EXPECTED_MEMBERS["macos"]:
                    info = zipfile.ZipInfo(name)
                    info.external_attr = 0o100755 << 16
                    archive.writestr(info, fat_macho_with_minimum_version(12))
                archive.writestr(
                    "KatanA Desktop.app/Contents/Info.plist",
                    plistlib.dumps({"LSMinimumSystemVersion": "13.0"}),
                )
            with self.assertRaisesRegex(MODULE.ContractError, "must equal 13.0.0"):
                MODULE.verify("macos", path)

    def test_macos_archive_rejects_plist_minimum_mismatch(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "macos.zip"
            with zipfile.ZipFile(path, "w") as archive:
                for name in MODULE.EXPECTED_MEMBERS["macos"]:
                    info = zipfile.ZipInfo(name)
                    info.external_attr = 0o100755 << 16
                    archive.writestr(info, fat_macho_with_minimum_version(13))
                archive.writestr(
                    "KatanA Desktop.app/Contents/Info.plist",
                    plistlib.dumps({"LSMinimumSystemVersion": "12.0"}),
                )
            with self.assertRaisesRegex(
                MODULE.ContractError,
                "LSMinimumSystemVersion='12.0' required='13.0'",
            ):
                MODULE.verify("macos", path)


if __name__ == "__main__":
    unittest.main()
