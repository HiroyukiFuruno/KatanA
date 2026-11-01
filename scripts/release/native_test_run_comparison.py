"""Compare candidate case sets and differences without claiming equivalent execution."""

import hashlib
import json

from native_test_run_observation import observe_run

LIMITATIONS = (
    "doctest inventory not captured", "author-writable diagnostic, not CI proof",
    "native binary digest measured at listing, not at test execution",
    "environment capture is selected inputs, not a hermetic environment",
)
PHASE_METRICS_SCOPE = "unique inventory candidates only; not execution count, equivalence, or reduction"


def _changed_paths(left: object, right: object, prefix: str = "") -> list[str]:
    if isinstance(left, dict) and isinstance(right, dict):
        paths = []
        for key in sorted(set(left) | set(right)):
            path = f"{prefix}.{key}" if prefix else str(key)
            if key not in left or key not in right:
                paths.append(path)
            else:
                paths.extend(_changed_paths(left[key], right[key], path))
        return paths
    return [] if left == right and type(left) is type(right) else [prefix]


def _record(run: dict | None, kind: str, label: str, reasons: list[str]):
    if run is None:
        reasons.append(f"{label}: observed invocation is missing")
        return None
    try:
        result = observe_run(run, kind)
    except (KeyError, TypeError, ValueError, AttributeError) as error:
        reasons.append(f"{label}: comparison unavailable: {error}")
        return None
    if run["exit_code"] != 0:
        reasons.append(f"{label}: observed invocation failed with exit {run['exit_code']}")
    return result


def _differences(normal: dict, coverage: dict, left: dict, right: dict) -> tuple[list, list]:
    conditions = {key: normal.get(key) for key in
                  ("kind", "source_before", "inputs", "requested_argv", "executed_argv")}
    changed = _changed_paths(conditions, {key: coverage.get(key) for key in conditions})
    artifact_changes = [{"artifact": key, "paths": _changed_paths(left[key], right[key])}
                        for key in sorted(set(left) & set(right)) if left[key] != right[key]]
    return changed, artifact_changes


def _phase(normal: dict, coverage: dict, baseline: tuple, observed: tuple) -> dict:
    conditions, artifacts = _differences(normal, coverage, baseline[1], observed[1])
    return {"run_id": coverage["run_id"], "condition_paths": conditions,
            "artifact_differences": artifacts, "candidates": observed[0]}


def _same_source(normal: dict, coverage: dict, reasons: list[str]) -> bool:
    if normal["source_before"] == coverage["source_before"]:
        return True
    reasons.append("normal and coverage source identities differ")
    return False


def _phase_metrics(phases: list[dict], available: bool) -> tuple[list[dict] | None, list[dict] | None]:
    if not available:
        return None, None
    candidates = [{"run_id": phase["run_id"], "candidate_count": len(phase["candidates"])}
                  for phase in phases]
    overlaps = [{"left_run_id": left["run_id"], "right_run_id": right["run_id"],
                 "intersection_count": len(left["candidates"] & right["candidates"])}
                for index, left in enumerate(phases) for right in phases[index + 1:]]
    return candidates, overlaps


def _thread_arguments(argv: list[str]) -> list[str]:
    selected = []
    for index, value in enumerate(argv):
        if value == "--test-threads":
            selected.extend(argv[index:index + 2])
        elif value.startswith("--test-threads="):
            selected.append(value)
    return selected


