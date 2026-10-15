#!/usr/bin/env python3
"""Publish collect-only inventory differences, never skip existing checks."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import sys

from native_test_run_comparison import compare_runs

MAX_MANIFEST_BYTES = 16 * 1024 * 1024
COVERAGE_PHASES = ("workspace-lib-bin", "workspace-integration", "ui-parallel", "ui-serial")
PHASE_SELECTORS = (
    ["--workspace", "--lib", "--bins"],
    ["--workspace", "--test", "*", "--exclude", "katana-ui"],
    ["-p", "katana-ui", "--test", "ui_integration_parallel"],
    ["-p", "katana-ui", "--test", "ui_integration_serial"],
)


def _unique_object(pairs: list[tuple]) -> dict:
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("inventory JSON contains duplicate object keys")
        result[key] = value
    return result


def _invalid_number(value: str) -> None:
    raise ValueError("inventory JSON contains a non-finite number")


def _load(path: Path, identities: dict) -> dict | None:
    if not path.exists():
        return None
    if path.is_symlink() or not path.is_file():
        raise ValueError("inventory input is not a regular non-symlink file")
    with path.open("rb") as stream:
        raw = stream.read(MAX_MANIFEST_BYTES + 1)
    if len(raw) > MAX_MANIFEST_BYTES:
        raise ValueError("inventory manifest exceeds the diagnostic byte limit")
    value = json.loads(raw.decode("utf-8"), object_pairs_hook=_unique_object,
                       parse_constant=_invalid_number)
    if not isinstance(value, dict):
        raise ValueError("inventory manifest must be an object")
    identities[str(path)] = hashlib.sha256(raw).hexdigest()
    return value


def _cargo_selection(arguments: list[str]) -> list[str]:
    selected = []
    iterator = iter(arguments)
    for argument in iterator:
        if argument in {"--jobs", "-j"}:
            value = next(iterator, None)
            if value is None or not value.isascii() or not value.isdigit() or int(value) <= 0:
                raise ValueError("coverage jobs value is invalid")
        elif argument not in {"--no-report", "--no-cfg-coverage", "-q", "--locked", "--offline"}:
            selected.append(argument)
    return selected


def _phase_selected(run: dict, phase: str) -> bool:
    arguments = run.get("requested_argv")
    if not isinstance(arguments, list) or any(not isinstance(arg, str) for arg in arguments):
        return False
    boundary = arguments.index("--") if "--" in arguments else len(arguments)
    selected = _cargo_selection(arguments[2:boundary])
    if arguments[:2] != ["cargo", "llvm-cov"] or selected != PHASE_SELECTORS[COVERAGE_PHASES.index(phase)]:
        return False
    native = [arg for arg in arguments[boundary + 1:] if not arg.startswith("--test-threads=")]
    return native == (["--skip", "fixture"] if phase == "workspace-integration" else [])


def collect_comparison(directory: Path) -> dict:
    identities, coverage, missing, invalid = {}, [], [], []
    normal = _load(directory / "normal.json", identities)
    for phase in COVERAGE_PHASES:
        run = _load(directory / "coverage" / f"{phase}.json", identities)
        if run is None:
            missing.append(phase)
        else:
            coverage.append(run)
            if not _phase_selected(run, phase):
                invalid.append(phase)
    report = compare_runs(normal, coverage)
    report.update({"input_sha256": identities, "missing_coverage_phases": missing,
                   "invalid_coverage_phase_selections": invalid,
                   "coverage_phase_inputs_complete": not missing,
                   "coverage_phase_selection_verified": not missing and not invalid})
    if missing or invalid:
        report["selection_comparison_available"] = False
        for key in ("common_candidates", "normal_only_candidates", "coverage_only_candidates"):
            report[key] = []
        report["reasons"].append("fixed coverage phase inputs are missing or selectors do not match")
    return report


def _summary(report: dict) -> str:
    counts = ", ".join(f"{name}={len(report[name])}" for name in
                       ("common_candidates", "normal_only_candidates", "coverage_only_candidates"))
    return ("## Test inventory diagnostics\n\n"
            f"Candidate comparison available: {report['selection_comparison_available']}. {counts}.\n\n"
            "Execution equivalence: false. Test skip/reuse authorization: false.\n\n"
            "Reasons: " + "; ".join(report["reasons"]) + "\n")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--directory", type=Path, required=True)
    arguments = parser.parse_args()
    try:
        report = collect_comparison(arguments.directory)
        arguments.directory.mkdir(parents=True, exist_ok=True)
        output = arguments.directory / "comparison.json"
        output.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")
        summary = _summary(report)
        print(summary)
        if path := os.environ.get("GITHUB_STEP_SUMMARY"):
            with Path(path).open("a", encoding="utf-8") as stream:
                stream.write(summary)
    except (OSError, ValueError, UnicodeError, RecursionError) as error:
        print(f"inventory comparison failed: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
