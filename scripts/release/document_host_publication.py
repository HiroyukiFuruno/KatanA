"""独立buildのmanifestと実download payloadを照合し、候補QAとのbytes同一性を仮定しない。"""

from __future__ import annotations

import hashlib
import importlib.util
import re
import subprocess
from pathlib import Path

from document_host_io import decode, digest, fields, integer, read, reference, require, sha

ASSETS = {
    "macos": {"KatanA-macOS.zip", "KatanA-Desktop-0.22.42.dmg"},
    "linux": {"KatanA-linux-x86_64.tar.gz"},
    "windows": {"KatanA-windows-x86_64.zip", "KatanA-windows-x86_64.msi"},
}
TARGETS = {"macos": {"macos-arm64", "macos-x86_64"}, "linux": {"linux-x86_64"},
           "windows": {"windows-x86_64"}}


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


def names(platform: str, paths: list[Path]) -> dict[str, Path]:
    require(platform in ASSETS and len(paths) == len(ASSETS[platform]), "platform artifact count differs")
    result = {path.name: path for path in paths}
    require(len(result) == len(paths) and set(result) == ASSETS[platform], "missing/extra/duplicate publication asset")
    return result


def make_manifest(platform: str, paths: list[Path], source: dict, main: Path, worker: Path,
                  *, run_id: str, run_attempt: str) -> dict:
    require(re.fullmatch(r"[1-9][0-9]*", run_id) is not None and re.fullmatch(r"[1-9][0-9]*", run_attempt) is not None,
            "actual GitHub Actions run identity required")
    assets = names(platform, paths)
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
    return {"schema": "katana-generated-asset-identity-v1", "version": "v0.22.42", "platform": platform,
            "provenance": source, "github_run_id": run_id, "github_run_attempt": run_attempt,
            "targets": targets,
            "assets": {name: {"sha256": file_digest(path), "size": path.stat().st_size}
                       for name, path in assets.items()}}


def manifest_metadata(platform: str, value: object, source: dict) -> tuple[dict, tuple[str, str]]:
    from document_host_acceptance import same_source_content
    record = fields(value, {"schema", "version", "platform", "provenance", "github_run_id",
                            "github_run_attempt", "targets", "assets"}, "generated manifest")
    require(record["schema"] == "katana-generated-asset-identity-v1" and record["version"] == "v0.22.42"
            and record["platform"] == platform and same_source_content(record["provenance"], source),
            "generated asset source/version/platform differs")
    identity = (record["github_run_id"], record["github_run_attempt"])
    require(all(isinstance(value, str) and re.fullmatch(r"[1-9][0-9]*", value) is not None for value in identity),
            "generated manifest needs actual build run identity")
    require(isinstance(record["targets"], dict) and set(record["targets"]) == TARGETS[platform], "manifest target set differs")
    for pair in record["targets"].values():
        fields(pair, {"main_sha256", "sidecar_sha256"}, "generated target")
        sha(pair["main_sha256"], "generated main")
        sha(pair["sidecar_sha256"], "generated worker")
    payloads = fields(record["assets"], ASSETS[platform], "generated assets")
    for name, descriptor in payloads.items():
        fields(descriptor, {"sha256", "size"}, "generated asset")
        sha(descriptor["sha256"], "generated asset hash")
        integer(descriptor["size"], "generated asset size", minimum=1)
    return record["targets"], identity


def verify_build_record(platform: str, value: object, source: dict, assets: Path) -> tuple[dict, tuple[str, str]]:
    targets, identity = manifest_metadata(platform, value, source)
    for name, descriptor in value["assets"].items():
        path = assets / name
        require(file_digest(path) == sha(descriptor["sha256"], "generated asset hash")
                and path.stat().st_size == integer(descriptor["size"], "generated asset size", minimum=1),
                "downloaded asset differs from build-job manifest")
    return targets, identity


def verify_generated_provenance(root: Path, record: dict, published_commit: str | None) -> None:
    from document_host_acceptance import provenance
    if published_commit is None:
        provenance(root, record)
    else:
        from published_git_source import verify_git_provenance
        verify_git_provenance(root, record, published_commit)


def verify_publication_context(root: Path, source: dict, published_commit: str | None,
                               run_id: str | None, run_attempt: str | None) -> None:
    if published_commit is None:
        return
    from postpublication_controller_scope import verify_controller_scope
    require(run_id is None and run_attempt is None, "frozen source is only valid after publication")
    require(source.get("source_commit") == published_commit, "published source commit differs")
    verify_controller_scope(root, published_commit)