def _dimensions(normal: dict | None, coverage: list[dict], phases: list[dict], available: bool) -> dict:
    result = {
        "doctest": {"status": "unknown", "evidence": "doctest execution inventory is not captured"},
        "ignored": {"status": "unknown", "evidence": "runtime ignored-test execution is not captured"},
        "fixture": {"status": "unknown", "evidence": "fixture execution is not completely captured"},
        "serial": {"status": "unknown", "evidence": "complete runtime scheduling is not captured"},
        "environment": {"status": "unknown", "evidence": "only selected environment inputs are captured"},
    }
    if not available or not isinstance(normal, dict):
        return result
    normal_argv = normal.get("executed_argv", [])
    normal_env = normal.get("inputs", {}).get("environment", {})
    for run in coverage:
        argv = run.get("executed_argv", [])
        env = run.get("inputs", {}).get("environment", {})
        if normal_env != env:
            result["environment"] = {"status": "different", "evidence": "captured environment values differ"}
        if (normal_env.get("RUST_TEST_THREADS") != env.get("RUST_TEST_THREADS")
                or _thread_arguments(normal_argv) != _thread_arguments(argv)):
            result["serial"] = {"status": "different", "evidence": "captured test thread settings differ"}
        if ([x for x in normal_argv if "fixture" in x]
                != [x for x in argv if "fixture" in x]):
            result["fixture"] = {"status": "different", "evidence": "captured fixture selection differs"}
        if ([x for x in normal_argv if x in ("--ignored", "--include-ignored")]
                != [x for x in argv if x in ("--ignored", "--include-ignored")]):
            result["ignored"] = {"status": "different", "evidence": "captured ignored-test selection differs"}
    if any(any(any(path == "ignored" or path.startswith("ignored.") for path in item["paths"])
                   for item in phase["artifact_differences"]) for phase in phases):
        result["ignored"] = {"status": "different", "evidence": "captured native ignored listings differ"}
    return result


def compare_runs(normal: dict | None, coverage: list[dict]) -> dict:
    reasons = list(LIMITATIONS)
    baseline = _record(normal, "test", "normal", reasons)
    observed, available, phases = set(), baseline is not None and bool(coverage), []
    normal_id = normal.get("run_id") if isinstance(normal, dict) else None
    run_ids = {normal_id} if isinstance(normal_id, str) else set()
    for index, run in enumerate(coverage):
        run_id = run.get("run_id") if isinstance(run, dict) else None
        if isinstance(run_id, str) and run_id in run_ids:
            reasons.append("duplicate observed invocation identity")
            available = False
            continue
        if isinstance(run_id, str):
            run_ids.add(run_id)
        result = _record(run, "coverage", f"coverage[{index}]", reasons)
        if result is None:
            available = False
            continue
        observed.update(result[0])
        if baseline is not None:
            phases.append(_phase(normal, run, baseline, result))
            available = _same_source(normal, run, reasons) and available
    if not coverage:
        reasons.append("coverage: observed invocations are missing")
    expected = baseline[0] if baseline is not None else set()
    phase_candidates, phase_overlaps = _phase_metrics(phases, available)
    report = _report(available, expected, observed, phases, coverage, reasons,
                     phase_candidates, phase_overlaps)
    try:
        encoded = json.dumps({"normal": normal, "coverage": coverage}, sort_keys=True,
                             separators=(",", ":"), allow_nan=False).encode()
        report["inputs_sha256"] = hashlib.sha256(encoded).hexdigest()
    except (TypeError, ValueError):
        report["inputs_sha256"] = None
        report["selection_comparison_available"] = False
        report["reasons"].append("input digest unavailable")
    report["inventory_complete"] = False
    report["dimensions"] = _dimensions(normal, coverage, phases, available)
    return report


def _report(available: bool, normal: set, coverage: set, phases: list,
            runs: list[dict], reasons: list[str], phase_candidates: list[dict] | None,
            phase_overlaps: list[dict] | None) -> dict:
    return {"schema": 1, "diagnostic_only": True, "execution_equivalent": False,
            "reuse_authorized": False, "selection_comparison_available": available,
            "coverage_runs": [run.get("run_id") for run in runs if isinstance(run, dict)],
            "coverage_phase_candidates": phase_candidates,
            "coverage_phase_overlaps": phase_overlaps,
            "coverage_phase_metrics_scope": PHASE_METRICS_SCOPE,
            "common_candidates": sorted(normal & coverage) if available else [],
            "normal_only_candidates": sorted(normal - coverage) if available else [],
            "coverage_only_candidates": sorted(coverage - normal) if available else [],
            "condition_differences": [{"run_id": phase["run_id"], "paths": phase["condition_paths"]}
                                      for phase in phases if phase["condition_paths"]],
            "artifact_differences": [{"run_id": phase["run_id"], "changes": phase["artifact_differences"]}
                                     for phase in phases if phase["artifact_differences"]],
            "reasons": sorted(set(reasons))}
