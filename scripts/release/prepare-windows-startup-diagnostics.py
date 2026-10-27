#!/usr/bin/env python3
"""診断結果を必須smokeへ混同せず、検証済み本文から限定派生scriptを作る。"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path
from windows_dx12_compiler_case_metadata import compiler_metadata


RUN_LOG_LINE = (
    '$env:RUST_LOG = "warn,katana_ui=info,katana_core=info,KatanA=info,'
    'eframe::native::run=trace,eframe::native::wgpu_integration=trace,'
    'egui_wgpu=debug,winit::platform_impl::windows::event_loop=trace"\n'
)
REDUCED_LOG_LINE = (
    '$env:RUST_LOG = "warn,katana_ui=info,katana_core=info,KatanA=info,'
    'eframe::native::run=info,eframe::native::wgpu_integration=info,'
    'egui_wgpu=info,winit::platform_impl::windows::event_loop=info"\n'
)
SMOKE_DIRECTORY_LINE = '$directory = Join-Path $env:RUNNER_TEMP "katana-windows-smoke"\n'
ISOLATED_DIRECTORY_LINE = (
    '$directory = Join-Path $env:RUNNER_TEMP '
    '("katana-windows-smoke-" + $env:KATANA_STARTUP_DIAGNOSTICS_CASE)\n'
)
DEBUG_LINE = '$env:DEBUG = "true"\n'
COMPILER_CASE = "dx12-compiler"
COMPILER_LABEL = "dx12-dxc"
DX12_ENVIRONMENT_LINE = '$env:WGPU_DX12_USE_FRAME_LATENCY_WAITABLE_OBJECT = "{value}"\n'
PRESENTATION_ENVIRONMENT_LINE = '$env:WGPU_DX12_PRESENTATION_SYSTEM = "{value}"\n'
SUPPORTED_CASES = ("original", "reduced-native-trace", "dx12-waiting", "dx12-presentation", COMPILER_CASE)
SUPPORTED_DX12_VALUES = ("DontWait", "None")
SUPPORTED_PRESENTATION_SYSTEMS = ("DxgiFromVisual",)
SUPPORTED_RUN_LABELS = (
    "original", "reduced-native-trace", "original-before", "dx12-dont-wait",
    "dx12-none", "original-after", "dx12-visual", COMPILER_LABEL,
)
CASE_LABELS = {
    "original": ("original", "original-before", "original-after"),
    "reduced-native-trace": ("reduced-native-trace",),
    "dx12-waiting": ("dx12-dont-wait", "dx12-none"),
    "dx12-presentation": ("original-before", "dx12-visual", "original-after"),
    COMPILER_CASE: ("original-before", COMPILER_LABEL, "original-after"),
}

class PreparationError(ValueError):
    """検証済み本文の想定外変更を診断へ持ち込まないための例外。"""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise PreparationError(message)


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def read_json(path: Path) -> dict[str, object]:
    require(path.is_file() and not path.is_symlink(), "metadata must be a regular file")
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise PreparationError(f"invalid preparation metadata: {error}") from error
    require(isinstance(value, dict), "preparation metadata must be an object")
    return value


def replace_once(source: str, old: str, new: str, label: str) -> str:
    require(source.count(old) == 1, f"trusted smoke must contain exactly one {label} line")
    return source.replace(old, new, 1)


def diagnostic_variant(source: str, case: str, dx12_value: str | None = None,
                       presentation_system: str | None = None) -> str:
    require(case in SUPPORTED_CASES, "unsupported diagnostic case")
    result = replace_once(source, SMOKE_DIRECTORY_LINE, ISOLATED_DIRECTORY_LINE, "smoke directory")
    if case == "reduced-native-trace":
        result = replace_once(result, RUN_LOG_LINE, REDUCED_LOG_LINE, "native trace RUST_LOG")
    if case == "dx12-waiting":
        require(dx12_value in SUPPORTED_DX12_VALUES, "unsupported DX12 waitable-object value")
        result = replace_once(result, DEBUG_LINE, DEBUG_LINE + DX12_ENVIRONMENT_LINE.format(value=dx12_value),
                              "debug environment")
    else:
        require(dx12_value is None, "DX12 environment value is only valid for dx12-waiting")
    if case == "dx12-presentation":
        require(presentation_system in SUPPORTED_PRESENTATION_SYSTEMS,
                "unsupported DX12 presentation system")
        result = replace_once(result, DEBUG_LINE,
                              DEBUG_LINE + PRESENTATION_ENVIRONMENT_LINE.format(value=presentation_system),
                              "debug environment")
    else:
        require(presentation_system is None,
                "DX12 presentation system is only valid for dx12-presentation")
    return result


def resolve_paths(args: argparse.Namespace) -> tuple[Path, Path, Path, Path]:
    source_input = Path(args.source_script)
    metadata_input = Path(args.source_metadata)
    source_path = source_input.resolve()
    metadata_path = metadata_input.resolve()
    output_input = Path(args.output_script)
    output_metadata_input = Path(args.output_metadata)
    output_script = output_input.resolve()
    output_metadata = output_metadata_input.resolve()
    require(source_input.is_file() and not source_input.is_symlink(), "source script must be a regular file")
    require(metadata_input.is_file() and not metadata_input.is_symlink(),
            "source metadata must be a regular file")
    require(output_script not in (source_path, metadata_path), "output script must not overwrite an input")
    require(output_metadata not in (source_path, metadata_path, output_script),
            "output metadata must not overwrite an input or script")
    require(not output_input.exists() and not output_input.is_symlink(), "output script already exists")
    require(not output_metadata_input.exists() and not output_metadata_input.is_symlink(),
            "output metadata already exists")
    return source_path, metadata_path, output_script, output_metadata


def validate_case_arguments(args: argparse.Namespace) -> str | None:
    require(args.run_label in SUPPORTED_RUN_LABELS, "unsupported diagnostic run label")
    require(args.run_label in CASE_LABELS[args.case], "run label does not match diagnostic case")
    presentation_system = getattr(args, "presentation_system", None)
    require((args.case == "dx12-waiting") == (args.dx12_value is not None),
            "DX12 environment value must match the diagnostic case")
    require((args.case == "dx12-presentation") == (presentation_system is not None),
            "DX12 presentation system must match the diagnostic case")
    expected_value = {"dx12-dont-wait": "DontWait", "dx12-none": "None"}.get(args.run_label)
    require(args.dx12_value == expected_value, "DX12 value does not match diagnostic run label")
    expected_system = "DxgiFromVisual" if args.run_label == "dx12-visual" else None
    require(presentation_system == expected_system,
            "DX12 presentation system does not match diagnostic run label")
    return presentation_system


def prepare(args: argparse.Namespace) -> None:
    source_path, metadata_path, output_script, output_metadata = resolve_paths(args)
    source_metadata = read_json(metadata_path)
    source = source_path.read_text(encoding="utf-8")
    require(sha256_bytes(source.encode("utf-8")) == source_metadata.get("powershell_body_sha256"),
            "source body hash differs from validated original smoke")
    require(source_metadata.get("version") == "v0.22.42", "validated source version differs")
    require(source_metadata.get("source_commit") == args.expected_source_commit,
            "validated source commit differs")
    require(source_metadata.get("github_run_id") == args.expected_run_id,
            "validated run id differs")
    require(source_metadata.get("github_run_attempt") == args.expected_run_attempt,
            "validated run attempt differs")
    presentation_system = validate_case_arguments(args)
    try:
        compiler_values = compiler_metadata(args.case, args.run_label,
                                            getattr(args, "compiler_receipt", None))
    except ValueError as error:
        raise PreparationError(str(error)) from error
    result = diagnostic_variant(source, args.case, args.dx12_value, presentation_system)
    output_script.parent.mkdir(parents=True, exist_ok=True)
    output_metadata.parent.mkdir(parents=True, exist_ok=True)
    output_script.write_text(result, encoding="utf-8", newline="")
    source_metadata.update({"diagnostic_case": args.case, "run_label": args.run_label,
                            "dx12_environment_value": args.dx12_value,
                            "dx12_presentation_system": presentation_system,
                            "diagnostic_body_sha256": sha256_bytes(result.encode("utf-8"))})
    source_metadata.update(compiler_values)
    output_metadata.write_text(json.dumps(source_metadata, indent=2) + "\n", encoding="utf-8", newline="")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-script", required=True)
    parser.add_argument("--source-metadata", required=True)
    parser.add_argument("--output-script", required=True)
    parser.add_argument("--output-metadata", required=True)
    parser.add_argument("--expected-source-commit", required=True)
    parser.add_argument("--expected-run-id", required=True)
    parser.add_argument("--expected-run-attempt", required=True)
    parser.add_argument("--case", choices=SUPPORTED_CASES, required=True)
    parser.add_argument("--run-label", choices=SUPPORTED_RUN_LABELS, required=True)
    parser.add_argument("--dx12-value", choices=SUPPORTED_DX12_VALUES)
    parser.add_argument("--presentation-system", choices=SUPPORTED_PRESENTATION_SYSTEMS)
    parser.add_argument("--compiler-receipt")
    args = parser.parse_args()
    try:
        require(re.fullmatch(r"[0-9a-f]{40}", args.expected_source_commit) is not None,
                "expected source commit must be a full commit")
        require(re.fullmatch(r"[1-9][0-9]*", args.expected_run_id) is not None,
                "expected run id must be positive decimal")
        require(re.fullmatch(r"[1-9][0-9]*", args.expected_run_attempt) is not None,
                "expected run attempt must be positive decimal")
        prepare(args)
    except (OSError, PreparationError) as error:
        print(f"ERROR: Windows startup diagnostic preparation: {error}")
        return 1
    print(f"OK: validated Windows startup diagnostic case prepared: {args.case}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