def verify_generated(root: Path, assets: Path, manifest_directory: Path, source: dict, *, run_id: str | None = None,
                     run_attempt: str | None = None, published_commit: str | None = None) -> dict:
    verify_publication_context(root, source, published_commit, run_id, run_attempt)
    manifests, targets, run_identity = {}, {}, None
    require(not manifest_directory.is_symlink(), "manifest directory cannot be symlinked")
    for platform in ASSETS:
        path = manifest_directory / f"artifacts-identity-{platform}" / "identity.json"
        require(not path.parent.is_symlink(), "manifest directory cannot be symlinked")
        record = decode(read(path))
        platform_targets, identity = verify_build_record(platform, record, source, assets)
        verify_generated_provenance(root, record["provenance"], published_commit)
        actual_build_commit = subprocess.run(["git", "-C", str(root), "rev-parse", "HEAD"], check=True,
                                             capture_output=True, text=True).stdout.strip()
        require(run_id is None or record["provenance"]["source_commit"] == actual_build_commit,
                "generated build commit differs from checked-out publication commit")
        require(run_identity is None or run_identity == identity, "build jobs came from different workflow runs")
        run_identity = identity
        require(run_id is None or identity == (run_id, run_attempt), "manifest reused from a different workflow run/attempt")
        targets.update(platform_targets)
        manifests[platform] = record
    identity_module = module("_downloaded_archive_identity", "check-published-artifact-identity.py")
    identity_module.check_portable(assets, targets)
    identity_module.check_dmg(assets / "KatanA-Desktop-0.22.42.dmg", targets["macos-arm64"])
    identity_module.check_msi(assets / "KatanA-windows-x86_64.msi", targets["windows-x86_64"])
    architecture = module("_downloaded_archive_architecture", "verify-binary-architecture.py")
    for platform, filename in (("macos", "KatanA-macOS.zip"), ("linux", "KatanA-linux-x86_64.tar.gz"),
                               ("windows", "KatanA-windows-x86_64.zip")):
        architecture.verify(platform, assets / filename)
    return {"manifests": manifests, "targets": targets}


def verify_download_receipt(root: Path, value: object, source: dict, bundle: dict) -> None:
    record = fields(value, {"status", "receipt"}, "postpublication")
    require(record["status"] == "published_macos_host_accepted", "postpublication remains pending")
    _, receipt = reference(root, record["receipt"], "actual Release download receipt")
    fields(receipt, {"schema", "release_url", "release_id", "tag", "draft", "published_at", "source_commit",
                     "provenance", "assets", "downloaded_macos", "main_sha256", "sidecar_sha256", "build_manifests",
                     "producer_sha256"}, "Release receipt")
    from document_host_acceptance import same_source_content
    require(receipt["schema"] == "katana-release-download-v1" and receipt["tag"] == "v0.22.42"
            and receipt["draft"] is False and receipt["source_commit"] == receipt["provenance"]["source_commit"]
            and same_source_content(receipt["provenance"], source), "Release identity/source differs from final QA")
    from document_host_io import moment
    moment(receipt["published_at"], "Release publication time")
    integer(receipt["release_id"], "Release id", minimum=1)
    require(receipt["release_url"] == "https://github.com/HiroyukiFuruno/KatanA/releases/tag/v0.22.42",
            "Release URL differs from approved repository/tag")
    expected = set().union(*ASSETS.values())
    fields(receipt["assets"], expected, "Release assets")
    for name, item in receipt["assets"].items():
        fields(item, {"asset_id", "size", "sha256", "downloaded_at", "download_url"}, name)
        integer(item["asset_id"], "Release asset id", minimum=1)
        integer(item["size"], "Release asset size", minimum=1)
        sha(item["sha256"], "actual downloaded asset hash")
        require(moment(item["downloaded_at"], "download time") >= moment(receipt["published_at"], "published"),
                "candidate bytes were not downloaded from the published Release")
        require(item["download_url"] == f"https://github.com/HiroyukiFuruno/KatanA/releases/download/v0.22.42/{name}",
                "download URL differs from actual Release asset")
    require(receipt["producer_sha256"] == digest(read(root / "scripts/release/record-published-artifact-download.py")),
            "actual download producer changed")
    fields(receipt["build_manifests"], set(ASSETS), "public build manifests")
    build_identity = None
    for platform, ref in receipt["build_manifests"].items():
        _, generated = reference(root, ref, "actual build identity manifest")
        targets, identity = manifest_metadata(platform, generated, source)
        require(build_identity is None or identity == build_identity, "published build manifests came from different workflow runs")
        build_identity = identity
        require(generated["provenance"]["source_commit"] == receipt["source_commit"], "published build source differs from actual Release tag")
        for name, descriptor in generated["assets"].items():
            require(all(descriptor[key] == receipt["assets"][name][key] for key in ("sha256", "size")),
                    "actual Release download differs from build-job manifest")
        if platform == "macos":
            for pair in targets.values():
                require(pair["main_sha256"] == receipt["main_sha256"] and pair["sidecar_sha256"] == receipt["sidecar_sha256"],
                        "published Mac target/main/worker differs from actual QA")
    require(receipt["downloaded_macos"] == "KatanA-macOS.zip"
            and receipt["main_sha256"] == bundle["main_sha256"]
            and receipt["sidecar_sha256"] == bundle["sidecar_sha256"]
            and receipt["assets"]["KatanA-macOS.zip"]["sha256"] == bundle["archive_sha256"],
            "final host run is not bound to downloaded Mac bytes")
