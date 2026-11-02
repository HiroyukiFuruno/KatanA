"""独立buildのmanifestと実download payloadを照合し、候補QAとのbytes同一性を仮定しない。"""

from __future__ import annotations

import subprocess
from pathlib import Path

from document_host_io import decode, digest, fields, integer, read, reference, require, sha
import generated_asset_identity as generated
from generated_asset_identity import TARGETS, file_digest, module
from v047_evidence_paths import HOST_MANIFESTS, HOST_RECEIPT

ASSETS = generated.assets_for("v0.22.42")


def allowed_publication_version(version: str) -> str:
    require(version in {"v0.22.42", "v0.22.47"}, "unsupported host publication version")
    return version


def names(platform: str, paths: list[Path]) -> dict[str, Path]:
    return generated.names(platform, paths, "v0.22.42")


def make_manifest(platform: str, paths: list[Path], source: dict, main: Path, worker: Path,
                  *, run_id: str, run_attempt: str) -> dict:
    return generated.make_manifest(platform, paths, source, main, worker,
                                   run_id=run_id, run_attempt=run_attempt, version="v0.22.42")


def manifest_metadata(platform: str, value: object, source: dict, version: str = "v0.22.42") -> tuple[dict, tuple[str, str]]:
    from document_host_acceptance import same_source_content
    result = generated.manifest_metadata(platform, value, allowed_publication_version(version))
    require(same_source_content(value["provenance"], source), "generated asset source/version/platform differs")
    return result


def verify_build_record(platform: str, value: object, source: dict, assets: Path,
                        version: str = "v0.22.42") -> tuple[dict, tuple[str, str]]:
    manifest_metadata(platform, value, source, version)
    return generated.verify_build_record(platform, value, assets, version)


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
                     run_attempt: str | None = None, published_commit: str | None = None,
                     version: str = "v0.22.42") -> dict:
    version = allowed_publication_version(version)
    verify_publication_context(root, source, published_commit, run_id, run_attempt)
    manifests, targets, run_identity = {}, {}, None
    require(not manifest_directory.is_symlink(), "manifest directory cannot be symlinked")
    for platform in generated.assets_for(version):
        path = manifest_directory / f"artifacts-identity-{platform}" / "identity.json"
        require(not path.parent.is_symlink(), "manifest directory cannot be symlinked")
        record = decode(read(path))
        platform_targets, identity = verify_build_record(platform, record, source, assets, version)
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
    generated.verify_payloads(assets, targets, version)
    return {"manifests": manifests, "targets": targets}


def verify_download_receipt(root: Path, value: object, source: dict, bundle: dict,
                            version: str = "v0.22.42") -> None:
    version = allowed_publication_version(version)
    record = fields(value, {"status", "receipt"}, "postpublication")
    require(record["status"] == "published_macos_host_accepted", "postpublication remains pending")
    _, receipt = reference(root, record["receipt"], "actual Release download receipt",
                           allowed_paths=frozenset({HOST_RECEIPT}) if version == "v0.22.47" else None)
    fields(receipt, {"schema", "release_url", "release_id", "tag", "draft", "published_at", "source_commit",
                     "provenance", "assets", "downloaded_macos", "main_sha256", "sidecar_sha256", "build_manifests",
                     "producer_sha256"}, "Release receipt")
    from document_host_acceptance import same_source_content
    require(receipt["schema"] == "katana-release-download-v1" and receipt["tag"] == version
            and receipt["draft"] is False and receipt["source_commit"] == receipt["provenance"]["source_commit"]
            and same_source_content(receipt["provenance"], source), "Release identity/source differs from final QA")
    if version == "v0.22.47":
        from published_git_source import git_workspace_version
        require(receipt["source_commit"] == source["source_commit"], "actual release source commit differs from host run")
        require(git_workspace_version(root, receipt["source_commit"]) == version,
                "actual release version differs from committed Cargo.toml")
    from document_host_io import moment
    moment(receipt["published_at"], "Release publication time")
    integer(receipt["release_id"], "Release id", minimum=1)
    require(receipt["release_url"] == f"https://github.com/HiroyukiFuruno/KatanA/releases/tag/{version}",
            "Release URL differs from approved repository/tag")
    assets = generated.assets_for(version)
    expected = set().union(*assets.values())
    fields(receipt["assets"], expected, "Release assets")
    for name, item in receipt["assets"].items():
        fields(item, {"asset_id", "size", "sha256", "downloaded_at", "download_url"}, name)
        integer(item["asset_id"], "Release asset id", minimum=1)
        integer(item["size"], "Release asset size", minimum=1)
        sha(item["sha256"], "actual downloaded asset hash")
        require(moment(item["downloaded_at"], "download time") >= moment(receipt["published_at"], "published"),
                "candidate bytes were not downloaded from the published Release")
        require(item["download_url"] == f"https://github.com/HiroyukiFuruno/KatanA/releases/download/{version}/{name}",
                "download URL differs from actual Release asset")
    producer_name = "record-published-artifact-download-v47.py" if version == "v0.22.47" else "record-published-artifact-download.py"
    require(receipt["producer_sha256"] == digest(read(root / "scripts/release" / producer_name)),
            "actual download producer changed")
    fields(receipt["build_manifests"], set(assets), "public build manifests")
    build_identity = None
    for platform, ref in receipt["build_manifests"].items():
        _, manifest = reference(root, ref, "actual build identity manifest",
                                allowed_paths=frozenset({HOST_MANIFESTS[platform]}) if version == "v0.22.47" else None)
        targets, identity = manifest_metadata(platform, manifest, source, version)
        require(build_identity is None or identity == build_identity, "published build manifests came from different workflow runs")
        build_identity = identity
        require(manifest["provenance"]["source_commit"] == receipt["source_commit"], "published build source differs from actual Release tag")
        for name, descriptor in manifest["assets"].items():
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
