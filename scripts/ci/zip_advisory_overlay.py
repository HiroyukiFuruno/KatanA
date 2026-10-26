#!/usr/bin/env python3
"""Fail-closed, time-limited cargo-deny overlay for the ZIP advisories."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import subprocess
import sys
import tempfile
from pathlib import Path
from zip_advisory_policy import ADVISORIES, EXPIRES_ON, overlay_config, policy_expired, validate_metadata, validate_policy_ids
import zstd_advisory_policy


def _run_manifest(cargo_deny: str, source: str, manifest: Path, today: dt.date) -> int:
    metadata = subprocess.run(["cargo", "metadata", "--locked", "--all-features", "--format-version", "1", "--manifest-path", str(manifest)], check=False, capture_output=True, text=True)
    if metadata.returncode:
        sys.stderr.write(metadata.stderr)
        return metadata.returncode
    try:
        config = zstd_advisory_policy.overlay_config(source, json.loads(metadata.stdout), today)
    except (ValueError, json.JSONDecodeError) as error:
        print(f"ZIP advisory policy invalid for {manifest}: {error}", file=sys.stderr)
        return 78
    ids = (*ADVISORIES, zstd_advisory_policy.ADVISORY_ID) if zstd_advisory_policy.ADVISORY_ID in config else ADVISORIES
    print(f"ZIP/zstd advisory overlay: ids={','.join(ids)} expires={EXPIRES_ON.isoformat()} manifest={manifest}", file=sys.stderr)
    with tempfile.NamedTemporaryFile("w", suffix=".toml", delete=False, encoding="utf-8") as handle:
        handle.write(config)
        overlay = Path(handle.name)
    try:
        result = subprocess.run([cargo_deny, "--locked", "--manifest-path", str(manifest), "--config", str(overlay), "check", "--hide-inclusion-graph"], check=False)
        return result.returncode
    finally:
        overlay.unlink(missing_ok=True)


def run(cargo_deny: str, deny_file: Path, manifests: list[Path]) -> int:
    today = dt.datetime.now(dt.timezone.utc).date()
    if policy_expired(today):
        print("ZIP advisory exception policy expired", file=sys.stderr)
        return 78
    source = deny_file.read_text(encoding="utf-8")
    for manifest in manifests:
        result = _run_manifest(cargo_deny, source, manifest, today)
        if result:
            return result
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--deny-file", type=Path, required=True)
    parser.add_argument("--cargo-deny", default="cargo-deny")
    parser.add_argument("manifests", nargs="+", type=Path)
    args = parser.parse_args()
    return run(args.cargo_deny, args.deny_file, args.manifests)


if __name__ == "__main__":
    raise SystemExit(main())
