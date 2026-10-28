"""Admit only successful product and publication jobs for a published source."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from datetime import datetime
from pathlib import Path

REPOSITORY = "HiroyukiFuruno/KatanA"
WORKFLOW = ".github/workflows/build-and-release.yml"
REQUIRED_JOBS = frozenset((
    "Preflight Check", "Build Linux", "Build macOS", "Build Windows",
    "Smoke macOS (x86_64)", "Smoke macOS (arm64)", "Publish Release",
))
OPTIONAL_JOBS = frozenset((
    "Compare Windows packaged startup (Server 2022)",
    "Audit and clean published release branches",
))
API_TIMEOUT_SECONDS = 30
TERMINAL_JOB_CONCLUSIONS = frozenset((
    "success", "failure", "neutral", "cancelled", "skipped", "timed_out",
    "action_required", "stale", "startup_failure",
))
FULL_COMMIT_PATTERN = re.compile(r"[0-9a-f]{40}")


def _positive(value: object) -> bool:
    return type(value) is int and value > 0


def _validate_run_metadata(run: object, source: str) -> tuple[dict, int, int]:
    if not isinstance(source, str) or not FULL_COMMIT_PATTERN.fullmatch(source):
        raise ValueError("full source commit is invalid")
    if not isinstance(run, dict) or not _positive(run.get("id")) or not _positive(run.get("run_attempt")):
        raise ValueError("run metadata identity is invalid")
    if (not isinstance(run.get("head_sha"), str) or run.get("head_sha") != source
            or run.get("path") != WORKFLOW
            or run.get("event") != "workflow_dispatch" or run.get("status") != "completed"
            or not isinstance(run.get("conclusion"), str)
            or run.get("conclusion") not in {"success", "failure", "cancelled"}):
        raise ValueError("run source, workflow, event, or terminal state does not match")
    repository = run.get("repository")
    if not isinstance(repository, dict) or repository.get("full_name") != REPOSITORY:
        raise ValueError("run repository does not match")
    return run, run["id"], run["run_attempt"]


def _flatten_job_pages(pages: object) -> list:
    if not isinstance(pages, list) or not pages:
        raise ValueError("attempt job pages are missing")
    jobs = []
    for page in pages:
        if not isinstance(page, dict) or not isinstance(page.get("jobs"), list):
            raise ValueError("attempt job page is invalid")
        jobs.extend(page["jobs"])
    if not jobs:
        raise ValueError("attempt jobs are empty")
    return jobs


def _validate_job_identity(job: object, run_id: int, attempt: int, source: str,
                           seen_ids: set) -> str:
    if not isinstance(job, dict) or not _positive(job.get("id")) or job["id"] in seen_ids:
        raise ValueError("job identity is invalid or duplicated")
    seen_ids.add(job["id"])
    if (type(job.get("run_id")) is not int or job["run_id"] != run_id
            or type(job.get("run_attempt")) is not int or job["run_attempt"] != attempt
            or not isinstance(job.get("head_sha"), str) or job.get("head_sha") != source):
        raise ValueError("job belongs to a different run, attempt, or source")
    name = job.get("name")
    if not isinstance(name, str) or name not in REQUIRED_JOBS | OPTIONAL_JOBS:
        raise ValueError(f"unknown job name: {name}")
    return name


def _collect_named_jobs(jobs: list, run_id: int, attempt: int, source: str) -> dict:
    seen_ids, seen_names = set(), set()
    by_name = {}
    for job in jobs:
        name = _validate_job_identity(job, run_id, attempt, source, seen_ids)
        if name in seen_names:
            raise ValueError(f"duplicate job name: {name}")
        seen_names.add(name)
        by_name[name] = job
    return by_name


def _validate_required_jobs(by_name: dict) -> None:
    missing = REQUIRED_JOBS - set(by_name)
    if missing:
        raise ValueError(f"required jobs are missing: {', '.join(sorted(missing))}")
    for name in REQUIRED_JOBS:
        job = by_name[name]
        if job.get("status") != "completed" or job.get("conclusion") != "success":
            raise ValueError(f"required job did not succeed: {name}")


def _optional_job_results(by_name: dict) -> dict:
    results = {}
    for name in sorted(OPTIONAL_JOBS):
        if name not in by_name:
            results[name] = {"status": "not_present", "conclusion": None}
            continue
        job = by_name[name]
        if (job.get("status") != "completed"
                or not isinstance(job.get("conclusion"), str)
                or job.get("conclusion") not in TERMINAL_JOB_CONCLUSIONS):
            raise ValueError(f"optional job has an invalid terminal state: {name}")
        results[name] = {"status": job["status"], "conclusion": job["conclusion"]}
    return results


def verify_run(run: dict, pages: list, source: str) -> dict:
    run, run_id, attempt = _validate_run_metadata(run, source)
    jobs = _flatten_job_pages(pages)
    by_name = _collect_named_jobs(jobs, run_id, attempt, source)
    _validate_required_jobs(by_name)
    return {
        "run": run,
        "jobs": jobs,
        "optional_job_results": _optional_job_results(by_name),
    }


def _api_json(endpoint: str) -> object:
    try:
        # WHY: CIにあるghを使い、開発端末専用のrtkを実行条件にしない。
        result = subprocess.run(
            ["gh", "api", "--paginate", "--slurp", endpoint],
            check=False, capture_output=True, text=True, timeout=API_TIMEOUT_SECONDS,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise RuntimeError(f"GitHub API request failed: {error}") from error
    if result.returncode:
        raise RuntimeError(f"GitHub API request failed ({result.returncode}): {result.stderr.strip()}")
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError("GitHub API returned invalid JSON") from error


def _pages(value: object) -> list:
    # gh --slurp はページ単位の JSON object を配列で返す。
    if not isinstance(value, list) or not value or any(not isinstance(page, dict) for page in value):
        raise ValueError("GitHub API pagination response is invalid")
    return value


def _run_sort_key(run: object, source: str) -> tuple[float, int]:
    if not isinstance(run, dict) or run.get("head_sha") != source:
        raise ValueError("candidate run metadata is invalid")
    run_id, attempt = run.get("id"), run.get("run_attempt")
    created_at = run.get("created_at")
    if (not _positive(run_id) or not _positive(attempt) or not isinstance(created_at, str)):
        raise ValueError("candidate run identity or creation time is invalid")
    try:
        parsed = datetime.fromisoformat(created_at.replace("Z", "+00:00"))
    except ValueError as error:
        raise ValueError("candidate run creation time is invalid") from error
    if parsed.tzinfo is None:
        raise ValueError("candidate run creation time has no timezone")
    try:
        timestamp = parsed.timestamp()
    except (OverflowError, OSError, ValueError) as error:
        raise ValueError("candidate run creation time is out of range") from error
    return timestamp, run_id


def _candidate_runs(run_pages: list, source: str) -> list:
    candidates = []
    for page in run_pages:
        page_runs = page.get("workflow_runs")
        if not isinstance(page_runs, list):
            raise ValueError("workflow run page is invalid")
        for run in page_runs:
            if not isinstance(run, dict) or run.get("head_sha") != source:
                raise ValueError("workflow run candidate source is invalid")
            _run_sort_key(run, source)
            candidates.append(run)
    if not candidates:
        raise ValueError("no workflow run exists for the source commit")
    return candidates


def _write_new_output(output: Path, admitted: dict) -> None:
    serialized = json.dumps(admitted, ensure_ascii=False, indent=2) + "\n"
    # 出力証跡を既存ファイルやリンクで上書きしない。
    with output.open("x", encoding="utf-8") as stream:
        stream.write(serialized)


def admit(source: str, output: Path) -> tuple[int, int, str]:
    if not isinstance(output, Path):
        raise ValueError("output path must be a Path")
    if not isinstance(source, str) or not FULL_COMMIT_PATTERN.fullmatch(source):
        raise ValueError("full source commit is invalid")
    runs_value = _api_json(
        f"repos/{REPOSITORY}/actions/workflows/build-and-release.yml/runs?head_sha={source}&per_page=100"
    )
    candidates = _candidate_runs(_pages(runs_value), source)
    # 同じ source に複数 run がある場合、古い成功 run への後退を防ぐ。
    newest = max(candidates, key=lambda run: _run_sort_key(run, source))
    jobs_value = _api_json(
        f"repos/{REPOSITORY}/actions/runs/{newest['id']}/attempts/{newest['run_attempt']}/jobs?per_page=100"
    )
    admitted = verify_run(newest, _pages(jobs_value), source)
    _write_new_output(output, admitted)
    return newest["id"], newest["run_attempt"], source


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--source", required=True)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    if args.repository != REPOSITORY:
        parser.error(f"--repository must be exactly {REPOSITORY}")
    if not FULL_COMMIT_PATTERN.fullmatch(args.source):
        parser.error("--source must be a full lowercase 40-character commit")
    try:
        run_id, attempt, source = admit(args.source, args.output)
    except (RuntimeError, ValueError, OSError) as error:
        print(f"postpublication admission failed: {error}", file=sys.stderr)
        return 1
    print(f"{run_id}\t{attempt}\t{source}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
