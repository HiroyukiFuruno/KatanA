#!/usr/bin/env python3
"""Verify release executables match KatanA's declared platform architectures."""

from __future__ import annotations

import argparse
import io
import plistlib
import struct
import tarfile
import zipfile
from pathlib import Path

CPU_ARCH = {
    0x003E: "x86_64",
    0x8664: "x86_64",
    0x01000007: "x86_64",
    0x0100000C: "arm64",
}

EXPECTED_MEMBERS = {
    "linux": ("KatanA", "kdv-office-worker"),
    "windows": ("KatanA.exe", "kdv-office-worker.exe"),
    "macos": (
        "KatanA Desktop.app/Contents/MacOS/KatanA",
        "KatanA Desktop.app/Contents/MacOS/kdv-office-worker",
    ),
}
MINIMUM_MACOS_VERSION = (13, 0, 0)
LC_VERSION_MIN_MACOSX = 0x24
LC_BUILD_VERSION = 0x32


class ContractError(RuntimeError):
    """Raised when a release binary violates the architecture contract."""


def read_u32(data: bytes, offset: int, endian: str) -> int:
    return struct.unpack_from(f"{endian}I", data, offset)[0]


def detect_architectures(data: bytes) -> set[str]:
    if data.startswith(b"\x7fELF"):
        endian = "<" if data[5] == 1 else ">"
        return {CPU_ARCH.get(struct.unpack_from(f"{endian}H", data, 18)[0], "unknown")}
    if data.startswith(b"MZ"):
        pe_offset = struct.unpack_from("<I", data, 0x3C)[0]
        if data[pe_offset : pe_offset + 4] != b"PE\0\0":
            raise ContractError("invalid PE signature")
        return {CPU_ARCH.get(struct.unpack_from("<H", data, pe_offset + 4)[0], "unknown")}
    return detect_macho_architectures(data)


def detect_macho_architectures(data: bytes) -> set[str]:
    magic = data[:4]
    if magic in (b"\xcf\xfa\xed\xfe", b"\xce\xfa\xed\xfe"):
        return {CPU_ARCH.get(read_u32(data, 4, "<"), "unknown")}
    if magic in (b"\xfe\xed\xfa\xcf", b"\xfe\xed\xfa\xce"):
        return {CPU_ARCH.get(read_u32(data, 4, ">"), "unknown")}
    if magic in (b"\xca\xfe\xba\xbe", b"\xca\xfe\xba\xbf"):
        return read_fat_macho(data, ">", magic == b"\xca\xfe\xba\xbf")
    if magic in (b"\xbe\xba\xfe\xca", b"\xbf\xba\xfe\xca"):
        return read_fat_macho(data, "<", magic == b"\xbf\xba\xfe\xca")
    raise ContractError(f"unsupported executable magic {magic.hex()}")


def detect_macos_minimum_versions(data: bytes) -> dict[str, tuple[int, int, int]]:
    versions = {}
    for architecture, thin in macho_slices(data):
        versions[architecture] = thin_macho_minimum_version(thin)
    return versions


def macho_slices(data: bytes) -> list[tuple[str, bytes]]:
    magic = data[:4]
    if magic in (b"\xcf\xfa\xed\xfe", b"\xce\xfa\xed\xfe"):
        return [(CPU_ARCH.get(read_u32(data, 4, "<"), "unknown"), data)]
    if magic in (b"\xfe\xed\xfa\xcf", b"\xfe\xed\xfa\xce"):
        return [(CPU_ARCH.get(read_u32(data, 4, ">"), "unknown"), data)]
    if magic in (b"\xca\xfe\xba\xbe", b"\xca\xfe\xba\xbf"):
        return read_fat_macho_slices(data, ">", magic == b"\xca\xfe\xba\xbf")
    if magic in (b"\xbe\xba\xfe\xca", b"\xbf\xba\xfe\xca"):
        return read_fat_macho_slices(data, "<", magic == b"\xbf\xba\xfe\xca")
    raise ContractError(f"unsupported executable magic {magic.hex()}")


def read_fat_macho_slices(
    data: bytes, endian: str, is_64_bit: bool
) -> list[tuple[str, bytes]]:
    count = read_u32(data, 4, endian)
    stride = 32 if is_64_bit else 20
    slices = []
    for index in range(count):
        entry = 8 + index * stride
        architecture = CPU_ARCH.get(read_u32(data, entry, endian), "unknown")
        if is_64_bit:
            offset = struct.unpack_from(f"{endian}Q", data, entry + 8)[0]
            size = struct.unpack_from(f"{endian}Q", data, entry + 16)[0]
        else:
            offset = read_u32(data, entry + 8, endian)
            size = read_u32(data, entry + 12, endian)
        if offset + size > len(data):
            raise ContractError(f"Mach-O slice for {architecture} exceeds file bounds")
        slices.append((architecture, data[offset : offset + size]))
    return slices


