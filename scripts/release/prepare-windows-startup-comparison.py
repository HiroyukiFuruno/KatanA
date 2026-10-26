#!/usr/bin/env python3
"""Prepare a byte-bound Windows startup comparison payload.

This helper only validates the build-job identity and extracts the existing
startup-smoke PowerShell body.  It does not unpack or execute the product.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import zipfile
from pathlib import Path
from typing import Any


VERSION = "v0.22.42"
PLATFORM = "windows"
ARCHIVE_NAME = "KatanA-windows-x86_64.zip"
MAIN_ENTRY = "KatanA.exe"
WORKER_ENTRY = "kdv-office-worker.exe"
ORIGINAL_BLOCK_SHA256 = "f4a7e1010e974d03171d5d5ba4b046da701328c98fc5670d08559afeaac4b852"
SMOKE_NAME = "Smoke launch packaged Windows app"


class PreparationError(ValueError):
    """Raised when comparison inputs are not bound to one build payload."""


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise PreparationError(message)


def _sha256(path: Path) -> str:
    _require(path.is_file() and not path.is_symlink(), f"regular file required: {path}")
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def _hex_sha(value: Any, label: str) -> str:
    _require(isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) is not None,
             f"{label} must be a lowercase SHA-256")
    return value


def _positive_decimal(value: Any, label: str) -> str:
    _require(isinstance(value, str) and re.fullmatch(r"[1-9][0-9]*", value) is not None,
             f"{label} must be a positive decimal string")
    return value


def _read_json(path: Path) -> dict[str, Any]:
    _require(path.is_file() and not path.is_symlink(), f"manifest must be a regular file: {path}")
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise PreparationError(f"invalid identity manifest: {error}") from error
    _require(isinstance(value, dict), "identity manifest must be an object")
    return value


def _git_head(root: Path) -> str:
    environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
    environment.update({"GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull})
    try:
        result = subprocess.run(
            ["git", "-C", str(root), "rev-parse", "HEAD"],
            check=True,
            capture_output=True,
            text=True,
            env=environment,
        )
    except (OSError, subprocess.SubprocessError) as error:
        raise PreparationError(f"cannot read current checkout commit: {error}") from error
    commit = result.stdout.strip()
    _require(re.fullmatch(r"[0-9a-f]{40}", commit) is not None,
             "current checkout did not yield a full commit")
    return commit


def _zip_entry_sha256(archive: Path, entry_name: str) -> str:
    try:
        with zipfile.ZipFile(archive) as bundle:
            names = bundle.namelist()
            _require(names.count(entry_name) == 1, f"ZIP must contain exactly one {entry_name}")
            info = bundle.getinfo(entry_name)
            _require(not info.is_dir(), f"ZIP entry is a directory: {entry_name}")
            digest = hashlib.sha256()
            with bundle.open(info) as entry:
                for chunk in iter(lambda: entry.read(1024 * 1024), b""):
                    digest.update(chunk)
            return digest.hexdigest()
    except (OSError, zipfile.BadZipFile, KeyError) as error:
        raise PreparationError(f"cannot inspect Windows ZIP: {error}") from error


def validate_payload(
    payload_dir: Path,
    *,
    expected_version: str,
    expected_source_commit: str,
    expected_run_id: str,
    expected_run_attempt: str,
    current_root: Path,
) -> dict[str, Any]:
    """Validate the downloaded build payload and return its bound identity."""
    _require(expected_version == VERSION, "comparison is fixed to v0.22.42")
    _require(re.fullmatch(r"[0-9a-f]{40}", expected_source_commit) is not None,
             "expected source commit must be a full commit")
    _require(_git_head(current_root) == expected_source_commit,
             "comparison checkout differs from expected source commit")
    run_id = _positive_decimal(expected_run_id, "expected run id")
    run_attempt = _positive_decimal(expected_run_attempt, "expected run attempt")
    manifest_path = payload_dir / "tmp" / "identity-windows.json"
    manifest = _read_json(manifest_path)
    _require(manifest.get("schema") == "katana-generated-asset-identity-v1", "unexpected identity schema")
    _require(manifest.get("version") == expected_version, "identity version differs")
    _require(manifest.get("platform") == PLATFORM, "identity platform differs")
    _require(manifest.get("github_run_id") == run_id, "identity run id differs")
    _require(manifest.get("github_run_attempt") == run_attempt, "identity run attempt differs")
    provenance = manifest.get("provenance")
    _require(isinstance(provenance, dict), "identity provenance is missing")
    _require(provenance.get("source_commit") == expected_source_commit,
             "identity source commit differs")
    targets = manifest.get("targets")
    _require(isinstance(targets, dict) and set(targets) == {"windows-x86_64"},
             "identity Windows target set differs")
    target = targets["windows-x86_64"]
    _require(isinstance(target, dict), "Windows target identity is missing")
    main_sha = _hex_sha(target.get("main_sha256"), "target main hash")
    worker_sha = _hex_sha(target.get("sidecar_sha256"), "target worker hash")
    assets = manifest.get("assets")
    _require(isinstance(assets, dict) and set(assets) == {ARCHIVE_NAME, "KatanA-windows-x86_64.msi"},
             "identity Windows asset set differs")
    archive_descriptor = assets[ARCHIVE_NAME]
    _require(isinstance(archive_descriptor, dict), "Windows archive identity is missing")
    archive = payload_dir / ARCHIVE_NAME
    archive_sha = _hex_sha(archive_descriptor.get("sha256"), "archive hash")
    archive_size = archive_descriptor.get("size")
    _require(isinstance(archive_size, int) and archive_size > 0, "archive size is invalid")
    _require(archive.is_file() and not archive.is_symlink(), "Windows archive is missing")
    _require(archive.stat().st_size == archive_size, "Windows archive size differs from manifest")
    _require(_sha256(archive) == archive_sha, "Windows archive hash differs from manifest")
    _require(_zip_entry_sha256(archive, MAIN_ENTRY) == main_sha, "ZIP main hash differs from manifest")
    _require(_zip_entry_sha256(archive, WORKER_ENTRY) == worker_sha, "ZIP worker hash differs from manifest")
    return {
        "schema": "katana-windows-startup-comparison-payload-v1",
        "version": expected_version,
        "platform": PLATFORM,
        "source_commit": expected_source_commit,
        "github_run_id": run_id,
        "github_run_attempt": run_attempt,
        "archive": ARCHIVE_NAME,
        "archive_sha256": archive_sha,
        "archive_size": archive_size,
        "main_entry": MAIN_ENTRY,
        "main_sha256": main_sha,
        "worker_entry": WORKER_ENTRY,
        "worker_sha256": worker_sha,
    }


def extract_original_smoke_block(workflow: Path) -> tuple[str, str]:
    """Extract the one existing smoke step and de-indent its PowerShell body."""
    source = workflow.read_text(encoding="utf-8")
    starts = [match.start() for match in re.finditer(
        r"(?m)^      - name: Smoke launch packaged Windows app\n", source)]
    _require(len(starts) == 1, "trusted workflow must contain exactly one Windows smoke step")
    start = starts[0]
    next_step = re.search(r"(?m)^      - name: ", source[start + 1:])
    _require(next_step is not None, "Windows smoke step has no following step boundary")
    end = start + 1 + next_step.start()
    block = source[start:end]
    _require(_sha256_bytes(block.encode("utf-8")) == ORIGINAL_BLOCK_SHA256,
             "original Windows smoke block changed")
    expected_header = (
        "      - name: Smoke launch packaged Windows app\n"
        "        id: windows_startup_smoke\n"
        "        shell: pwsh\n"
        "        run: |\n"
    )
    _require(block.startswith(expected_header), "Windows smoke step header changed")
    body = block[len(expected_header):]
    lines = body.splitlines(keepends=True)
    _require(lines and lines[-1].strip() == "", "Windows smoke step separator changed")
    lines = lines[:-1]
    _require(lines and all(not line.strip() or line.startswith("          ") for line in lines),
             "Windows smoke body indentation changed")
    powershell = "".join(line[10:] for line in lines)
    _require(powershell and not powershell.endswith("\n      "), "invalid Windows smoke body")
    return block, powershell


def prepare(args: argparse.Namespace) -> None:
    payload = Path(args.payload_dir).resolve()
    workflow = Path(args.workflow).resolve()
    output_script = Path(args.output_script).resolve()
    output_metadata = Path(args.output_metadata).resolve()
    identity = validate_payload(
        payload,
        expected_version=args.expected_version,
        expected_source_commit=args.expected_source_commit,
        expected_run_id=args.expected_run_id,
        expected_run_attempt=args.expected_run_attempt,
        current_root=Path(args.current_root).resolve(),
    )
    block, powershell = extract_original_smoke_block(workflow)
    output_script.parent.mkdir(parents=True, exist_ok=True)
    output_metadata.parent.mkdir(parents=True, exist_ok=True)
    output_script.write_text(powershell, encoding="utf-8", newline="")
    identity.update({
        "workflow_block_sha256": _sha256_bytes(block.encode("utf-8")),
        "powershell_body_sha256": _sha256_bytes(powershell.encode("utf-8")),
    })
    output_metadata.write_text(json.dumps(identity, indent=2) + "\n", encoding="utf-8", newline="")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--payload-dir", required=True)
    parser.add_argument("--workflow", required=True)
    parser.add_argument("--current-root", required=True)
    parser.add_argument("--output-script", required=True)
    parser.add_argument("--output-metadata", required=True)
    parser.add_argument("--expected-version", required=True)
    parser.add_argument("--expected-source-commit", required=True)
    parser.add_argument("--expected-run-id", required=True)
    parser.add_argument("--expected-run-attempt", required=True)
    args = parser.parse_args()
    try:
        prepare(args)
    except (OSError, PreparationError) as error:
        print(f"ERROR: Windows startup comparison preparation: {error}")
        return 1
    print("OK: Windows comparison payload and original smoke body prepared")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
