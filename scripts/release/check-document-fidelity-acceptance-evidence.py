#!/usr/bin/env python3
"""Validate machine-readable v0.22.42 document acceptance evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import ntpath
import re
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Any


TARGET = "v0.22.42"
EVIDENCE_RELATIVE = (
    "openspec/changes/post-v0-22-41-document-fidelity-regressions/evidence/"
    "document-acceptance-v0.22.42.json"
)
CRATES_IO_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"
RENDERER_PACKAGES = {
    "katana-document-viewer",
    "katana-render-runtime",
    "katana-ui-core",
}
SUPPORTED_TARGETS = {
    "macos-arm64",
    "macos-x86_64",
    "linux-x86_64",
    "windows-x86_64",
}
ORIGINAL_HTML_SHA256 = "c02d2d7a2420e4e15e3d98a044a310c67bc75fa858c95c4867b9c3f5d7aca012"
SHA256 = 64
SHA256_PATTERN = re.compile(r"[0-9a-fA-F]{64}")
SOURCE_ROOTS = (
    "Cargo.toml",
    "Cargo.lock",
    ".cargo",
    "crates",
    "vendor",
    "scripts",
    "platforms",
    ".github",
    "assets",
    "just",
    "Justfile",
    "lefthook.yml",
    "Makefile",
)
EXCLUDED_PARTS = {"evidence", "tasks.md", "docs", "target", "__pycache__", "node_modules"}
SUPPLIED_OFFICE_FIXTURES = {
    "04bf541a7cdb4e332e1f58cc8dc14148e2fd46110fb6c73a77bbafbcaa768aaa": "xlsx",
    "c77f80d3f28daf69c19d3c9986e60f2bf1bdc2cba81b55aea3fb2e741ce00756": "xlsx",
    "77b03ab621c3d197db1bd7b1a6a6a5b544fac56ecf12c24eca2f4fd6fd3e15a9": "xlsx",
    "34f462ac1c38e581f8b286f549aaf54fe55cd45c6d28a16cfbf1ebb163b3af57": "pptx",
    "0d034ac494a6c7757516d99ea2dd7f8d47217c47fd050ee84ce469ee708d9cdb": "pptx",
    "ebe4633b927ecf2895d057ac3301d4f6ba2fd8dde6cfdbe9cdab4039dc4e37f4": "pptx",
}


class AcceptanceEvidenceError(RuntimeError):
    """Raised when acceptance evidence is missing or cannot prove release readiness."""


def fail(message: str) -> None:
    raise AcceptanceEvidenceError(message)


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def require_sha256(value: object, name: str) -> str:
    if not isinstance(value, str) or SHA256_PATTERN.fullmatch(value) is None:
        fail(f"{name} must be a SHA-256 hex string")
    return value.lower()


def source_paths(root: Path) -> list[Path]:
    relative_paths: list[str] = []
    try:
        result = subprocess.run(
            ["git", "-C", str(root), "ls-files", "--cached", "--others", "--exclude-standard", "-z", "--", *SOURCE_ROOTS],
            check=False,
            capture_output=True,
        )
    except OSError as error:
        fail(f"cannot enumerate source tree with git: {error}")
    if result.returncode != 0:
        detail = result.stderr.decode(errors="replace").strip()
        fail(f"cannot enumerate source tree with git: {detail or 'git ls-files failed'}")
    relative_paths = [item.decode() for item in result.stdout.split(b"\0") if item]

    paths = []
    for relative in sorted(set(relative_paths)):
        path = Path(relative)
        if any(part in EXCLUDED_PARTS for part in path.parts):
            continue
        absolute = root / path
        if absolute.is_file():
            paths.append(absolute)
    return paths


def source_tree_sha256(root: Path) -> str:
    digest = hashlib.sha256()
    for path in source_paths(root):
        relative = path.relative_to(root).as_posix().encode()
        data = path.read_bytes()
        digest.update(len(relative).to_bytes(8, "big"))
        digest.update(relative)
        digest.update(len(data).to_bytes(8, "big"))
        digest.update(data)
    return digest.hexdigest()


def lock_packages(root: Path) -> dict[str, dict[str, Any]]:
    lock_path = root / "Cargo.lock"
    if not lock_path.is_file():
        fail(f"Cargo.lock is missing: {lock_path}")
    with lock_path.open("rb") as handle:
        lock = tomllib.load(handle)
    package_records = [
        package
        for package in lock.get("package", [])
        if isinstance(package, dict) and package.get("name") in RENDERER_PACKAGES
    ]
    packages = {str(package["name"]): package for package in package_records}
    if set(packages) != RENDERER_PACKAGES:
        fail("Cargo.lock must contain exactly the KDV, KRR, and KUC packages")
    if len(package_records) != len(RENDERER_PACKAGES):
        fail("Cargo.lock must resolve exactly one KDV, KRR, and KUC package")
    for name, package in packages.items():
        if package.get("source") != CRATES_IO_SOURCE or not isinstance(package.get("version"), str):
            fail(f"Cargo.lock {name} must resolve from canonical crates.io")
    return packages


def require_finite_number(value: object, name: str) -> float:
    if not isinstance(value, (int, float)) or isinstance(value, bool) or not math.isfinite(value):
        fail(f"{name} must be a finite non-negative number")
    if value < 0:
        fail(f"{name} must be a finite non-negative number")
    return float(value)


def require_positive_finite_number(value: object, name: str) -> float:
    number = require_finite_number(value, name)
    if number <= 0:
        fail(f"{name} must be a finite positive number")
    return number


def require_positive_integer(value: object, name: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
        fail(f"{name} must be a positive integer")
    return value


def require_canonical_path(value: object, name: str) -> str:
    if not isinstance(value, str) or not value:
        fail(f"{name} must be a canonical executable path")
    if not (Path(value).is_absolute() or ntpath.isabs(value)):
        fail(f"{name} must be an absolute canonical executable path")
    if ".." in Path(value).parts or ".." in ntpath.normpath(value).split(ntpath.sep):
        fail(f"{name} must not contain parent traversal")
    if value != str(Path(value)) and value != ntpath.normpath(value):
        fail(f"{name} must be normalized")
    return value


def verify_html(value: object) -> None:
    if not isinstance(value, dict):
        fail("HTML acceptance evidence is missing")
    if value.get("fixture_sha256") != ORIGINAL_HTML_SHA256:
        fail("HTML evidence must identify the supplied original fixture")
    if value.get("runner_mode") != "packaged_main":
        fail("HTML evidence must come from packaged_main, not in_process")
    if value.get("status") != "passed" or value.get("normal_close") is not True:
        fail("HTML evidence must record a passed run with normal close")
    first_frame = require_positive_finite_number(value.get("first_frame_ms"), "HTML first_frame_ms")
    if first_frame > 60000:
        fail("HTML first frame must be within 60000 ms")
    require_finite_number(value.get("cpu_percent"), "HTML cpu_percent")
    require_positive_finite_number(value.get("rss_bytes"), "HTML rss_bytes")


def verify_packaged(value: object) -> None:
    if not isinstance(value, dict) or set(value) != SUPPORTED_TARGETS:
        fail("packaged evidence must cover every declared target")
    for target, record in value.items():
        if not isinstance(record, dict) or record.get("status") != "passed":
            fail(f"packaged evidence is not passed for {target}")
        if record.get("runner_mode") != "packaged_main":
            fail(f"packaged evidence runner mode is invalid for {target}")
        if record.get("clean_machine") is not True or record.get("normal_close") is not True:
            fail(f"packaged evidence must be a clean-machine normal close for {target}")
        pid = require_positive_integer(record.get("pid"), f"{target}.pid")
        sidecar_pid = require_positive_integer(record.get("sidecar_pid"), f"{target}.sidecar_pid")
        if sidecar_pid == pid:
            fail(f"packaged main and sidecar PIDs must differ for {target}")
        before = require_positive_integer(record.get("heartbeat_frame_before"), f"{target}.heartbeat_frame_before")
        after = require_positive_integer(record.get("heartbeat_frame_after"), f"{target}.heartbeat_frame_after")
        if after <= before:
            fail(f"packaged heartbeat did not advance for {target}")
        require_finite_number(record.get("cpu_percent"), f"{target}.cpu_percent")
        require_positive_finite_number(record.get("rss_bytes"), f"{target}.rss_bytes")
        require_canonical_path(record.get("main_path"), f"{target}.main_path")
        sidecar_path = require_canonical_path(record.get("sidecar_path"), f"{target}.sidecar_path")
        observed_sidecar_path = require_canonical_path(
            record.get("observed_sidecar_path"), f"{target}.observed_sidecar_path"
        )
        if observed_sidecar_path != sidecar_path:
            fail(f"packaged sidecar path mismatch for {target}")
        for artifact in ("main_sha256", "sidecar_sha256", "observed_main_sha256", "observed_sidecar_sha256"):
            require_sha256(record.get(artifact), f"{target}.{artifact}")
        if record["main_sha256"].lower() != record["observed_main_sha256"].lower():
            fail(f"packaged main identity mismatch for {target}")
        if record["sidecar_sha256"].lower() != record["observed_sidecar_sha256"].lower():
            fail(f"packaged sidecar identity mismatch for {target}")


def verify_office(value: object) -> None:
    if not isinstance(value, list) or len(value) < len(SUPPLIED_OFFICE_FIXTURES):
        fail("Office evidence must contain all six fixture results")
    for index, record in enumerate(value):
        if not isinstance(record, dict) or record.get("status") != "passed":
            fail(f"Office fixture {index} is not passed")
        format_name = record.get("format")
        if format_name not in {"docx", "xlsx", "pptx"}:
            fail(f"Office fixture {index} has an invalid format")
        input_sha = require_sha256(record.get("input_sha256"), f"Office fixture {index}.input_sha256")
        require_positive_finite_number(record.get("first_frame_ms"), f"Office fixture {index}.first_frame_ms")
        require_positive_integer(record.get("item_count"), f"Office fixture {index}.item_count")
        if record.get("release_worker") is not True:
            fail(f"Office fixture {index} must use the release worker")
        if input_sha in SUPPLIED_OFFICE_FIXTURES and format_name != SUPPLIED_OFFICE_FIXTURES[input_sha]:
            fail(f"Office fixture {index} format does not match its supplied input hash")
    input_hashes = [record["input_sha256"].lower() for record in value]
    if len(set(input_hashes)) != len(input_hashes):
        fail("Office evidence must contain distinct input fixture hashes")
    if not set(SUPPLIED_OFFICE_FIXTURES).issubset(input_hashes):
        fail("Office evidence must include all six supplied fixture hashes")
    formats = {record["format"] for record in value}
    if not {"xlsx", "pptx"}.issubset(formats):
        fail("Office evidence must include XLSX and PPTX fixtures")


def verify(root: Path, evidence_path: Path | None = None) -> None:
    evidence_path = evidence_path or root / EVIDENCE_RELATIVE
    if not evidence_path.is_file():
        fail(f"acceptance evidence is required: {evidence_path}")
    try:
        evidence = json.loads(evidence_path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as error:
        fail(f"acceptance evidence is not valid JSON: {error}")
    if (
        not isinstance(evidence, dict)
        or not isinstance(evidence.get("schema_version"), int)
        or isinstance(evidence.get("schema_version"), bool)
        or evidence.get("schema_version") != 1
        or evidence.get("target") != TARGET
    ):
        fail("acceptance evidence schema or target is invalid")
    if evidence.get("runner_mode") != "packaged_main":
        fail("acceptance evidence runner_mode must be packaged_main")
    if evidence.get("published_registry_graph") is not True:
        fail("acceptance evidence must use the published registry graph")
    lock_path = root / "Cargo.lock"
    expected_lock_sha = sha256_bytes(lock_path.read_bytes()) if lock_path.is_file() else ""
    if require_sha256(evidence.get("cargo_lock_sha256"), "cargo_lock_sha256") != expected_lock_sha:
        fail("acceptance evidence Cargo.lock hash is stale")
    if require_sha256(evidence.get("source_tree_sha256"), "source_tree_sha256") != source_tree_sha256(root):
        fail("acceptance evidence source tree hash is stale")
    packages = lock_packages(root)
    declared = evidence.get("published_dependencies")
    if not isinstance(declared, dict) or set(declared) != RENDERER_PACKAGES:
        fail("acceptance evidence must declare KDV, KRR, and KUC")
    for name, package in packages.items():
        record = declared[name]
        if not isinstance(record, dict) or record.get("version") != package["version"] or record.get("source") != CRATES_IO_SOURCE:
            fail(f"published dependency evidence does not match Cargo.lock for {name}")
    verify_html(evidence.get("html"))
    verify_packaged(evidence.get("packaged_targets"))
    verify_office(evidence.get("office_fixtures"))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--evidence", type=Path)
    args = parser.parse_args()
    try:
        verify(args.root.resolve(), args.evidence.resolve() if args.evidence else None)
    except (AcceptanceEvidenceError, OSError) as error:
        print(f"ERROR: document acceptance evidence: {error}", file=sys.stderr)
        return 1
    print("OK: v0.22.42 document acceptance evidence is current and complete.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
