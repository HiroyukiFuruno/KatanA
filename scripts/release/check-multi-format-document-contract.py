#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import re
import sys
import tempfile
import tomllib
from pathlib import Path


TARGET_VERSION = "0.22.38"
REQUIRED_DEPENDENCIES = {
    "katana-document-viewer": (0, 4, 1),
}
FORBIDDEN_DOCUMENT_MARKERS = (
    "chromium",
    "webview",
    "pdfium",
    "libreoffice",
    "onlyoffice",
    "office2pdf",
    "ironcalc",
    "hayro",
    "calamine",
    "docx_rs",
    "lopdf",
    "quick_xml",
    "zip::",
    "katana_render_runtime",
    "katana_document_viewer_kuc",
    "katana_ui_core",
    "browsersessionadapter",
)
FORBIDDEN_DIRECT_DEPENDENCIES = {
    "calamine",
    "docx-rs",
    "hayro",
    "ironcalc",
    "katana-document-viewer-kuc",
    "katana-ui-core",
    "libreoffice",
    "lopdf",
    "office2pdf",
    "onlyoffice",
    "pdfium-render",
    "quick-xml",
}
FORBIDDEN_DOCUMENT_SURFACE_FILES = (
    "grid.rs",
    "grid_cell_text.rs",
    "grid_conditional.rs",
    "grid_paint.rs",
    "page_render.rs",
)
REQUIRED_FORMATS = ("pdf", "docx", "xlsx", "pptx")


def fail(message: str) -> None:
    raise SystemExit(message)


def parse_requirement(value: object, name: str) -> tuple[int, int, int]:
    if isinstance(value, dict):
        forbidden = [key for key in ("path", "git") if key in value]
        if forbidden:
            fail(f"{name} uses forbidden dependency source: {forbidden[0]}")
        value = value.get("version")
    if not isinstance(value, str):
        fail(f"{name} must declare a crates.io version")
    match = re.fullmatch(r"\^?(\d+)\.(\d+)\.(\d+)", value)
    if match is None:
        fail(f"{name} must use one caret-compatible x.y.z requirement: {value}")
    return tuple(int(part) for part in match.groups())


def verify_registry_package(
    lock: dict[str, object], name: str, minimum: tuple[int, int, int]
) -> None:
    packages = [
        package
        for package in lock.get("package", [])
        if isinstance(package, dict) and package.get("name") == name
    ]
    if len(packages) != 1:
        fail(f"Cargo.lock must resolve exactly one {name} package; found {len(packages)}")
    package = packages[0]
    version = package.get("version")
    if not isinstance(version, str):
        fail(f"Cargo.lock {name} version is missing")
    parsed = tuple(int(part) for part in version.split("."))
    if parsed < minimum or parsed[0:2] != minimum[0:2]:
        fail(f"Cargo.lock {name} must resolve within {minimum[0]}.{minimum[1]}.x; found {version}")
    source = package.get("source")
    if not isinstance(source, str) or not source.startswith("registry+"):
        fail(f"Cargo.lock {name} must resolve from a registry; found {source!r}")


def verify_kdv_features(value: object) -> None:
    if not isinstance(value, dict):
        fail("katana-document-viewer must enable the KDV-owned egui document surface")
    features = value.get("features")
    if not isinstance(features, list) or features != ["egui"]:
        fail("katana-document-viewer must enable only the egui feature")


def verify_macos_bundle_metadata(value: object) -> None:
    if not isinstance(value, dict):
        fail("katana-ui manifest must be a TOML table")
    package = value.get("package")
    if not isinstance(package, dict):
        fail("katana-ui manifest must declare a package table")
    metadata = package.get("metadata")
    if not isinstance(metadata, dict):
        fail("katana-ui manifest must declare package metadata")
    bundle = metadata.get("bundle")
    if not isinstance(bundle, dict):
        fail("katana-ui manifest must declare bundle metadata")
    binaries = bundle.get("bin")
    if not isinstance(binaries, dict):
        fail("KatanA bundle metadata must be scoped to a binary")
    binary = binaries.get("KatanA")
    if not isinstance(binary, dict):
        fail("KatanA must use package.metadata.bundle.bin.KatanA metadata")
    if binary.get("name") != "KatanA Desktop":
        fail("KatanA macOS bundle name must be KatanA Desktop")


