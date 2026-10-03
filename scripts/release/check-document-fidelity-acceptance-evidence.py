#!/usr/bin/env python3
"""Validate machine-readable v0.22.42 document acceptance evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import ntpath
import os
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
MAX_NORMAL_CLOSE_MS = 5000
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


def git_environment() -> dict[str, str]:
    local_names = subprocess.run(
        ["git", "rev-parse", "--local-env-vars"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.splitlines()
    return {
        key: value
        for key, value in os.environ.items()
        if key not in local_names
        and not key.startswith(("GIT_CONFIG_KEY_", "GIT_CONFIG_VALUE_"))
    }


def source_paths(root: Path) -> list[Path]:
    relative_paths: list[str] = []
    try:
        result = subprocess.run(
            ["git", "-C", str(root), "ls-files", "--cached", "--others", "--exclude-standard", "-z", "--", *SOURCE_ROOTS],
            check=False,
            capture_output=True,
            env=git_environment(),
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


def require_viewport(value: object, name: str) -> dict[str, float]:
    if not isinstance(value, dict) or set(value) != {"width", "height"}:
        fail(f"{name} must declare width and height")
    width = require_positive_finite_number(value.get("width"), f"{name}.width")
    height = require_positive_finite_number(value.get("height"), f"{name}.height")
    return {"width": width, "height": height}


def require_rect(value: object, name: str) -> dict[str, float]:
    if not isinstance(value, dict) or set(value) != {"x", "y", "width", "height"}:
        fail(f"{name} must contain x, y, width, and height")
    rect = {}
    for field in ("x", "y"):
        coordinate = value.get(field)
        if not isinstance(coordinate, (int, float)) or isinstance(coordinate, bool) or not math.isfinite(coordinate):
            fail(f"{name}.{field} must be a finite number")
        rect[field] = float(coordinate)
    for field in ("width", "height"):
        rect[field] = require_positive_finite_number(value.get(field), f"{name}.{field}")
    if rect["width"] <= 0 or rect["height"] <= 0:
        fail(f"{name}.width and height must be positive")
    return rect


def require_same_rect(actual: dict[str, float], expected: dict[str, float], name: str) -> None:
    if actual != expected:
        fail(f"{name} does not match the contract reference rectangle")


def require_tracked_contract(root: Path, path: Path, name: str) -> None:
    current = root
    for part in path.parts:
        current = current / part
        if current.is_symlink():
            fail(f"{name}.contract must not use symbolic links")
    result = subprocess.run(
        ["git", "-C", str(root), "ls-files", "--error-unmatch", "--", path.as_posix()],
        check=False,
        capture_output=True,
        env=git_environment(),
    )
    if result.returncode != 0:
        fail(f"{name}.contract must be a versioned reference contract")


def load_contract(root: Path, value: object, name: str) -> dict[str, Any]:
    if not isinstance(value, str) or not value:
        fail(f"{name}.contract must be a relative contract path")
    contract_path = Path(value)
    if contract_path.is_absolute() or ".." in contract_path.parts or contract_path.suffix != ".json":
        fail(f"{name}.contract must be a normalized JSON path without traversal")
    contracts_root = (root / "scripts" / "release" / "document-fidelity-contracts").resolve()
    if not contracts_root.is_relative_to(root.resolve()):
        fail("contract directory escapes the repository root")
    path = root / contract_path
    try:
        resolved = path.resolve(strict=True)
    except OSError as error:
        fail(f"{name}.contract cannot be read: {error}")
    if not resolved.is_relative_to(contracts_root):
        fail(f"{name}.contract must stay inside the contract directory")
    require_tracked_contract(root, contract_path, name)
    try:
        raw = resolved.read_bytes()
        contract = json.loads(raw)
    except (OSError, ValueError) as error:
        fail(f"{name}.contract is invalid: {error}")
    if not isinstance(contract, dict):
        fail(f"{name}.contract must contain an object")
    contract["_raw_sha256"] = sha256_bytes(raw)
    return contract


def load_artifact_bytes(root: Path, value: object, name: str) -> bytes:
    if not isinstance(value, str) or not value:
        fail(f"{name} artifact path is required")
    path = Path(value)
    if path.is_absolute() or ntpath.isabs(value) or ".." in path.parts or path.as_posix() != value:
        fail(f"{name} artifact path must be relative without traversal")
    try:
        resolved = (root / path).resolve(strict=True)
    except (OSError, RuntimeError) as error:
        fail(f"{name} artifact cannot be read: {error}")
    if not resolved.is_relative_to(root.resolve()):
        fail(f"{name} artifact path escapes the repository")
    if not resolved.is_file():
        fail(f"{name} artifact must be a regular file")
    current = root
    for part in path.parts:
        current /= part
        if current.is_symlink():
            fail(f"{name} artifact path must not use symbolic links")
    try:
        raw = resolved.read_bytes()
    except OSError as error:
        fail(f"{name} artifact cannot be read: {error}")
    return raw


def load_artifact(root: Path, value: object, name: str) -> tuple[bytes, dict[str, Any]]:
    raw = load_artifact_bytes(root, value, name)
    try:
        artifact = json.loads(raw)
    except ValueError as error:
        fail(f"{name} artifact is not valid JSON: {error}")
    if not isinstance(artifact, dict):
        fail(f"{name} artifact must contain an object")
    return raw, artifact


def verify_render_artifact(root: Path, manifest: dict[str, Any], name: str) -> None:
    render = manifest.get("render")
    if not isinstance(render, dict) or set(render) != {"path", "sha256", "pixel_ratio"}:
        fail(f"{name}.render must bind a real PNG output")
    raw = load_artifact_bytes(root, render["path"], f"{name}.render")
    if not raw.startswith(b"\x89PNG\r\n\x1a\n") or len(raw) < 33 or raw[12:16] != b"IHDR":
        fail(f"{name}.render must be a PNG, not a metrics JSON")
    viewport = require_viewport(manifest.get("viewport"), f"{name}.viewport")
    ratio = require_positive_finite_number(render["pixel_ratio"], f"{name}.render.pixel_ratio")
    dimensions = (math.ceil(viewport["width"] * ratio), math.ceil(viewport["height"] * ratio))
    if (int.from_bytes(raw[16:20], "big"), int.from_bytes(raw[20:24], "big")) != dimensions:
        fail(f"{name}.render dimensions do not match the measured viewport")
    digest = require_sha256(render["sha256"], f"{name}.render.sha256")
    if sha256_bytes(raw) != digest:
        fail(f"{name}.render SHA-256 is stale or forged")


def verify_render_run(manifest: dict[str, Any], comparison: dict[str, Any], run: dict[str, Any], target: str, name: str) -> None:
    identity = manifest.get("run_identity")
    expected = {field: run.get(field) for field in ("run_id", "main_sha256", "sidecar_sha256", "fixture_sha256")}
    expected["target"] = target
    if not isinstance(expected["run_id"], str) or not expected["run_id"] or identity != expected:
        fail(f"{name} measured artifact must bind its packaged run identity")
    output = run.get("render_output")
    expected_output = dict(manifest["render"], metrics_path=comparison.get("measured_artifact"), metrics_sha256=comparison.get("measured_sha256"))
    if output != expected_output:
        fail(f"{name} packaged run output does not match the measured artifact")


def verify_contract_identity(root: Path, contract: dict[str, Any], comparison: dict[str, Any], name: str, input_sha: str, renderer: str, run: dict[str, Any], target: str) -> tuple[dict[str, float], dict[str, Any], dict[str, Any]]:
    if require_sha256(comparison.get("contract_sha256"), f"{name}.contract_sha256") != contract["_raw_sha256"]:
        fail(f"{name} contract SHA-256 is stale")
    if require_sha256(contract.get("input_sha256"), f"{name}.contract.input_sha256") != input_sha:
        fail(f"{name} contract input identity does not match the fixture")
    if contract.get("reference_renderer") != renderer:
        fail(f"{name} contract renderer is invalid")
    reference_raw, reference_artifact = load_artifact(root, contract.get("reference_artifact"), f"{name}.reference")
    reference_sha = sha256_bytes(reference_raw)
    if reference_sha != require_sha256(contract.get("reference_sha256"), f"{name}.contract.reference_sha256"):
        fail(f"{name} reference artifact SHA-256 does not match the contract")
    if require_sha256(comparison.get("reference_sha256"), f"{name}.reference_sha256") != reference_sha:
        fail(f"{name} reference identity does not match the contract")
    measured_raw, measured_artifact = load_artifact(root, comparison.get("measured_artifact"), f"{name}.measured")
    if require_sha256(comparison.get("measured_sha256"), f"{name}.measured_sha256") != sha256_bytes(measured_raw):
        fail(f"{name} measured artifact SHA-256 is stale or forged")
    verify_render_artifact(root, reference_artifact, f"{name}.reference")
    verify_render_artifact(root, measured_artifact, f"{name}.measured")
    if reference_artifact.get("producer_mode") != renderer or measured_artifact.get("producer_mode") != "packaged_main":
        fail(f"{name} artifact producers do not match the independent reference and packaged main")
    verify_render_run(measured_artifact, comparison, run, target, name)
    if comparison.get("producer_mode") != "packaged_main":
        fail(f"{name} measured receipt must come from packaged_main")
    viewports = comparison.get("viewports")
    if not isinstance(viewports, dict) or set(viewports) != {"reference", "measured"}:
        fail(f"{name}.viewports must contain reference and measured")
    expected_viewport = require_viewport(contract.get("viewport"), f"{name}.contract.viewport")
    for side in ("reference", "measured"):
        if require_viewport(viewports.get(side), f"{name}.viewports.{side}") != expected_viewport:
            fail(f"{name}.viewports.{side} does not match the contract")
    for artifact in (reference_artifact, measured_artifact):
        if require_viewport(artifact.get("viewport"), f"{name}.artifact.viewport") != expected_viewport:
            fail(f"{name} artifact viewport does not match the contract")
    if reference_artifact.get("input_sha256") != input_sha or measured_artifact.get("input_sha256") != input_sha:
        fail(f"{name} artifact input identity does not match the fixture")
    return expected_viewport, reference_artifact, measured_artifact


def verify_comparison_geometry(comparison: dict[str, Any], contract: dict[str, Any], names: set[str], name: str, reference_artifact: dict[str, Any], measured_artifact: dict[str, Any]) -> None:
    geometry = comparison.get("geometry")
    contract_geometry = contract.get("geometry")
    if not isinstance(geometry, dict) or set(geometry) != names:
        fail(f"{name}.geometry must contain every required target exactly once")
    if not isinstance(contract_geometry, dict) or not contract_geometry or set(contract_geometry) != names:
        fail(f"{name}.contract.geometry must contain every required target exactly once")
    for target in names:
        receipt = geometry[target]
        expected = contract_geometry[target]
        if not isinstance(receipt, dict) or set(receipt) != {"reference", "measured"}:
            fail(f"{name}.geometry.{target} must contain reference and measured only")
        if not isinstance(expected, dict) or set(expected) != {"reference", "tolerance"}:
            fail(f"{name}.contract.geometry.{target} is invalid")
        reference = require_rect(receipt["reference"], f"{name}.geometry.{target}.reference")
        measured = require_rect(receipt["measured"], f"{name}.geometry.{target}.measured")
        contract_reference = require_rect(expected["reference"], f"{name}.contract.geometry.{target}.reference")
        require_same_rect(reference, contract_reference, f"{name}.geometry.{target}.reference")
        reference_geometry = reference_artifact.get("geometry")
        measured_geometry = measured_artifact.get("geometry")
        if not isinstance(reference_geometry, dict) or not isinstance(measured_geometry, dict):
            fail(f"{name} artifact geometry must be an object")
        artifact_reference = reference_geometry.get(target)
        artifact_measured = measured_geometry.get(target)
        require_same_rect(reference, require_rect(artifact_reference, f"{name}.reference_artifact.geometry.{target}"), f"{name}.geometry.{target}.reference")
        require_same_rect(measured, require_rect(artifact_measured, f"{name}.measured_artifact.geometry.{target}"), f"{name}.geometry.{target}.measured")
        tolerance = require_finite_number(expected["tolerance"], f"{name}.contract.geometry.{target}.tolerance")
        delta = max(abs(measured[field] - reference[field]) for field in ("x", "y", "width", "height"))
        if delta > tolerance:
            fail(f"{name}.geometry.{target} exceeds the contract tolerance")


def verify_html_comparison(root: Path, value: dict[str, Any]) -> None:
    input_sha = require_sha256(value.get("fixture_sha256"), "HTML.fixture_sha256")
    comparison = value.get("comparison")
    if not isinstance(comparison, dict):
        fail("HTML comparison is missing")
    verify_html_navigation(comparison.get("navigation"))
    contract = load_contract(root, comparison.get("contract"), "HTML")
    target = value.get("packaged_target")
    if not isinstance(target, str) or target not in SUPPORTED_TARGETS:
        fail("HTML must identify its packaged target")
    run = value.get("packaged_run")
    verify_packaged_record(target, run)
    if run.get("fixture_sha256") != input_sha:
        fail("HTML packaged run input does not match")
    viewport, reference_artifact, measured_artifact = verify_contract_identity(root, contract, comparison, "HTML", input_sha, "chromeHTML", run, target)
    if viewport != {"width": 1280, "height": 900}:
        fail("HTML comparison must use the agreed 1280x900 viewport")
    input_record = comparison.get("input")
    if not isinstance(input_record, dict) or input_record.get("anchor") != "#s15":
        fail("HTML.comparison.input must bind the #s15 anchor")
    if require_sha256(input_record.get("fixture_sha256"), "HTML.comparison.input.fixture_sha256") != input_sha:
        fail("HTML.comparison input fixture identity does not match HTML fixture")
    verify_comparison_geometry(comparison, contract, {"sticky_toc", "main", "visible_section"}, "HTML", reference_artifact, measured_artifact)
    if measured_artifact.get("navigation") != comparison.get("navigation"):
        fail("HTML navigation does not match the measured artifact")
    for field in ("active_toc", "visible_section_state"):
        state = comparison.get(field)
        if (not isinstance(state, dict) or set(state) != {"reference", "measured"}
                or not isinstance(state["reference"], str) or not state["reference"]
                or not isinstance(state["measured"], str) or not state["measured"]
                or state["reference"] != state["measured"] or state["reference"] != "#s15"):
            fail(f"HTML.comparison.{field} must match the reference state")
        if reference_artifact.get(field) != state["reference"] or measured_artifact.get(field) != state["measured"]:
            fail(f"HTML.comparison.{field} does not match the render measurements")


def verify_html_navigation(value: object) -> None:
    if not isinstance(value, dict) or set(value) != {
        "from_fragment", "to_fragment", "frame_before", "frame_after"
    }:
        fail("HTML comparison must record an actual anchor navigation")
    if value["from_fragment"] != "" or value["to_fragment"] != "#s15":
        fail("HTML comparison must navigate from the initial document to #s15")
    before = require_positive_integer(value["frame_before"], "HTML navigation frame_before")
    after = require_positive_integer(value["frame_after"], "HTML navigation frame_after")
    if after <= before:
        fail("HTML navigation must advance the rendered frame")


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


def verify_normal_close_duration(record: dict[str, Any], name: str) -> None:
    duration = require_finite_number(record.get("close_ms"), f"{name}.close_ms")
    if duration > MAX_NORMAL_CLOSE_MS:
        fail(f"{name} normal close must be within {MAX_NORMAL_CLOSE_MS} ms")


def verify_html(root: Path, value: object, packaged_targets: dict[str, Any]) -> None:
    if not isinstance(value, dict):
        fail("HTML acceptance evidence is missing")
    if value.get("fixture_sha256") != ORIGINAL_HTML_SHA256:
        fail("HTML evidence must identify the supplied original fixture")
    if value.get("runner_mode") != "packaged_main":
        fail("HTML evidence must come from packaged_main, not in_process")
    if value.get("status") != "passed" or value.get("normal_close") is not True:
        fail("HTML evidence must record a passed run with normal close")
    verify_normal_close_duration(value, "HTML")
    first_frame = require_positive_finite_number(value.get("first_frame_ms"), "HTML first_frame_ms")
    if first_frame > 60000:
        fail("HTML first frame must be within 60000 ms")
    require_finite_number(value.get("cpu_percent"), "HTML cpu_percent")
    require_positive_finite_number(value.get("rss_bytes"), "HTML rss_bytes")
    verify_html_comparison(root, value)
    target_artifact = packaged_targets[value["packaged_target"]]
    for field in ("main_path", "sidecar_path", "main_sha256", "sidecar_sha256"):
        if value["packaged_run"][field] != target_artifact[field]:
            fail(f"HTML packaged {field} does not match its target")


def verify_office_fidelity(root: Path, record: dict[str, Any], index: int, input_sha: str) -> None:
    fidelity = record.get("fidelity")
    name = f"Office fixture {index}.fidelity"
    if not isinstance(fidelity, dict):
        fail(f"{name} comparison is missing")
    contract = load_contract(root, fidelity.get("contract"), name)
    packaged_run = record.get("packaged_run")
    if not isinstance(packaged_run, dict) or fidelity.get("run_id") != packaged_run.get("run_id"):
        fail(f"{name}.run_id must bind the packaged run")
    _, reference_artifact, measured_artifact = verify_contract_identity(root, contract, fidelity, name, input_sha, "sourceOffice", packaged_run, record["packaged_target"])
    missing = fidelity.get("missing_elements")
    if not isinstance(missing, dict) or set(missing) != {"count"}:
        fail(f"{name}.missing_elements must contain count only")
    count = missing.get("count")
    if not isinstance(count, int) or isinstance(count, bool) or count < 0:
        fail(f"{name}.missing_elements.count must be a non-negative integer")
    tolerance_value = contract.get("missing_elements_tolerance")
    if not isinstance(tolerance_value, int) or isinstance(tolerance_value, bool) or tolerance_value < 0:
        fail(f"{name}.contract.missing_elements_tolerance must be a non-negative integer")
    if count > tolerance_value:
        fail(f"{name} missing element count exceeds the contract tolerance")
    if measured_artifact.get("missing_elements") != missing:
        fail(f"{name} missing elements do not match the measured artifact")
    geometry = contract.get("geometry")
    if not isinstance(geometry, dict) or not geometry:
        fail(f"{name}.contract.geometry must contain named reference elements")
    if any(not key or key != key.strip() for key in geometry):
        fail(f"{name}.contract.geometry names must be non-empty and normalized")
    geometry_names = set(geometry)
    verify_comparison_geometry(fidelity, contract, geometry_names, name, reference_artifact, measured_artifact)


def verify_packaged(value: object) -> None:
    if not isinstance(value, dict) or set(value) != SUPPORTED_TARGETS:
        fail("packaged evidence must cover every declared target")
    for target, record in value.items():
        verify_packaged_record(target, record)


def verify_packaged_record(target: str, record: object) -> None:
    if not isinstance(record, dict) or record.get("status") != "passed":
        fail(f"packaged evidence is not passed for {target}")
    if record.get("runner_mode") != "packaged_main":
        fail(f"packaged evidence runner mode is invalid for {target}")
    if record.get("clean_machine") is not True or record.get("normal_close") is not True:
        fail(f"packaged evidence must be a clean-machine normal close for {target}")
    verify_normal_close_duration(record, target)
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


def verify_office(root: Path, value: object, packaged_targets: dict[str, Any]) -> None:
    if not isinstance(value, list) or len(value) < len(SUPPLIED_OFFICE_FIXTURES):
        fail("Office evidence must contain all six fixture results")
    run_ids: set[str] = set()
    for index, record in enumerate(value):
        if not isinstance(record, dict) or record.get("status") != "passed":
            fail(f"Office fixture {index} is not passed")
        format_name = record.get("format")
        if format_name not in {"docx", "xlsx", "pptx"}:
            fail(f"Office fixture {index} has an invalid format")
        input_sha = require_sha256(record.get("input_sha256"), f"Office fixture {index}.input_sha256")
        first_frame = require_positive_finite_number(record.get("first_frame_ms"), f"Office fixture {index}.first_frame_ms")
        if first_frame > 15000:
            fail(f"Office fixture {index} first frame must be within 15000 ms")
        require_positive_integer(record.get("item_count"), f"Office fixture {index}.item_count")
        if record.get("release_worker") is not True:
            fail(f"Office fixture {index} must use the release worker")
        target = record.get("packaged_target")
        if not isinstance(target, str) or target not in SUPPORTED_TARGETS:
            fail(f"Office fixture {index} must identify its packaged target")
        run = record.get("packaged_run")
        verify_packaged_record(target, run)
        run_id = run.get("run_id")
        if not isinstance(run_id, str) or not run_id or run_id != run_id.strip() or run_id in run_ids:
            fail(f"Office fixture {index} must have a distinct packaged run ID")
        run_ids.add(run_id)
        if require_sha256(run.get("fixture_sha256"), f"Office fixture {index}.packaged_run.fixture_sha256") != input_sha:
            fail(f"Office fixture {index} packaged run input does not match")
        artifact = packaged_targets[target]
        for field in ("main_path", "sidecar_path"):
            if run[field] != artifact[field]:
                fail(f"Office fixture {index} packaged {field} does not match its target")
        for field in ("main_sha256", "sidecar_sha256"):
            if run[field].lower() != artifact[field].lower():
                fail(f"Office fixture {index} packaged {field} does not match its target")
        cold_rss = require_positive_integer(run.get("cold_rss_bytes"), f"Office fixture {index}.cold_rss_bytes")
        after_close_rss = require_positive_integer(run.get("after_close_rss_bytes"), f"Office fixture {index}.after_close_rss_bytes")
        if after_close_rss - cold_rss > 196608 * 1024:
            fail(f"Office fixture {index} close RSS increase exceeds 196608 KiB")
        if input_sha in SUPPLIED_OFFICE_FIXTURES and format_name != SUPPLIED_OFFICE_FIXTURES[input_sha]:
            fail(f"Office fixture {index} format does not match its supplied input hash")
        verify_office_fidelity(root, record, index, input_sha)
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
    verify_packaged(evidence.get("packaged_targets"))
    verify_html(root, evidence.get("html"), evidence["packaged_targets"])
    verify_office(root, evidence.get("office_fixtures"), evidence["packaged_targets"])


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
