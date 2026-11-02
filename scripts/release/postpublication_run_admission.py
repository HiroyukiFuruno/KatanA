"""Admit only successful product and publication jobs for a published source."""

from __future__ import annotations

import argparse
import base64
import json
import re
import subprocess
import sys
from datetime import datetime
from pathlib import Path
from manual_bump_artifact import (download_artifact, manual_steps_succeeded, validate_artifact,
                                  validate_commit_metadata)
from manual_bump_transition import git as transition_git, verify_transition
from postpublication_source_binding import (
    PreMergePullRequestRun,
    REPOSITORY,
    WORKFLOW,
    SourceBinding,
    bind_workflow_run,
    parse_utc_timestamp,
    select_merged_pull_request,
    validate_version,
    verify_tag_source,
)

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


def _validate_run_metadata(run: object, run_head_sha: str) -> tuple[dict, int, int]:
    if not isinstance(run_head_sha, str) or not FULL_COMMIT_PATTERN.fullmatch(run_head_sha):
        raise ValueError("workflow run head commit is invalid")
    if not isinstance(run, dict) or not _positive(run.get("id")) or not _positive(run.get("run_attempt")):
        raise ValueError("run metadata identity is invalid")
    if (not isinstance(run.get("head_sha"), str) or run.get("head_sha") != run_head_sha
            or run.get("path") != WORKFLOW
            or run.get("event") not in {"workflow_dispatch", "pull_request"}
            or run.get("status") != "completed"
            or not isinstance(run.get("conclusion"), str)
            or run.get("conclusion") not in {"success", "failure", "cancelled"}):
        raise ValueError("run source, workflow, event, or terminal state does not match")
    repository = run.get("repository")
    head_repository = run.get("head_repository")
    if (not isinstance(repository, dict) or repository.get("full_name") != REPOSITORY
            or not isinstance(head_repository, dict)
            or head_repository.get("full_name") != REPOSITORY):
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


def verify_run(run: dict, pages: list, source: str, run_head_sha: str | None = None) -> dict:
    if not isinstance(source, str) or not FULL_COMMIT_PATTERN.fullmatch(source):
        raise ValueError("full source commit is invalid")
    expected_head = run_head_sha or source
    run, run_id, attempt = _validate_run_metadata(run, expected_head)
    jobs = _flatten_job_pages(pages)
    by_name = _collect_named_jobs(jobs, run_id, attempt, expected_head)
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
    parsed = parse_utc_timestamp(created_at, "candidate run creation")
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
            if not isinstance(run, dict):
                raise ValueError("workflow run candidate is invalid")
            if run.get("head_sha") != source:
                continue
            _run_sort_key(run, source)
            candidates.append(run)
    return candidates


def _pull_request_numbers(pages: object) -> list[int]:
    if not isinstance(pages, list):
        raise ValueError("associated pull request pages are invalid")
    numbers = set()
    for page in pages:
        if not isinstance(page, list):
            raise ValueError("associated pull request page is invalid")
        for pull_request in page:
            if not isinstance(pull_request, dict):
                raise ValueError("associated pull request record is invalid")
            number = pull_request.get("number")
            if not _positive(number):
                raise ValueError("associated pull request number is invalid")
            numbers.add(number)
    return sorted(numbers)