def require_markers(path: Path, markers: tuple[str, ...]) -> None:
    text = path.read_text(encoding="utf-8")
    for marker in markers:
        if marker not in text:
            fail(f"{path} is missing required marker: {marker}")


def require_markers_across(paths: list[Path], markers: tuple[str, ...]) -> None:
    text = "\n".join(path.read_text(encoding="utf-8") for path in paths)
    for marker in markers:
        if marker not in text:
            joined = ", ".join(str(path) for path in paths)
            fail(f"[{joined}] is missing required marker: {marker}")


def reject_markers(paths: list[Path], markers: tuple[str, ...]) -> None:
    for path in paths:
        text = path.read_text(encoding="utf-8").lower()
        for marker in markers:
            if marker in text:
                fail(f"{path} contains forbidden document ownership marker: {marker}")


def source_files(root: Path, relative: str) -> list[Path]:
    return sorted((root / relative).rglob("*.rs"))


def dependency_names(value: object) -> set[str]:
    if not isinstance(value, dict):
        return set()
    names: set[str] = set()
    for key, child in value.items():
        if key.endswith("dependencies") and isinstance(child, dict):
            names.update(name.replace("_", "-").lower() for name in child)
        names.update(dependency_names(child))
    return names


def verify_release_evaluation(value: object) -> None:
    if not isinstance(value, dict):
        fail("release evaluation must be a JSON object")
    if value.get("schema_version") != 1 or value.get("target") != f"v{TARGET_VERSION}":
        fail("release evaluation schema or target is invalid")
    minimum = value.get("minimum_engine_score")
    profiles = value.get("engine_profiles")
    if not isinstance(minimum, int) or not isinstance(profiles, dict):
        fail("release evaluation engine score contract is invalid")
    if set(profiles) != set(REQUIRED_FORMATS):
        fail("release evaluation must score exactly PDF, DOCX, XLSX, and PPTX")
    for format_name, profile in profiles.items():
        if not isinstance(profile, dict) or not isinstance(profile.get("score"), int):
            fail(f"release evaluation score is missing for {format_name}")
        if profile["score"] < minimum:
            fail(f"release evaluation score for {format_name} is below {minimum}")

    integration = value.get("integration_score")
    if not isinstance(integration, dict):
        fail("release evaluation integration score is missing")
    categories = integration.get("categories")
    if not isinstance(categories, dict) or not categories:
        fail("release evaluation categories are missing")
    achieved = sum(
        category.get("score", -1)
        for category in categories.values()
        if isinstance(category, dict)
    )
    maximum = sum(
        category.get("maximum", -1)
        for category in categories.values()
        if isinstance(category, dict)
    )
    if achieved != integration.get("achieved") or maximum != integration.get("maximum"):
        fail("release evaluation category totals do not match the declared score")
    if achieved < 95 or maximum != 100 or integration.get("release_required") != 100:
        fail("release evaluation must retain the 95-point local and 100-point release gates")


