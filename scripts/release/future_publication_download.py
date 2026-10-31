"""Download actual publication bytes without importing historical QA approval."""

from datetime import datetime, timezone
from pathlib import Path
import subprocess

from document_host_io import integer, moment, require
from generated_asset_identity import assets_for, file_digest, module
from generated_asset_verification import verify_checksums
from github_publication import check_release
from postpublication_source_binding import REPOSITORY

# WHY: v42のreceiptはproducer自身のhashを固定するため、旧producerを変更せずAPI readerだけ再利用する。
api = module("_frozen_publication_api", "record-published-artifact-download.py").api


def selected_assets(release: dict, version: str) -> dict:
    require(isinstance(release, dict), "Release metadata missing")
    check_release(release, REPOSITORY, version, version.removeprefix("v"),
                  f"https://github.com/{REPOSITORY}/releases/tag/{version}")
    integer(release.get("id"), "Release id", minimum=1)
    moment(release["published_at"], "Release publication time")
    expected = set().union(*assets_for(version).values()) | {"checksums.txt"}
    require(isinstance(release.get("assets"), list), "Release asset metadata missing")
    selected = {}
    for item in release["assets"]:
        require(isinstance(item, dict), "invalid Release asset metadata")
        name = item.get("name")
        if name in expected:
            require(name not in selected, "duplicate actual Release asset")
            integer(item.get("id"), "Release asset id", minimum=1)
            integer(item.get("size"), "Release asset size", minimum=1)
            selected[name] = item
    require(set(selected) == expected, "Release is missing one of the five assets/checksums")
    return selected


def download(directory: Path, version: str, release: dict) -> dict:
    require(not directory.exists() and not directory.is_symlink(), "refusing to reuse a download directory")
    selected = selected_assets(release, version)
    directory.mkdir(parents=True)
    command = ["gh", "release", "download", version, "--repo", REPOSITORY, "--dir", str(directory)]
    for name in sorted(selected):
        command.extend(("--pattern", name))
    subprocess.run(command, check=True, timeout=1800)
    verify_checksums(directory, version)
    downloaded = {}
    for name, item in selected.items():
        path = directory / name
        url = f"https://github.com/{REPOSITORY}/releases/download/{version}/{name}"
        require(path.stat().st_size == item.get("size") and item.get("browser_download_url") == url,
                "actual downloaded bytes differ from Release size/URL")
        downloaded[name] = {"asset_id": item["id"], "size": item["size"], "sha256": file_digest(path),
                            "downloaded_at": datetime.now(timezone.utc).isoformat(), "download_url": url}
    return downloaded


def verify_unchanged(before: dict, after: dict, version: str) -> None:
    selected, current = selected_assets(before, version), selected_assets(after, version)
    require(all(after.get(key) == before.get(key) for key in ("id", "published_at")),
            "Release changed while downloading")
    for name, item in selected.items():
        require(all(current[name].get(key) == item.get(key) for key in ("id", "size", "digest", "updated_at")),
                "Release asset was replaced while downloading")
