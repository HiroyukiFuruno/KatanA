"""Synthetic unit-only binary headers in real archives, never release evidence."""

import io
import json
import os
from pathlib import Path
import plistlib
import subprocess
import tarfile
import zipfile

from document_host_io import legacy
from generated_asset_identity import assets_for, file_digest, make_manifest, module
from published_git_source import git_source_identity


def git(root: Path, *args: str) -> str:
    return subprocess.run(["git", "-C", str(root), "-c", "user.name=Identity Unit Test",
                           "-c", "user.email=identity@example.invalid", *args], check=True,
                          capture_output=True, text=True, env=legacy().git_environment()).stdout.strip()


def repository(root: Path, version: str = "v0.22.45") -> str:
    (root / "Cargo.toml").write_text(f'[workspace]\n[workspace.package]\nversion="{version[1:]}"\n')
    lock = ""
    for name in sorted(legacy().RENDERER_PACKAGES):
        lock += (f'[[package]]\nname="{name}"\nversion="0.1.0"\n'
                 f'source="{legacy().CRATES_IO_SOURCE}"\nchecksum="' + "a" * 64 + '"\n\n')
    (root / "Cargo.lock").write_text(lock)
    git(root, "init", "-q")
    git(root, "add", "Cargo.toml", "Cargo.lock")
    git(root, "commit", "-q", "-m", "Synthetic unit source only")
    return git(root, "rev-parse", "HEAD")


def archives(directory: Path, version: str) -> dict:
    directory.mkdir(parents=True)
    samples = module("_unit_architecture_samples", "test-verify-binary-architecture.py")
    minimum = samples.MODULE.MINIMUM_MACOS_VERSION
    headers = {"macos": samples.fat_macho_with_minimum_version(*minimum[:2]),
               "linux": samples.elf_x86_64(), "windows": samples.pe_x86_64()}
    binaries = {}
    for platform, header in headers.items():
        main, worker = directory.parent / f"{platform}-main", directory.parent / f"{platform}-worker"
        main.write_bytes(header + b"SYNTHETIC-UNIT-MAIN")
        worker.write_bytes(header + b"SYNTHETIC-UNIT-WORKER")
        binaries[platform] = main, worker
        portable = next(name for name in assets_for(version)[platform] if name.endswith((".zip", ".tar.gz")))
        if platform == "linux":
            with tarfile.open(directory / portable, "w:gz") as archive:
                for name, path in (("KatanA", main), ("kdv-office-worker", worker)):
                    data = path.read_bytes()
                    member = tarfile.TarInfo(name)
                    member.size, member.mode = len(data), 0o755
                    archive.addfile(member, io.BytesIO(data))
        else:
            with zipfile.ZipFile(directory / portable, "w") as archive:
                for name, path in (("KatanA", main), ("kdv-office-worker", worker)):
                    prefix = "KatanA Desktop.app/Contents/MacOS/" if platform == "macos" else ""
                    entry = zipfile.ZipInfo(prefix + name + (".exe" if platform == "windows" else ""))
                    entry.external_attr = 0o100755 << 16
                    archive.writestr(entry, path.read_bytes())
                if platform == "macos":
                    archive.writestr("KatanA Desktop.app/Contents/Info.plist",
                                     plistlib.dumps({"LSMinimumSystemVersion": f"{minimum[0]}.{minimum[1]}"}))
        for name in assets_for(version)[platform] - {portable}:
            (directory / name).write_bytes(b"SYNTHETIC-UNIT-INSTALLER-NOT-A-RELEASE")
    return binaries


def bundle(root: Path, version: str = "v0.22.45") -> tuple:
    commit = repository(root, version)
    source = git_source_identity(root, commit)
    assets, manifests = root / "target/assets", root / "target/manifests"
    binaries = archives(assets, version)
    records = {}
    for platform, (main, worker) in binaries.items():
        record = make_manifest(platform, [assets / name for name in assets_for(version)[platform]], source,
                               main, worker, run_id="123", run_attempt="2", version=version)
        write_record(manifests, platform, record)
        records[platform] = record
    checksums(assets)
    return commit, assets, manifests, records, binaries


def write_record(manifests: Path, platform: str, record: dict) -> None:
    path = manifests / f"artifacts-identity-{platform}" / "identity.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(record))


def checksums(assets: Path) -> None:
    paths = sorted(path for path in assets.iterdir() if path.name != "checksums.txt")
    (assets / "checksums.txt").write_text("".join(f"{file_digest(path)}  {path.name}\n" for path in paths))
