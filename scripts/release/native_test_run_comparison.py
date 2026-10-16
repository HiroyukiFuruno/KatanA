"""Compare candidate case sets and differences without claiming equivalent execution."""

from native_test_run_observation import observe_run

LIMITATIONS = (
    "doctest inventory not captured", "author-writable diagnostic, not CI proof",
    "native binary digest measured at listing, not at test execution",
    "environment capture is selected inputs, not a hermetic environment",
)


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
            "artifact_differences": artifacts}


def _same_source(normal: dict, coverage: dict, reasons: list[str]) -> bool:
    if normal["source_before"] == coverage["source_before"]:
        return True
    reasons.append("normal and coverage source identities differ")
    return False


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
    return _report(available, expected, observed, phases, coverage, reasons)


def _report(available: bool, normal: set, coverage: set, phases: list,
            runs: list[dict], reasons: list[str]) -> dict:
    return {"schema": 1, "diagnostic_only": True, "execution_equivalent": False,
            "reuse_authorized": False, "selection_comparison_available": available,
            "coverage_runs": [run.get("run_id") for run in runs if isinstance(run, dict)],
            "common_candidates": sorted(normal & coverage) if available else [],
            "normal_only_candidates": sorted(normal - coverage) if available else [],
            "coverage_only_candidates": sorted(coverage - normal) if available else [],
            "condition_differences": [{"run_id": phase["run_id"], "paths": phase["condition_paths"]}
                                      for phase in phases if phase["condition_paths"]],
            "artifact_differences": [{"run_id": phase["run_id"], "changes": phase["artifact_differences"]}
                                     for phase in phases if phase["artifact_differences"]],
            "reasons": sorted(set(reasons))}
