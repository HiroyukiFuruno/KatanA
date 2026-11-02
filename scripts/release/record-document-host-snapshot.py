#!/usr/bin/env python3
"""実buildの前後と実bundle/runを記録する。古い実行物のsourceを現在値から補完しない。"""

from __future__ import annotations

import argparse
from pathlib import Path
import re
import subprocess
import sys

from document_host_acceptance import provenance
from document_host_io import decode, digest, read, require, write_new
from document_host_publication import file_digest, module, verify_download_receipt


def source(root: Path) -> dict:
    return module("_host_actual_source_snapshot", "record-generated-artifact-identity.py").source_identity(root)


def actual_bundle(root: Path, before_path: Path, after_path: Path, archive: Path,
                  main: Path, worker: Path, target: str) -> dict:
    before, after = decode(read(before_path)), decode(read(after_path))
    require(before == after == source(root), "actual package source/lock/registry changed or was not captured before build")
    require(target in {"macos-arm64", "macos-x86_64"}, "only actual macOS host target allowed")
    require(main.is_absolute() and worker.is_absolute() and archive.is_absolute(), "actual absolute bundle/archive paths required")
    identity = module("_host_candidate_actual_archive", "check-published-artifact-identity.py")
    architecture = module("_host_candidate_actual_architecture", "verify-binary-architecture.py")
    architecture.verify("macos", archive)
    main_sha, worker_sha = file_digest(main), file_digest(worker)
    prefix = "KatanA Desktop.app/Contents/MacOS/"
    require(identity.zip_digest(archive, prefix+"KatanA") == main_sha
            and identity.zip_digest(archive, prefix+"kdv-office-worker") == worker_sha,
            "actual unpacked candidate differs from archive payload")
    return {"target": target, "main_path": str(main), "worker_path": str(worker),
            "main_sha256": main_sha, "sidecar_sha256": worker_sha,
            "archive_sha256": file_digest(archive), "provenance": before}


def release_bundle(root: Path, receipt_path: Path, archive: Path, main: Path, worker: Path, target: str,
                   publication_version: str = "v0.22.42") -> dict:
    require(receipt_path.is_absolute() and receipt_path.is_relative_to(root), "actual public Release receipt required")
    raw = read(receipt_path)
    receipt = decode(raw)
    require(isinstance(receipt, dict) and target in {"macos-arm64", "macos-x86_64"}, "actual Release target required")
    source_record = provenance(root, receipt["provenance"])
    bundle = {"target": target, "main_path": str(main), "worker_path": str(worker), "main_sha256": file_digest(main),
              "sidecar_sha256": file_digest(worker), "archive_sha256": file_digest(archive), "provenance": source_record}
    verify_download_receipt(root, {"status": "published_macos_host_accepted",
                                  "receipt": {"path": receipt_path.relative_to(root).as_posix(), "sha256": digest(raw)}}, source_record, bundle, publication_version)
    identity = module("_actual_published_mac_archive", "check-published-artifact-identity.py")
    prefix = "KatanA Desktop.app/Contents/MacOS/"
    require(identity.zip_digest(archive, prefix+"KatanA") == bundle["main_sha256"]
            and identity.zip_digest(archive, prefix+"kdv-office-worker") == bundle["sidecar_sha256"],
            "actual published unpacked main/worker differs from downloaded ZIP")
    module("_actual_published_mac_architecture", "verify-binary-architecture.py").verify("macos", archive)
    return bundle