def _api_object(endpoint: str) -> dict:
    try:
        result = subprocess.run(
            ["gh", "api", endpoint], check=False, capture_output=True, text=True,
            timeout=API_TIMEOUT_SECONDS,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise RuntimeError(f"GitHub API request failed: {error}") from error
    if result.returncode:
        raise RuntimeError(f"GitHub API request failed ({result.returncode}): {result.stderr.strip()}")
    try:
        value = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError("GitHub API returned invalid JSON") from error
    if not isinstance(value, dict):
        raise RuntimeError("GitHub API returned invalid pull request metadata")
    return value


def _is_release_pull_request(pull_request: dict, source: str) -> bool:
    if pull_request.get("merge_commit_sha") != source:
        return False
    head = pull_request.get("head")
    if isinstance(head, dict) and isinstance(head.get("ref"), str):
        return head["ref"].startswith("release/")
    return True


def _verified_pull_request(source: str, version: str) -> SourceBinding | None:
    pages = _api_json(f"repos/{REPOSITORY}/commits/{source}/pulls?per_page=100")
    pull_requests = []
    for number in _pull_request_numbers(pages):
        pull_request = _api_object(f"repos/{REPOSITORY}/pulls/{number}")
        if pull_request.get("number") != number:
            raise ValueError("associated pull request detail identity differs")
        if _is_release_pull_request(pull_request, source):
            pull_requests.append(pull_request)
    return select_merged_pull_request(pull_requests, source, version)


def _source_binding_record(binding: SourceBinding) -> dict:
    return {
        "kind": binding.kind,
        "product_commit": binding.product_commit,
        "tag": binding.tag,
        "run_head_sha": binding.run_head_sha,
        "head_branch": binding.head_branch,
        "event": binding.event,
        "pull_request_number": binding.pull_request_number,
        "merged_at": binding.merged_at.isoformat() if binding.merged_at else None,
    }


def _write_new_output(output: Path, admitted: dict) -> None:
    serialized = json.dumps(admitted, ensure_ascii=False, indent=2) + "\n"
    # 出力証跡を既存ファイルやリンクで上書きしない。
    with output.open("x", encoding="utf-8") as stream:
        stream.write(serialized)


def _direct_candidates(source: str, version: str) -> list[tuple[dict, SourceBinding]]:
    run_pages = _pages(_api_json(
        f"repos/{REPOSITORY}/actions/workflows/build-and-release.yml/runs?head_sha={source}&per_page=100"
    ))
    candidates = []
    for run in _candidate_runs(run_pages, source):
        if run.get("event") != "workflow_dispatch":
            continue
        candidates.append((run, bind_workflow_run(run, source, version)))
    return candidates


def _pull_request_candidates(source: str, version: str) -> list[tuple[dict, SourceBinding]]:
    pull_request = _verified_pull_request(source, version)
    if pull_request is None:
        return []
    run_pages = _pages(_api_json(
        f"repos/{REPOSITORY}/actions/workflows/build-and-release.yml/runs?branch={pull_request.head_branch}&event=pull_request&head_sha={pull_request.run_head_sha}&per_page=100"
    ))
    candidates = []
    for run in _candidate_runs(run_pages, pull_request.run_head_sha):
        if run.get("event") != "pull_request":
            continue
        try:
            binding = bind_workflow_run(run, source, version, pull_request)
        except PreMergePullRequestRun:
            continue
        candidates.append((run, binding))
    return candidates


def _newest_candidate(candidates: list[tuple[dict, SourceBinding]]) -> tuple[dict, SourceBinding]:
    if not candidates:
        raise ValueError("no direct or merged-pull-request workflow run matches the published tag")
    return max(candidates, key=lambda item: _run_sort_key(item[0], item[1].run_head_sha))


def bind_generated_transition(root: Path, run: dict, artifact: dict, archive: bytes,
                              trigger: str, source: str, version: str) -> tuple[SourceBinding, dict]:
    bind_workflow_run(run, trigger, version)
    trusted = (WORKFLOW, "scripts/release/bump-version.sh", "scripts/release/manual_bump_transition.py",
               "scripts/release/manual_bump_graphql.py")
    for path in trusted:
        transition_git(root, "cat-file", "-e", f"{trigger}:{path}")
        transition_git(root, "cat-file", "-e", f"HEAD:{path}")
    if transition_git(root, "diff", "--name-only", "-z", trigger, "HEAD", "--", *trusted):
        raise ValueError("manual transition producer differs from the trusted controller")
    record, raw = validate_artifact(artifact, run, archive)
    commits = verify_transition(root, record, trigger, source, version, run["id"], run["run_attempt"])
    binding = SourceBinding("generated_manual_bump", source, version, trigger, "master", "workflow_dispatch")
    return binding, {"artifact": artifact, "transition": record,
                     "transition_raw_base64": base64.b64encode(raw).decode("ascii"), "commits": commits}


def _generated_candidates(root: Path, source: str, version: str) -> list[tuple[dict, SourceBinding, dict]]:
    result = []
    for distance in (1, 2, 3):
        result.extend(_generated_candidates_at_distance(root, source, version, distance))
    return result


def _generated_candidates_at_distance(root: Path, source: str, version: str,
                                      distance: int) -> list[tuple[dict, SourceBinding, dict]]:
    try:
        trigger = transition_git(root, "rev-parse", "--verify", f"{source}~{distance}").decode().strip()
    except subprocess.CalledProcessError:
        return []
    pages = _pages(_api_json(
        f"repos/{REPOSITORY}/actions/workflows/build-and-release.yml/runs?head_sha={trigger}&per_page=100"
    ))
    result = []
    for run in _candidate_runs(pages, trigger):
        if run.get("event") != "workflow_dispatch" or run.get("head_branch") != "master":
            continue
        bind_workflow_run(run, trigger, version)
        artifact_pages = _pages(_api_json(f"repos/{REPOSITORY}/actions/runs/{run['id']}/artifacts?per_page=100"))
        artifacts = [artifact for page in artifact_pages for artifact in page.get("artifacts", [])
                     if artifact.get("name") == f"manual-bump-transition-{run['run_attempt']}"]
        if not artifacts:
            continue
        if len(artifacts) != 1:
            raise ValueError("duplicate manual transition artifacts")
        artifact = artifacts[0]
        archive = download_artifact(artifact)
        advertised, _ = validate_artifact(artifact, run, archive)
        if advertised.get("source") != source or advertised.get("version") != version:
            continue
        binding, evidence = bind_generated_transition(root, run, artifact, archive, trigger, source, version)
        result.append((run, binding, evidence))
    return result


def admit(source: str, version: str, output: Path, root: Path | None = None) -> tuple[int, int, str]:
    if not isinstance(output, Path):
        raise ValueError("output path must be a Path")
    if not isinstance(source, str) or not FULL_COMMIT_PATTERN.fullmatch(source):
        raise ValueError("full source commit is invalid")
    version = validate_version(version)
    verify_tag_source(_api_object(f"repos/{REPOSITORY}/commits/{version}"), source)
    candidates = _direct_candidates(source, version) + _pull_request_candidates(source, version)
    generated = _generated_candidates(root or Path.cwd(), source, version) if not candidates else []
    candidates += [(run, binding) for run, binding, _ in generated]

    newest, binding = _newest_candidate(candidates)
    jobs_value = _api_json(
        f"repos/{REPOSITORY}/actions/runs/{newest['id']}/attempts/{newest['run_attempt']}/jobs?per_page=100"
    )
    admitted = verify_run(newest, _pages(jobs_value), source, binding.run_head_sha)
    admitted["source_binding"] = _source_binding_record(binding)
    if binding.kind == "generated_manual_bump":
        manual_steps_succeeded(admitted["jobs"])
        evidence = next(evidence for run, _, evidence in generated if run["id"] == newest["id"])
        metadata = [_api_object(f"repos/{REPOSITORY}/commits/{commit}") for commit in evidence["commits"]]
        validate_commit_metadata(metadata, evidence["commits"], binding.run_head_sha)
        admitted["manual_bump"] = evidence | {"commit_metadata": metadata}
    _write_new_output(output, admitted)
    return newest["id"], newest["run_attempt"], source


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--source", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    if args.repository != REPOSITORY:
        parser.error(f"--repository must be exactly {REPOSITORY}")
    if not FULL_COMMIT_PATTERN.fullmatch(args.source):
        parser.error("--source must be a full lowercase 40-character commit")
    try:
        validate_version(args.version)
    except ValueError as error:
        parser.error(str(error))
    try:
        run_id, attempt, source = admit(args.source, args.version, args.output)
    except (RuntimeError, ValueError, OSError) as error:
        print(f"postpublication admission failed: {error}", file=sys.stderr)
        return 1
    print(f"{run_id}\t{attempt}\t{source}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
