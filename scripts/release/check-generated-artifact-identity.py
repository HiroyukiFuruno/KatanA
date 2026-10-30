#!/usr/bin/env python3
"""Verify generated release identity without granting document QA acceptance."""

import argparse
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import zipfile

from document_host_io import require
from generated_asset_identity import module
from generated_asset_verification import verify
from postpublication_source_binding import validate_version


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--version")
    parser.add_argument("--phase", choices=("prepublish", "postpublication"), required=True)
    parser.add_argument("--assets", type=Path)
    parser.add_argument("--build-manifests", type=Path)
    parser.add_argument("--work", type=Path)
    parser.add_argument("--run-id")
    parser.add_argument("--run-attempt")
    args = parser.parse_args()
    try:
        root = args.root.resolve()
        if args.phase == "prepublish":
            require(args.version is not None and args.assets is not None and args.build_manifests is not None,
                    "prepublication requires version, assets and all three build manifests")
            require(args.work is None and args.run_id is None and args.run_attempt is None,
                    "prepublication uses its actual GitHub build run")
            require(os.environ.get("GITHUB_ACTIONS") == "true", "actual publication workflow required")
            builder = module("_generated_identity_source", "record-generated-artifact-identity.py")
            source = builder.source_identity(root)
            verify(root, args.assets, args.build_manifests, validate_version(args.version),
                   source["source_commit"], os.environ.get("GITHUB_RUN_ID", ""), os.environ.get("GITHUB_RUN_ATTEMPT", ""))
        else:
            from future_publication_identity import collect
            require(args.work is not None and args.assets is None and args.build_manifests is None,
                    "postpublication requires a new work directory, never candidate assets/manifests")
            collect(root, args.work, args.run_id, args.run_attempt,
                    validate_version(args.version) if args.version is not None else None)
    except (ValueError, RuntimeError, OSError, KeyError, TypeError, subprocess.SubprocessError,
            tarfile.TarError, zipfile.BadZipFile) as error:
        print(f"ERROR: generated publication identity: {error}", file=sys.stderr)
        return 1
    print("OK: all five publication payloads match exact source/version/run/attempt; document QA not evaluated")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
