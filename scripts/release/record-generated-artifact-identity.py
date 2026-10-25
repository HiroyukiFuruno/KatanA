#!/usr/bin/env python3
"""Build job自身の実payload/source/lock/target identityを独立manifestへ保存する。"""

import argparse
import os
import subprocess
import sys
from pathlib import Path

from document_host_acceptance import provenance
from document_host_io import digest, legacy, read, require, write_new
from document_host_publication import make_manifest


def source_identity(root: Path) -> dict:
    helper = legacy()
    source = {"source_commit": subprocess.run(["git", "-C", str(root), "rev-parse", "HEAD"], check=True,
              capture_output=True, text=True, env=helper.git_environment()).stdout.strip(),
              "source_tree_sha256": helper.source_tree_sha256(root), "cargo_lock_sha256": digest(read(root / "Cargo.lock")),
              "published_dependencies": {name: {key: package[key] for key in ("version", "source", "checksum")}
                                         for name, package in helper.lock_packages(root).items()}}
    provenance(root, source)
    return source


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--platform", choices=("macos", "linux", "windows"), required=True)
    parser.add_argument("--main", type=Path, required=True)
    parser.add_argument("--worker", type=Path, required=True)
    parser.add_argument("--asset", type=Path, action="append", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        require(os.environ.get("GITHUB_ACTIONS") == "true", "generated publication manifests must be produced by the actual build job")
        root = args.root.resolve()
        source = source_identity(root)
        value = make_manifest(args.platform, args.asset, source, args.main, args.worker,
                              run_id=os.environ.get("GITHUB_RUN_ID", ""), run_attempt=os.environ.get("GITHUB_RUN_ATTEMPT", ""))
        provenance(root, source)
        write_new(args.output, value)
    except (ValueError, RuntimeError, OSError, KeyError, subprocess.SubprocessError) as error:
        print(f"ERROR: generated publication identity: {error}", file=sys.stderr)
        return 1
    print("OK: actual build payloads/source/lock/targets recorded")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