def run_snapshot(root: Path, directory: Path, bundle_path: Path, archive: Path, profile: Path, when: str) -> None:
    require(directory.is_absolute() and directory.is_dir() and not directory.is_symlink(), "actual run directory required")
    require(profile.is_absolute() and profile.is_dir() and not profile.is_symlink()
            and profile.resolve().is_relative_to((root / "tmp").resolve()), "existing isolated QA profile only; personal profile forbidden")
    bundle = decode(read(bundle_path))
    require(isinstance(bundle, dict) and bundle.get("provenance") == source(root), "run source differs from actual package source")
    provenance(root, bundle["provenance"])
    require(file_digest(Path(bundle["main_path"])) == bundle["main_sha256"]
            and file_digest(Path(bundle["worker_path"])) == bundle["sidecar_sha256"]
            and file_digest(archive) == bundle["archive_sha256"], "actual bundle/archive changed")
    if when == "before":
        pid = int((directory / "app.pid").read_text().strip())
        from process_memory_sampler import process_identity
        identity = process_identity(pid)
        require(identity["executable"] == bundle["main_path"], "launched process is not selected bundle")
        expected_arch = "arm64" if bundle["target"] == "macos-arm64" else "x86_64"
        launch = decode(read(directory / "launch.json"))
        require(isinstance(launch, dict) and launch.get("argv") == ["/usr/bin/arch", "-"+expected_arch, bundle["main_path"]]
                and launch.get("target") == bundle["target"], "actual explicit launch slice differs from declared target")
        # WHY: 環境全体は私的情報を含むため保存/表示せず、このrunの固定変数だけを抽出する。
        command_environment = subprocess.run(["ps", "eww", "-p", str(pid), "-o", "command="], check=True,
                                              capture_output=True, text=True, timeout=10).stdout

        def environment(name: str) -> str | None:
            values = re.findall(r"(?:^|\s)"+re.escape(name)+r"=([^\s]*)", command_environment)
            require(len(values) <= 1, "ambiguous actual process environment")
            return values[0] if values else None

        require(environment("KATANA_CONFIG_DIR") == str(profile), "actual process uses another config/profile")
        require(environment("KATANA_DOCUMENT_RESOURCE_RECEIPT_DIR") == str(directory / "resource-receipts")
                and environment("KATANA_DOCUMENT_RESOURCE_RUN_ID") == directory.name,
                "readonly recorder environment differs from this run")
        actual_debug = environment("DEBUG")
        require(actual_debug in {"true", "false"}, "actual DEBUG value must be recorded exactly")
        require(all(environment(name) in {None, "0", "false"} for name in ("MallocStackLogging", "MallocStackLoggingNoCompact")),
                "allocator inspector environment is enabled")
        require("--memory-diagnostics" not in command_environment, "vmmap/heap inspector mode is enabled")
        write_new(directory / "run-settings.json", {"debug": actual_debug == "true", "readonly_recorder": True,
                  "malloc_inspector": False, "vmmap_inspector": False, "heap_inspector": False,
                  "isolation": "isolated_profile", "clean_machine": False, "protections_disabled": False,
                  "personal_profile_changed": False, "terms_scope": "this_isolated_run_only"})
    elif when == "after":
        require((directory / "app.exit").read_text().strip() == "0", "normal app exit required")
    else:
        require(False, "unknown run snapshot moment")
    write_new(directory / f"source-identity.{when}.json", bundle["provenance"])
    write_new(directory / f"bundle-identity.{when}.json", bundle)
    with (directory / f"head.{when}").open("x") as handle:
        handle.write(bundle["provenance"]["source_commit"]+"\n")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    modes = parser.add_subparsers(dest="mode", required=True)
    snapshot = modes.add_parser("source")
    snapshot.add_argument("--output", type=Path, required=True)
    bundle = modes.add_parser("bundle")
    for name in ("package-source-before", "package-source-after", "archive", "main", "worker", "output"):
        bundle.add_argument("--"+name, type=Path, required=True)
    bundle.add_argument("--target", choices=("macos-arm64", "macos-x86_64"), required=True)
    published = modes.add_parser("bundle-from-release")
    for name in ("receipt", "archive", "main", "worker", "output"):
        published.add_argument("--"+name, type=Path, required=True)
    published.add_argument("--target", choices=("macos-arm64", "macos-x86_64"), required=True)
    published.add_argument("--publication-version", choices=("v0.22.42", "v0.22.47"), default="v0.22.42")
    run = modes.add_parser("run")
    for name in ("run-directory", "package-bundle-manifest", "archive", "profile"):
        run.add_argument("--"+name, type=Path, required=True)
    run.add_argument("--moment", choices=("before", "after"), required=True)
    args = parser.parse_args()
    try:
        root = args.root.resolve()
        if args.mode == "source":
            write_new(args.output, source(root))
        elif args.mode == "bundle":
            write_new(args.output, actual_bundle(root, args.package_source_before, args.package_source_after,
                                                args.archive, args.main, args.worker, args.target))
        elif args.mode == "bundle-from-release":
            write_new(args.output, release_bundle(root, args.receipt, args.archive, args.main, args.worker, args.target, args.publication_version))
        else:
            run_snapshot(root, args.run_directory, args.package_bundle_manifest, args.archive, args.profile, args.moment)
    except (ValueError, RuntimeError, OSError, KeyError, TypeError, subprocess.SubprocessError) as error:
        print(f"ERROR: actual host provenance snapshot: {error}", file=sys.stderr)
        return 1
    print("OK: actual source/bundle/run snapshot recorded")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