def thin_macho_minimum_version(data: bytes) -> tuple[int, int, int]:
    magic = data[:4]
    if magic in (b"\xcf\xfa\xed\xfe", b"\xce\xfa\xed\xfe"):
        endian = "<"
    elif magic in (b"\xfe\xed\xfa\xcf", b"\xfe\xed\xfa\xce"):
        endian = ">"
    else:
        raise ContractError("fat Mach-O contains an invalid thin slice")
    header_size = 32 if magic in (b"\xcf\xfa\xed\xfe", b"\xfe\xed\xfa\xcf") else 28
    command_count = read_u32(data, 16, endian)
    offset = header_size
    for _ in range(command_count):
        command = read_u32(data, offset, endian)
        command_size = read_u32(data, offset + 4, endian)
        if command_size < 8 or offset + command_size > len(data):
            raise ContractError("invalid Mach-O load command bounds")
        if command == LC_BUILD_VERSION:
            return unpack_macho_version(read_u32(data, offset + 12, endian))
        if command == LC_VERSION_MIN_MACOSX:
            return unpack_macho_version(read_u32(data, offset + 8, endian))
        offset += command_size
    raise ContractError("Mach-O binary does not declare a minimum macOS version")


def unpack_macho_version(value: int) -> tuple[int, int, int]:
    return (value >> 16, (value >> 8) & 0xFF, value & 0xFF)


def format_version(version: tuple[int, int, int]) -> str:
    return ".".join(str(component) for component in version)


def read_fat_macho(data: bytes, endian: str, is_64_bit: bool) -> set[str]:
    count = read_u32(data, 4, endian)
    stride = 32 if is_64_bit else 20
    architectures = set()
    for index in range(count):
        cpu_type = read_u32(data, 8 + index * stride, endian)
        architectures.add(CPU_ARCH.get(cpu_type, "unknown"))
    return architectures


def read_archive_members(platform: str, path: Path) -> list[tuple[str, bytes, bool]]:
    names = EXPECTED_MEMBERS[platform]
    if platform == "linux":
        with tarfile.open(path, "r:gz") as archive:
            return [read_tar_member(archive, name) for name in names]
    with zipfile.ZipFile(path) as archive:
        return [read_zip_member(archive, name, platform) for name in names]


def read_tar_member(archive: tarfile.TarFile, name: str) -> tuple[str, bytes, bool]:
    member = archive.getmember(name)
    stream = archive.extractfile(member)
    if stream is None:
        raise ContractError(f"archive member is not a file: {name}")
    return name, stream.read(), bool(member.mode & 0o111)


def read_zip_member(
    archive: zipfile.ZipFile, name: str, platform: str
) -> tuple[str, bytes, bool]:
    info = archive.getinfo(name)
    unix_mode = info.external_attr >> 16
    executable = platform == "windows" or bool(unix_mode & 0o111)
    return name, archive.read(info), executable


def verify(platform: str, path: Path) -> list[str]:
    required = {"arm64", "x86_64"} if platform == "macos" else {"x86_64"}
    messages = []
    for name, data, executable in read_archive_members(platform, path):
        if not executable:
            raise ContractError(f"{name} is not executable")
        architectures = detect_architectures(data)
        if architectures != required:
            raise ContractError(
                f"{name} architectures={sorted(architectures)} required={sorted(required)}"
            )
        detail = ",".join(sorted(architectures))
        if platform == "macos":
            versions = detect_macos_minimum_versions(data)
            mismatches = {
                architecture: version
                for architecture, version in versions.items()
                if version != MINIMUM_MACOS_VERSION
            }
            if mismatches:
                rendered = ", ".join(
                    f"{architecture}={format_version(version)}"
                    for architecture, version in sorted(mismatches.items())
                )
                raise ContractError(
                    f"{name} minimum macOS versions ({rendered}) must equal "
                    f"{format_version(MINIMUM_MACOS_VERSION)}"
                )
            detail += f" minos={format_version(MINIMUM_MACOS_VERSION)}"
        messages.append(f"{name}: {detail}")
    if platform == "macos":
        verify_macos_info_plist(path)
    return messages


def verify_macos_info_plist(path: Path) -> None:
    with zipfile.ZipFile(path) as archive:
        payload = archive.read("KatanA Desktop.app/Contents/Info.plist")
    plist = plistlib.loads(payload)
    actual = plist.get("LSMinimumSystemVersion")
    expected = ".".join(str(component) for component in MINIMUM_MACOS_VERSION[:2])
    if actual != expected:
        raise ContractError(
            f"Info.plist LSMinimumSystemVersion={actual!r} required={expected!r}"
        )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("platform", choices=EXPECTED_MEMBERS)
    parser.add_argument("archive", type=Path)
    args = parser.parse_args()
    try:
        messages = verify(args.platform, args.archive)
    except (ContractError, KeyError, OSError, struct.error) as error:
        print(f"FAIL: binary architecture contract violation: {error}")
        return 1
    for message in messages:
        print(f"OK: {message}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
