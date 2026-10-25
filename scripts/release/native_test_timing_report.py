"""Expose measured invocation time without inferring equivalence or savings."""

from datetime import datetime, timedelta
import math


def _duration(value: object) -> float:
    if type(value) not in (int, float) or not math.isfinite(value) or value < 0:
        raise ValueError("duration is not a finite nonnegative number")
    return float(value)


def _utc(value: object) -> str:
    if not isinstance(value, str):
        raise ValueError("measurement timestamp is missing")
    parsed = datetime.fromisoformat(value)
    if parsed.utcoffset() != timedelta(0):
        raise ValueError("measurement timestamp is not UTC")
    return value


def _observation(run: dict) -> dict:
    timing = run["invocation_timing"]
    if (not isinstance(timing, dict) or set(timing) != {"schema", "clock", "command", "diagnostics"}
            or type(timing["schema"]) is not int or timing["schema"] != 1
            or timing["clock"] != "perf_counter"):
        raise ValueError("timing schema or clock is unsupported")
    command, diagnostics = timing["command"], timing["diagnostics"]
    if (not isinstance(command, dict)
            or set(command) != {"started_at", "finished_at", "elapsed_seconds"}
            or not isinstance(diagnostics, dict) or set(diagnostics) != {"elapsed_seconds"}
            or not isinstance(run.get("run_id"), str) or not run["run_id"]
            or type(run.get("exit_code")) is not int):
        raise ValueError("timing identity, status or measurement is incomplete")
    return {"run_id": run["run_id"], "exit_code": run["exit_code"],
            "started_at": _utc(command["started_at"]), "finished_at": _utc(command["finished_at"]),
            "command_seconds": _duration(command["elapsed_seconds"]),
            "diagnostics_seconds": _duration(diagnostics["elapsed_seconds"])}


def _record(run: dict | None, label: str, reasons: list[str]) -> dict | None:
    try:
        if run is None:
            raise ValueError("invocation is missing")
        return _observation(run)
    except (KeyError, TypeError, ValueError, OverflowError) as error:
        reasons.append(f"{label}: timing not measured or invalid: {error}")
        return None


def _coverage_totals(records: list, complete: bool, reasons: list[str]) -> tuple:
    if not complete or not records or any(record is None for record in records):
        return None, None
    if len({record["run_id"] for record in records}) != len(records):
        reasons.append("duplicate coverage timing invocation")
        return None, None
    command = sum(record["command_seconds"] for record in records)
    diagnostics = sum(record["diagnostics_seconds"] for record in records)
    if not math.isfinite(command) or not math.isfinite(diagnostics):
        reasons.append("coverage timing total is non-finite")
        return None, None
    return command, diagnostics


def timing_report(normal: dict | None, coverage: list[dict], phases_complete: bool) -> dict:
    reasons = []
    baseline = _record(normal, "normal", reasons)
    phases = [_record(run, f"coverage[{index}]", reasons) for index, run in enumerate(coverage)]
    command, diagnostics = _coverage_totals(phases, phases_complete, reasons)
    return {"schema": 1, "diagnostic_only": True, "reuse_authorized": False,
            "normal": baseline, "coverage": phases,
            "coverage_command_seconds_sum": command,
            "coverage_diagnostics_seconds_sum": diagnostics,
            "unavailable_reasons": reasons,
            "scope": "Cargo command wall time and post-command listing/source diagnostics only; "
                     "excludes pre-command setup, JSON save, full job/runner/CI wait; not savings"}


def timing_summary(report: dict | None) -> str:
    if not isinstance(report, dict):
        return "Invocation timing: not measured.\n"
    rows = ["### Invocation timing (diagnostic only)", "",
            "| Invocation | Command seconds | Post-command diagnostics seconds | Exit |",
            "| --- | ---: | ---: | ---: |"]
    records = [("normal", report.get("normal"))]
    records.extend((f"coverage[{index}]", record)
                   for index, record in enumerate(report.get("coverage", [])))
    for label, record in records:
        rows.append(f"| {label} | not measured | not measured | not measured |" if record is None else
                    f"| {label} | {record['command_seconds']:.6f} | "
                    f"{record['diagnostics_seconds']:.6f} | {record['exit_code']} |")
    total = report.get("coverage_command_seconds_sum")
    rows.extend(["", f"Coverage command sum: {total:.6f} seconds." if total is not None else
                 "Coverage command sum: not measured.", "", report["scope"], ""])
    return "\n".join(rows)
