#!/usr/bin/env python3
"""Match downloaded publication payloads to the accepted executable bytes."""

import argparse
import hashlib
import importlib.util
import json
import shutil
import stat
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import zipfile
from pathlib import Path


class ArtifactIdentityError(ValueError):
    pass


def digest(stream):
    value = hashlib.sha256()
    total = 0
    while block := stream.read(1024 * 1024):
        total += len(block)
        if total > 2 * 1024**3:
            raise ArtifactIdentityError("executable exceeds 2 GiB safety limit")
        value.update(block)
    if total == 0:
        raise ArtifactIdentityError("empty executable")
    return value.hexdigest()


def regular(path):
    if path.is_symlink() or not path.is_file():
        raise ArtifactIdentityError(f"not a regular artifact: {path}")
    return path


def zip_digest(path, member):
    with zipfile.ZipFile(regular(path)) as archive:
        entries = [entry for entry in archive.infolist() if entry.filename == member]
        if len(entries) != 1:
            raise ArtifactIdentityError(f"missing or duplicate ZIP executable: {member}")
        entry = entries[0]
        if entry.is_dir() or stat.S_ISLNK(entry.external_attr >> 16):
            raise ArtifactIdentityError(f"nonregular ZIP executable: {member}")
        with archive.open(entry) as stream:
            return digest(stream)


def tar_digest(path, member):
    with tarfile.open(regular(path), "r:gz") as archive:
        entries = [entry for entry in archive.getmembers() if entry.name == member]
        if len(entries) != 1 or not entries[0].isfile():
            raise ArtifactIdentityError(f"missing, duplicate or nonregular TAR executable: {member}")
        with archive.extractfile(entries[0]) as stream:
            return digest(stream)


def compare(actual, record, field, label):
    expected = record.get(field)
    if not isinstance(expected, str) or actual != expected.lower():
        raise ArtifactIdentityError(f"published {label} does not match accepted {field}")


def check_portable(assets, targets):
    pairs = {}
    for target in ("macos-arm64", "macos-x86_64"):
        pair = []
        for name, field in (("KatanA", "main_sha256"), ("kdv-office-worker", "sidecar_sha256")):
            actual = zip_digest(assets / "KatanA-macOS.zip", f"KatanA Desktop.app/Contents/MacOS/{name}")
            compare(actual, targets[target], field, f"macOS ZIP {target}/{name}")
            pair.append(actual)
        pairs[target] = pair
    for target, archive, reader, suffix in (
        ("linux-x86_64", "KatanA-linux-x86_64.tar.gz", tar_digest, ""),
        ("windows-x86_64", "KatanA-windows-x86_64.zip", zip_digest, ".exe"),
    ):
        pair = []
        for name, field in (("KatanA", "main_sha256"), ("kdv-office-worker", "sidecar_sha256")):
            actual = reader(assets / archive, name + suffix)
            compare(actual, targets[target], field, f"{archive}/{name}{suffix}")
            pair.append(actual)
        pairs[target] = pair
    return pairs


def check_dmg(path, target):
    with tempfile.TemporaryDirectory(prefix="katana-publish-dmg-") as directory:
        mount = Path(directory) / "mount"
        mounted = False
        try:
            subprocess.run(["hdiutil", "attach", "-readonly", "-nobrowse", "-mountpoint", str(mount), str(regular(path))], check=True, stdout=subprocess.PIPE, timeout=120)
            mounted = True
            for name, field in (("KatanA", "main_sha256"), ("kdv-office-worker", "sidecar_sha256")):
                binary = mount / "KatanA Desktop.app" / "Contents" / "MacOS" / name
                if any(part.is_symlink() for part in (binary, *binary.parents) if part.is_relative_to(mount)):
                    raise ArtifactIdentityError(f"symlink in DMG executable path: {name}")
                with regular(binary).open("rb") as stream:
                    compare(digest(stream), target, field, f"DMG/{name}")
        finally:
            if mounted:
                subprocess.run(["hdiutil", "detach", str(mount)], check=True, stdout=subprocess.PIPE, timeout=120)


def sevenzip_member(tool, archive, member, output):
    with output.open("wb") as destination:
        subprocess.run([tool, "e", "-so", str(regular(archive)), member], stdout=destination, stderr=subprocess.PIPE, check=True, timeout=120)
    regular(output)
    if output.stat().st_size == 0 or output.stat().st_size > 2 * 1024**3:
        raise ArtifactIdentityError(f"missing or oversized installer member: {member}")


def check_msi(path, target):
    tool = shutil.which("7zz")
    if tool is None:
        raise ArtifactIdentityError("7zz is required to verify the actual MSI cabinet payload")
    listing = subprocess.run([tool, "l", "-slt", str(regular(path))], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=120).stdout
    if "\nType = Compound\n" not in listing or "\nExtension = msi\n" not in listing or "\nType = Cab\n" not in listing:
        raise ArtifactIdentityError("MSI must contain an embedded cabinet in a Compound installer")
    with tempfile.TemporaryDirectory(prefix="katana-publish-msi-") as directory:
        root = Path(directory)
        # 7zzはMSI内cabinetへ自動で入る。WiXのFile IDを実payloadから取り出す。
        for member, field in (("exe0", "main_sha256"), ("OfficeWorkerExe", "sidecar_sha256")):
            if listing.split("----------\n", 1)[-1].count(f"Path = {member}\n") != 1:
                raise ArtifactIdentityError(f"missing or duplicate MSI executable: {member}")
            extracted = root / member
            sevenzip_member(tool, path, member, extracted)
            with extracted.open("rb") as stream:
                compare(digest(stream), target, field, f"MSI/{member}")


def verify(root, assets, evidence_path):
    spec = importlib.util.spec_from_file_location("acceptance", Path(__file__).with_name("check-document-fidelity-acceptance-evidence.py"))
    acceptance = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(acceptance)
    acceptance.verify(root, evidence_path)
    evidence = json.loads(evidence_path.read_text())
    targets = evidence["packaged_targets"]
    check_portable(assets, targets)
    dmgs = list(assets.glob("KatanA-Desktop-*.dmg"))
    if len(dmgs) != 1:
        raise ArtifactIdentityError("exactly one macOS DMG is required")
    check_dmg(dmgs[0], targets["macos-arm64"])
    check_msi(assets / "KatanA-windows-x86_64.msi", targets["windows-x86_64"])


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--assets", type=Path, required=True)
    parser.add_argument("--evidence", type=Path)
    parser.add_argument("--version", required=True)
    args = parser.parse_args()
    try:
        if args.version.removeprefix("v") != "0.22.42":
            raise ArtifactIdentityError("unsupported publication acceptance version")
        root = args.root.resolve()
        with (root / "Cargo.toml").open("rb") as manifest:
            if tomllib.load(manifest).get("workspace", {}).get("package", {}).get("version") != "0.22.42":
                raise ArtifactIdentityError("publication version does not match Cargo.toml")
        gate_spec = importlib.util.spec_from_file_location("release_gate", Path(__file__).with_name("check-document-fidelity-release-gate.py"))
        gate = importlib.util.module_from_spec(gate_spec)
        gate_spec.loader.exec_module(gate)
        evidence = args.evidence or gate.tasks_evidence_path(root)
        verify(root, args.assets.resolve(), evidence.resolve())
    except (ValueError, RuntimeError, OSError, subprocess.SubprocessError, tarfile.TarError, zipfile.BadZipFile) as error:
        print(f"ERROR: publication executable identity: {error}", file=sys.stderr)
        return 1
    print("OK: all five publication payloads match the accepted main and sidecar bytes")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
