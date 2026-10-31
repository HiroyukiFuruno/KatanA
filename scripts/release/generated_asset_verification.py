"""Strict source/run/version verification for generated publication bytes."""

from pathlib import Path

from document_host_io import decode, read, require
from generated_asset_identity import assets_for, file_digest, run_identity, verify_build_record, verify_payloads
from published_git_source import git_source_identity, git_workspace_version


def verify_checksums(assets: Path, version: str) -> None:
    expected = set().union(*assets_for(version).values())
    require(assets.is_dir() and not assets.is_symlink(), "regular asset directory required")
    require({path.name for path in assets.iterdir()} == expected | {"checksums.txt"},
            "missing/extra publication asset")
    checksums = {}
    for line in read(assets / "checksums.txt").decode("utf-8").splitlines():
        value, separator, name = line.partition("  ")
        require(separator == "  " and name in expected and name not in checksums,
                "checksum file contains missing/duplicate/unknown entry")
        checksums[name] = value
    require(set(checksums) == expected, "checksum set differs from five release assets")
    for name in expected:
        require(checksums[name] == file_digest(assets / name), "publication checksum differs from actual bytes")


def verify_records(root: Path, assets: Path, manifests: Path, version: str,
                   source_commit: str, run_id: str, run_attempt: str) -> dict:
    expected_run = run_identity(run_id, run_attempt)
    require(git_workspace_version(root, source_commit) == version,
            "publication version differs from committed Cargo.toml")
    source = git_source_identity(root, source_commit)
    verify_checksums(assets, version)
    require(manifests.is_dir() and not manifests.is_symlink(), "regular manifest directory required")
    records, targets = {}, {}
    for platform in assets_for(version):
        directory = manifests / f"artifacts-identity-{platform}"
        require(not directory.is_symlink(), "manifest directory cannot be symlinked")
        record = decode(read(directory / "identity.json"))
        platform_targets, identity = verify_build_record(platform, record, assets, version)
        require(identity == expected_run, "manifest reused from a different workflow run/attempt")
        require(record["provenance"] == source, "manifest provenance differs from exact committed source")
        records[platform] = record
        targets.update(platform_targets)
    return {"manifests": records, "targets": targets, "provenance": source}


def verify(root: Path, assets: Path, manifests: Path, version: str,
           source_commit: str, run_id: str, run_attempt: str) -> dict:
    result = verify_records(root, assets, manifests, version, source_commit, run_id, run_attempt)
    verify_payloads(assets, result["targets"], version)
    return result
