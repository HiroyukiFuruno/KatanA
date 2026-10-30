"""Version-bound generated payload identity, independent of document acceptance."""

from __future__ import annotations

import hashlib
import importlib.util
import re
from pathlib import Path

from document_host_io import fields, integer, read, require, sha
from postpublication_source_binding import validate_version

TARGETS = {"macos": {"macos-arm64", "macos-x86_64"}, "linux": {"linux-x86_64"},
           "windows": {"windows-x86_64"}}


def assets_for(version: str) -> dict[str, set[str]]:
    bare = validate_version(version).removeprefix("v")
    return {"macos": {"KatanA-macOS.zip", f"KatanA-Desktop-{bare}.dmg"},
            "linux": {"KatanA-linux-x86_64.tar.gz"},
            "windows": {"KatanA-windows-x86_64.zip", "KatanA-windows-x86_64.msi"}}


def module(name: str, filename: str):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    require(spec is not None and spec.loader is not None, "publication helper unavailable")
    loaded = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(loaded)
    return loaded


def file_digest(path: Path) -> str:
    require(path.is_file() and not path.is_symlink(), "regular publication asset required")
    require(0 < path.stat().st_size <= 4 * 1024**3, "empty or oversized publication asset")
    with path.open("rb") as handle:
        return hashlib.file_digest(handle, "sha256").hexdigest()


def names(platform: str, paths: list[Path], version: str) -> dict[str, Path]:
    expected = assets_for(version)
    require(platform in expected and len(paths) == len(expected[platform]), "platform artifact count differs")
    result = {path.name: path for path in paths}
    require(len(result) == len(paths) and set(result) == expected[platform], "missing/extra/duplicate publication asset")
    return result


def run_identity(run_id: str, run_attempt: str) -> tuple[str, str]:
    require(all(isinstance(value, str) and re.fullmatch(r"[1-9][0-9]*", value) is not None
                for value in (run_id, run_attempt)), "actual GitHub Actions run identity required")
    return run_id, run_attempt


def make_manifest(platform: str, paths: list[Path], source: dict, main: Path, worker: Path,
                  *, run_id: str, run_attempt: str, version: str) -> dict:
    run_identity(run_id, run_attempt)
    assets = names(platform, paths, version)
    arch = module("_publication_architecture", "verify-binary-architecture.py")
    expected_archs = {"arm64", "x86_64"} if platform == "macos" else {"x86_64"}
    main_sha, worker_sha = file_digest(main), file_digest(worker)
    for path in (main, worker):
        require(arch.detect_architectures(read(path, maximum=2*1024**3)) == expected_archs,
                "build main/worker architecture differs from target")
    targets = {target: {"main_sha256": main_sha, "sidecar_sha256": worker_sha}
               for target in TARGETS[platform]}
    identity = module("_publication_archive_identity", "check-published-artifact-identity.py")
    portable = "KatanA-macOS.zip" if platform == "macos" else next(name for name in assets if name.endswith((".zip", ".tar.gz")))
    archive_path = assets[portable]
    arch.verify(platform, archive_path)
    prefix, suffix = ("KatanA Desktop.app/Contents/MacOS/", "") if platform == "macos" else ("", ".exe" if platform == "windows" else "")
    reader = identity.tar_digest if platform == "linux" else identity.zip_digest
    for binary, expected in (("KatanA", main_sha), ("kdv-office-worker", worker_sha)):
        require(reader(archive_path, prefix+binary+suffix) == expected, "build portable payload differs from actual main/worker")
    return {"schema": "katana-generated-asset-identity-v1", "version": version, "platform": platform,
            "provenance": source, "github_run_id": run_id, "github_run_attempt": run_attempt,
            "targets": targets,
            "assets": {name: {"sha256": file_digest(path), "size": path.stat().st_size}
                       for name, path in assets.items()}}


def manifest_metadata(platform: str, value: object, version: str) -> tuple[dict, tuple[str, str]]:
    expected = assets_for(version)
    require(platform in expected, "unknown generated platform")
    record = fields(value, {"schema", "version", "platform", "provenance", "github_run_id",
                            "github_run_attempt", "targets", "assets"}, "generated manifest")
    require(record["schema"] == "katana-generated-asset-identity-v1" and record["version"] == version
            and record["platform"] == platform, "generated asset source/version/platform differs")
    identity = run_identity(record["github_run_id"], record["github_run_attempt"])
    require(isinstance(record["targets"], dict) and set(record["targets"]) == TARGETS[platform], "manifest target set differs")
    for pair in record["targets"].values():
        fields(pair, {"main_sha256", "sidecar_sha256"}, "generated target")
        sha(pair["main_sha256"], "generated main")
        sha(pair["sidecar_sha256"], "generated worker")
    payloads = fields(record["assets"], expected[platform], "generated assets")
    for descriptor in payloads.values():
        fields(descriptor, {"sha256", "size"}, "generated asset")
        sha(descriptor["sha256"], "generated asset hash")
        integer(descriptor["size"], "generated asset size", minimum=1)
    return record["targets"], identity


def verify_build_record(platform: str, value: object, assets: Path, version: str) -> tuple[dict, tuple[str, str]]:
    targets, identity = manifest_metadata(platform, value, version)
    for name, descriptor in value["assets"].items():
        path = assets / name
        require(file_digest(path) == sha(descriptor["sha256"], "generated asset hash")
                and path.stat().st_size == integer(descriptor["size"], "generated asset size", minimum=1),
                "downloaded asset differs from build-job manifest")
    return targets, identity


def verify_payloads(assets: Path, targets: dict, version: str) -> None:
    identity = module("_downloaded_archive_identity", "check-published-artifact-identity.py")
    identity.check_portable(assets, targets)
    identity.check_dmg(assets / f"KatanA-Desktop-{validate_version(version).removeprefix('v')}.dmg", targets["macos-arm64"])
    identity.check_msi(assets / "KatanA-windows-x86_64.msi", targets["windows-x86_64"])
    architecture = module("_downloaded_archive_architecture", "verify-binary-architecture.py")
    for platform, filename in (("macos", "KatanA-macOS.zip"), ("linux", "KatanA-linux-x86_64.tar.gz"),
                               ("windows", "KatanA-windows-x86_64.zip")):
        architecture.verify(platform, assets / filename)