def verify(root: Path, target_version: str) -> None:
    if target_version.removeprefix("v") != TARGET_VERSION:
        fail(
            f"multi-format document contract applies only to v{TARGET_VERSION}; "
            f"received v{target_version.removeprefix('v')}"
        )
    cargo_path = root / "Cargo.toml"
    lock_path = root / "Cargo.lock"
    with cargo_path.open("rb") as handle:
        cargo = tomllib.load(handle)
    with lock_path.open("rb") as handle:
        lock = tomllib.load(handle)
    with (root / "crates/katana-ui/Cargo.toml").open("rb") as handle:
        katana_ui_cargo = tomllib.load(handle)
    with (
        root
        / "openspec/changes/v0-22-38-multi-format-document-viewer/evidence/release-evaluation.json"
    ).open(encoding="utf-8") as handle:
        verify_release_evaluation(json.load(handle))
    workspace_version = cargo.get("workspace", {}).get("package", {}).get("version")
    if workspace_version != TARGET_VERSION:
        fail(f"workspace version must be {TARGET_VERSION}; found {workspace_version}")
    dependencies = cargo.get("workspace", {}).get("dependencies", {})
    for name, expected in REQUIRED_DEPENDENCIES.items():
        actual = parse_requirement(dependencies.get(name), name)
        if actual != expected:
            fail(f"{name} must declare {expected[0]}.{expected[1]}.{expected[2]}; found {actual}")
        verify_registry_package(lock, name, expected)
    verify_kdv_features(dependencies.get("katana-document-viewer"))
    verify_macos_bundle_metadata(katana_ui_cargo)
    forbidden_direct = (
        dependency_names(cargo) | dependency_names(katana_ui_cargo)
    ) & FORBIDDEN_DIRECT_DEPENDENCIES
    if forbidden_direct:
        fail(
            "KatanA must not declare document engines directly: "
            + ", ".join(sorted(forbidden_direct))
        )

    worker = root / "crates/katana-ui/src/preview_pane/document_surface/worker.rs"
    require_markers(worker, ("DocumentRuntime::open", "runtime.apply", "runtime.frame"))
    runtime = (
        root
        / "crates/katana-ui/src/preview_pane/document_surface/document_runtime.rs"
    )
    require_markers(
        runtime,
        (
            "PdfViewerSession",
            "OfficeStaticViewerSession",
            "SpreadsheetViewerSession",
            "OfficeWorkerConfig",
        ),
    )
    require_markers(
        root / "crates/katana-ui/src/preview_pane/document_surface/paged_runtime.rs",
        ("DocumentSurfaceFrame::from_rendered_page",),
    )
    require_markers(
        root / "crates/katana-ui/src/preview_pane/document_surface/spreadsheet_runtime.rs",
        ("SpreadsheetGridSurface",),
    )
    require_markers(
        root / "crates/katana-ui/src/preview_pane/document_surface/render.rs",
        ("DocumentSurfaceHost", "output.into_commands()"),
    )
    ownership_sources = source_files(
        root, "crates/katana-ui/src/preview_pane/document_surface"
    )
    surface_root = root / "crates/katana-ui/src/preview_pane/document_surface"
    for name in FORBIDDEN_DOCUMENT_SURFACE_FILES:
        if (surface_root / name).exists():
            fail(f"KatanA must not own document surface renderer file: {name}")
    ownership_sources.extend(
        root / relative
        for relative in (
            "crates/katana-core/src/document_source.rs",
            "crates/katana-ui/src/app/action/url_source.rs",
            "crates/katana-ui/src/app/url_source/stream.rs",
            "crates/katana-ui/src/app/url_source/types.rs",
        )
    )
    reject_markers(ownership_sources, FORBIDDEN_DOCUMENT_MARKERS)
    require_markers(
        root / "crates/katana-ui/Cargo.toml",
        (
            'name = "kdv-office-worker"',
            'path = "src/bin/kdv-office-worker.rs"',
            'ehttp = { version = "0.7.1", features = ["streaming"] }',
            "[package.metadata.bundle.bin.KatanA]",
            'name = "KatanA Desktop"',
        ),
    )
    require_markers(
        root / "crates/katana-ui/src/bin/kdv-office-worker.rs",
        ("OfficeWorkerEntrypoint::run_from_env",),
    )
    require_markers(
        root / "crates/katana-core/src/update/installer.rs",
        (
            '&extracted_app_path.join("Contents/MacOS")',
            'require_extracted_sidecar(&extract_dir, "kdv-office-worker")',
            'require_extracted_sidecar(&extract_dir, "kdv-office-worker.exe")',
        ),
    )
    update_script_sources = [root / "crates/katana-core/src/update/scripts.rs"]
    update_script_sources.extend(
        source_files(root, "crates/katana-core/src/update/scripts")
    )
    require_markers_across(
        update_script_sources,
        ("TARGET_SIDECAR", "$targetSidecar", "$sidecarBak"),
    )
    require_markers(
        root / "scripts/build/package-mac.sh",
        ("--bin KatanA", "${CONTENTS}/MacOS/kdv-office-worker"),
    )
    for package_script, marker in (
        ("scripts/build/package-linux.sh", "KatanA kdv-office-worker"),
        ("scripts/build/package-windows.sh", "KatanA.exe kdv-office-worker.exe"),
        ("scripts/release/check-release-asset-contract.sh", "kdv-office-worker"),
        ("scripts/release/update-linuxbrew.sh", 'bin.install \\"kdv-office-worker\\"'),
        ("crates/katana-ui/wix/main.wxs", "kdv-office-worker.exe"),
    ):
        require_markers(root / package_script, (marker,))
    require_markers(
        root / "crates/katana-ui/src/app/action/url_source.rs",
        ("ehttp::streaming::fetch", "UrlResponseCollector"),
    )
    require_markers(
        root / "crates/katana-ui/src/preview_pane/document_surface/controls.rs",
        ("horizontal_wrapped", ".stroke("),
    )
    require_markers(
        root / "scripts/screenshot/examples/v0-22-38-multi-format-documents.json",
        (
            '"expected_document_format": "pdf"',
            '"expected_document_format": "docx"',
            '"expected_document_format": "xlsx"',
            '"expected_document_format": "pptx"',
            '"expected_document_node_kind": "Page"',
            '"expected_document_node_kind": "Grid"',
            '"open_fixture_document_url"',
            '"open_fixture_document_error_url"',
            '"Document source intake failed"',
            '"authentication or error page"',
            '"Layer: KDV worker"',
            '"encrypted or password protected"',
            '"limit is 268435456 bytes"',
            '"neither HTML nor a supported document"',
            '"Invalid URL: UnsupportedScheme"',
            '"13-pdf-direct-url-recovery"',
        ),
    )
    require_markers(
        root / ".github/workflows/test-and-build.yml",
        (
            "Run multi-format headless acceptance",
            "v0-22-38-multi-format-documents.json",
            "Upload multi-format headless evidence",
            "multi-format-headless-${{ runner.os }}",
        ),
    )
    require_markers(
        root / "scripts/release/preflight.sh",
        (
            'TASK_GATE_MODE=${KATANA_OPENSPEC_TASK_GATE:-strict}',
            '--allow 5.7',
            '--allow 7.5',
            '--allow 8.4',
            'check-openspec-task-completion.py',
        ),
    )
    require_markers(
        root / "lefthook.yml",
        ("check-pr-ready.sh --pr-bootstrap",),
    )
    require_markers(
        root / ".github/workflows/release-readiness.yml",
        ('check-pr-ready.sh "$version" --pr-bootstrap',),
    )
    print("OK: KatanA multi-format document ownership and release contract is satisfied.")


