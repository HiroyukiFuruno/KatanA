#!/usr/bin/env python3
"""Match downloaded publication payloads to the accepted executable bytes."""

import argparse
import hashlib
import importlib.util
import json
import os
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


def verify(root, assets, evidence_path, *, scope="strict", build_manifests=None, phase="prepublish"):
    if scope == "automated-release":
        from document_host_io import require
        from document_host_publication import module, verify_generated
        require(phase in {"prepublish", "postpublication"}, "unknown automated publication phase")
        require(evidence_path is None, "automated publication does not consume manual QA evidence")
        require(build_manifests is not None, "automated publication requires all three build-job identity manifests")
        gate = module("_automated_release_gate", "check-document-fidelity-release-gate.py")
        gate.verify_automated_approval(root)
        if phase == "postpublication":
            # 公開後docsのHEAD/runはbuildと異なるため、実tagから全5assetを新規downloadして照合する。
            recorder = module("_automated_release_download", "record-published-artifact-download.py")
            release, manifests = recorder.record(root, assets, build_manifests)
            require(all(item["provenance"]["source_commit"] == release["source_commit"] for item in manifests.values()),
                    "published build source differs from actual Release tag")
        else:
            builder = module("_automated_publication_source", "record-generated-artifact-identity.py")
            source = builder.source_identity(root)
            verify_generated(root, assets, build_manifests, source,
                             run_id=os.environ.get("GITHUB_RUN_ID"), run_attempt=os.environ.get("GITHUB_RUN_ATTEMPT"))
        return
    if scope == "katana-host":
        from document_host_acceptance import verify as verify_host
        from document_host_io import require
        from document_host_publication import verify_generated
        require(build_manifests is not None, "host publication requires all three build-job identity manifests")
        evidence = verify_host(root, evidence_path, phase=phase)
        verify_generated(root, assets, build_manifests, evidence["provenance"],
                         run_id=os.environ.get("GITHUB_RUN_ID"), run_attempt=os.environ.get("GITHUB_RUN_ATTEMPT"))
        return
    if scope != "strict":
        raise ArtifactIdentityError("unknown publication scope")
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
    parser.add_argument("--scope", choices=("strict", "katana-host", "automated-release"), default="strict")
    parser.add_argument("--phase", choices=("prepublish", "postpublication"), default="prepublish")
    parser.add_argument("--build-manifests", type=Path)
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
        evidence = args.evidence or (None if args.scope == "automated-release" else gate.tasks_evidence_path(root, host_scope=args.scope == "katana-host", phase=args.phase))
        verify(root, args.assets.resolve(), evidence.resolve() if evidence is not None else None, scope=args.scope,
               build_manifests=args.build_manifests.resolve() if args.build_manifests else None, phase=args.phase)
    except (ValueError, RuntimeError, OSError, subprocess.SubprocessError, tarfile.TarError, zipfile.BadZipFile) as error:
        print(f"ERROR: publication executable identity: {error}", file=sys.stderr)
        return 1
    if args.scope == "automated-release":
        print("OK: all five publication payloads match current-source build manifests; manual document QA remains not verified")
        return 0
    print("OK: all five publication payloads match " + ("their independently built source/lock/target/payload manifests; final host QA remains pending" if args.scope == "katana-host" and args.phase == "prepublish" else "the accepted main and sidecar bytes"))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
