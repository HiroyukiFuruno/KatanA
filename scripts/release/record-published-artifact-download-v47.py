#!/usr/bin/env python3
"""実Releaseの全5assetをdownload/検査し、最終Mac再QAへの実bytes bindingを保存する。"""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import subprocess
import sys
from pathlib import Path

from document_host_io import decode, digest, read, require, write_new
from document_host_publication import allowed_publication_version, file_digest, verify_generated
from generated_asset_identity import assets_for
from postpublication_controller_scope import verify_controller_scope
from published_git_source import git_source_identity, git_workspace_version
from v047_evidence_paths import HOST_RECEIPT

REPOSITORY = "HiroyukiFuruno/KatanA"
TAG = "v0.22.47"


def api(endpoint: str) -> object:
    result = subprocess.run(["gh", "api", endpoint], check=True, capture_output=True, timeout=120)
    return decode(result.stdout)


def record(root: Path, directory: Path, manifests: Path, tag: str = TAG) -> tuple[dict, dict]:
    tag = allowed_publication_version(tag)
    require(tag == TAG, "v47 recorder requires v0.22.47")
    require(not directory.exists(), "refusing to reuse candidate/preexisting download directory")
    release = api(f"repos/{REPOSITORY}/releases/tags/{tag}")
    require(isinstance(release, dict) and release.get("tag_name") == tag and release.get("draft") is False
            and isinstance(release.get("published_at"), str), "actual published Release does not exist")
    commit = api(f"repos/{REPOSITORY}/commits/{tag}")
    require(isinstance(commit, dict) and isinstance(commit.get("sha"), str), "actual Release commit could not be resolved")
    # WHY: 公開後の検証コードと配布物のソースを区別し、実タグの全Git objectをproducerと照合する。
    verify_controller_scope(root, commit["sha"])
    source = git_source_identity(root, commit["sha"])
    require(git_workspace_version(root, commit["sha"]) == tag, "actual tag and Cargo version differ")
    expected = set().union(*assets_for(tag).values())
    selected = select_assets(release, expected)
    downloaded = download_assets(directory, tag, selected, expected)
    generated = verify_generated(root, directory, manifests, source, published_commit=commit["sha"], version=tag)
    verify_release_unchanged(release, selected, tag, source)
    target = generated["targets"]["macos-arm64"]
    receipt = {"schema": "katana-release-download-v1", "release_url": release["html_url"], "release_id": release["id"],
            "tag": tag, "draft": False, "published_at": release["published_at"], "source_commit": source["source_commit"],
            "provenance": source, "assets": downloaded, "downloaded_macos": "KatanA-macOS.zip",
            "main_sha256": target["main_sha256"], "sidecar_sha256": target["sidecar_sha256"],
            "producer_sha256": digest(read(Path(__file__)))}
    return receipt, generated["manifests"]


def select_assets(release: dict, expected: set[str]) -> dict:
    assets = release.get("assets")
    require(isinstance(assets, list), "Release asset metadata missing")
    selected = {}
    for item in assets:
        require(isinstance(item, dict), "invalid Release asset metadata")
        name = item.get("name")
        if name in expected | {"checksums.txt"}:
            require(name not in selected, "duplicate actual Release asset")
            selected[name] = item
    require(set(selected) == expected | {"checksums.txt"}, "Release is missing one of the five assets/checksums")
    return selected


def download_assets(directory: Path, tag: str, selected: dict, expected: set[str]) -> dict:
    directory.mkdir(parents=True)
    command = ["gh", "release", "download", tag, "--repo", REPOSITORY, "--dir", str(directory)]
    for name in sorted(selected):
        command.extend(("--pattern", name))
    subprocess.run(command, check=True, timeout=1800)
    checksums = {}
    for line in (directory / "checksums.txt").read_text().splitlines():
        value, separator, name = line.partition("  ")
        require(separator == "  " and name in expected and name not in checksums, "actual checksum file contains missing/duplicate/unknown entry")
        checksums[name] = value
    require(set(checksums) == expected, "actual checksum set differs from five release assets")
    downloaded = {}
    for name in sorted(expected):
        path, item = directory/name, selected[name]
        actual_sha = file_digest(path)
        require(checksums[name] == actual_sha and path.stat().st_size == item.get("size"), "actual downloaded bytes differ from Release checksum/size")
        url = f"https://github.com/{REPOSITORY}/releases/download/{tag}/{name}"
        require(item.get("browser_download_url") == url, "Release asset download URL differs")
        downloaded[name] = {"asset_id": item["id"], "size": item["size"], "sha256": actual_sha,
                            "downloaded_at": datetime.now(timezone.utc).isoformat(), "download_url": url}
    return downloaded


def verify_release_unchanged(release: dict, selected: dict, tag: str, source: dict) -> None:
    # WHY: Releaseを読んだ後に公開物が置き換わっていないか、asset ID/size/digestとtag commitをもう一度照合する。
    after = api(f"repos/{REPOSITORY}/releases/tags/{tag}")
    require(after.get("id") == release.get("id") and after.get("draft") is False
            and after.get("published_at") == release.get("published_at"), "Release changed while downloading")
    for name in selected:
        matches = [item for item in after.get("assets", []) if item.get("name") == name]
        require(len(matches) == 1 and all(matches[0].get(key) == selected[name].get(key) for key in ("id", "size", "digest", "updated_at")),
                "Release asset was replaced while downloading")
    require(api(f"repos/{REPOSITORY}/commits/{tag}").get("sha") == source["source_commit"], "Release tag changed while downloading")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--download-directory", type=Path, required=True)
    parser.add_argument("--build-manifests", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--version", default=TAG, choices=(TAG,))
    args = parser.parse_args()
    try:
        root = args.root.resolve()
        output = args.output.absolute()
        require(output == root / HOST_RECEIPT and output.resolve() == output,
                "canonical v47 download receipt required")
        require(not output.exists(), "refusing to replace an earlier Release receipt")
        value, manifests = record(root, args.download_directory.resolve(), args.build_manifests.resolve(), args.version)
        refs = {}
        for platform, generated in manifests.items():
            path = output.with_name(output.stem+f"-build-{platform}.json")
            ref = write_new(path, generated)
            ref["path"] = path.relative_to(root).as_posix()
            refs[platform] = ref
        value["build_manifests"] = refs
        write_new(output, value)
    except (ValueError, RuntimeError, OSError, KeyError, subprocess.SubprocessError) as error:
        print(f"ERROR: actual published asset download: {error}", file=sys.stderr)
        return 1
    print("OK: actual Release/five downloaded asset payloads checked; Mac host QA remains pending")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