def self_test() -> None:
    assert parse_requirement("0.4.1", "kdv") == (0, 4, 1)
    assert parse_requirement({"version": "^0.4.1"}, "kdv") == (0, 4, 1)
    verify_kdv_features({"version": "0.4.1", "features": ["egui"]})
    verify_macos_bundle_metadata(
        {
            "package": {
                "metadata": {
                    "bundle": {"bin": {"KatanA": {"name": "KatanA Desktop"}}}
                }
            }
        }
    )
    for invalid in (
        {"path": "../kdv", "version": "0.4.1"},
        {"git": "https://example.test/kdv", "version": "0.4.1"},
        "0.4",
    ):
        try:
            parse_requirement(invalid, "kdv")
        except SystemExit:
            continue
        raise AssertionError(f"forbidden dependency requirement was accepted: {invalid!r}")
    assert dependency_names(
        {
            "workspace": {"dependencies": {"katana-document-viewer": "0.4.1"}},
            "target": {"cfg(unix)": {"build-dependencies": {"office2pdf": "0.6"}}},
        }
    ) == {"katana-document-viewer", "office2pdf"}
    verify_release_evaluation(
        {
            "schema_version": 1,
            "target": "v0.22.38",
            "minimum_engine_score": 80,
            "engine_profiles": {
                format_name: {"score": 80} for format_name in REQUIRED_FORMATS
            },
            "integration_score": {
                "maximum": 100,
                "release_required": 100,
                "achieved": 95,
                "categories": {
                    "verified": {"score": 95, "maximum": 100}
                },
            },
        }
    )
    with tempfile.TemporaryDirectory() as directory:
        first = Path(directory) / "first.rs"
        second = Path(directory) / "second.rs"
        first.write_text("TARGET_SIDECAR", encoding="utf-8")
        second.write_text("$targetSidecar $sidecarBak", encoding="utf-8")
        require_markers_across(
            [first, second], ("TARGET_SIDECAR", "$targetSidecar", "$sidecarBak")
        )
    print("OK: multi-format document contract self-test passed.")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("version", nargs="?", default=TARGET_VERSION)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return
    verify(args.root.resolve(), args.version)


if __name__ == "__main__":
    main()
