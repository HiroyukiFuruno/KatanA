"""Collect strict publication identity from an admitted finished release run."""

from pathlib import Path
import subprocess

from document_host_io import decode, read, require, write_new
from future_publication_download import api, download, verify_unchanged
from generated_asset_identity import file_digest, manifest_metadata, run_identity
from generated_asset_verification import verify
from postpublication_run_admission import admit
from postpublication_source_binding import COMMIT_PATTERN, REPOSITORY, validate_version
from postpublication_controller_scope import git, validate_worktree_clean
from published_git_source import SOURCE_ROOTS


def controller_identity(root: Path) -> str:
    validate_worktree_clean(git(root, "diff", "--name-only", "-z", "HEAD", "--", *SOURCE_ROOTS),
                            git(root, "ls-files", "--others", "--exclude-standard", "-z", "--", *SOURCE_ROOTS))
    return git(root, "rev-parse", "HEAD").decode().strip()


def publication_candidate(candidate: object, run_id: str, run_attempt: str,
                          version: str | None = None) -> tuple[str, str]:
    require(isinstance(candidate, dict) and isinstance(candidate.get("provenance"), dict),
            "generated source metadata missing")
    actual_version = validate_version(candidate.get("version"))
    require(version is None or version == actual_version, "requested publication version differs")
    _, identity = manifest_metadata("macos", candidate, actual_version)
    require(identity == run_identity(run_id, run_attempt), "candidate differs from triggering run/attempt")
    source = candidate["provenance"].get("source_commit")
    require(isinstance(source, str) and COMMIT_PATTERN.fullmatch(source) is not None,
            "actual product source commit required")
    return actual_version, source


def collect(root: Path, work: Path, run_id: str, run_attempt: str, version: str | None = None) -> dict:
    run_identity(run_id, run_attempt)
    require(not work.exists() and not work.is_symlink(), "publication work directory must be new")
    controller = controller_identity(root)
    work.mkdir(parents=True)
    manifests = work / "manifests"
    command = ["gh", "run", "download", run_id, "--repo", REPOSITORY, "--dir", str(manifests)]
    for platform in ("macos", "linux", "windows"):
        command.extend(("--name", f"artifacts-identity-{platform}"))
    subprocess.run(command, check=True, timeout=1800)
    for platform in ("macos", "linux", "windows"):
        directory = manifests / f"artifacts-identity-{platform}"
        require(not directory.is_symlink() and not (directory / "identity.json").exists()
                and not (directory / "identity.json").is_symlink(),
                "manifest normalization collision or symlink")
        raw = directory / f"identity-{platform}.json"
        read(raw)
        raw.rename(directory / "identity.json")
    candidate = decode(read(manifests / "artifacts-identity-macos/identity.json"))
    actual_version, source = publication_candidate(candidate, run_id, run_attempt, version)
    try:
        selected, attempt, admitted_source = admit(source, actual_version, work / "run-admission.json", root)
    except ValueError as error:
        raise ValueError(f"publication identity ineligible: {error}") from error
    require((str(selected), str(attempt), admitted_source) == (run_id, run_attempt, source),
            "downloaded manifests differ from the admitted run/attempt/source")
    release = api(f"repos/{REPOSITORY}/releases/tags/{actual_version}")
    require(isinstance(release, dict), "Release metadata missing")
    downloaded = download(work / "assets", actual_version, release)
    result = verify(root, work / "assets", manifests, actual_version, source, run_id, run_attempt)
    verify_unchanged(release, api(f"repos/{REPOSITORY}/releases/tags/{actual_version}"), actual_version)
    require(api(f"repos/{REPOSITORY}/commits/{actual_version}").get("sha") == source,
            "Release tag changed while downloading")
    receipt = {"schema": "katana-published-asset-identity-v1", "version": actual_version,
               "scope": "asset-identity-only", "document_qa": "not_evaluated",
               "github_run_id": run_id, "github_run_attempt": run_attempt,
               "provenance": result["provenance"], "release_id": release["id"], "assets": downloaded,
               "controller_commit": controller,
               "build_manifests": {platform: {"path": f"manifests/artifacts-identity-{platform}/identity.json",
                                               "sha256": file_digest(manifests / f"artifacts-identity-{platform}/identity.json")}
                                   for platform in result["manifests"]}}
    write_new(work / "publication-identity.json", receipt)
    return receipt
