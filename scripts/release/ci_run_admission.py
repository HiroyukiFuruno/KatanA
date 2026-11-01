"""Validate live post-merge push evidence without authorizing other check reuse."""

import re

REQUIRED_JOBS = frozenset(f"Test and Build ({platform})" for platform in
                          ("macos-latest", "windows-latest", "ubuntu-latest"))
LINT_JOBS = frozenset(f"Lint ({platform})" for platform in
                      ("macos-latest", "windows-latest", "ubuntu-latest"))
FULL_JOBS = REQUIRED_JOBS | LINT_JOBS | {"Dependency Supply Chain", "CodeQL Security Scan"}
WORKFLOW = ".github/workflows/test-and-build.yml"


def _positive(value):
    return type(value) is int and value > 0


def verify_identity(run: dict, sha: str, run_id: int, repository: str) -> int:
    expected = {"id": run_id, "head_sha": sha, "head_branch": "master",
                "path": WORKFLOW, "status": "completed", "conclusion": "success"}
    if (not isinstance(run, dict) or not _positive(run_id)
            or not re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", sha)
            or not _positive(run.get("id")) or not _positive(run.get("run_attempt"))):
        raise ValueError("CI run identity is invalid")
    if any(run.get(key) != value for key, value in expected.items()):
        raise ValueError("CI run source, workflow, event or success does not match")
    event = run.get("event")
    if event not in ("push", "workflow_dispatch"):
        raise ValueError("CI event is not a master push or trusted dispatch")
    if event == "workflow_dispatch" and (
            not isinstance(run.get("actor"), dict)
            or run["actor"].get("login") not in {"HiroyukiFuruno", "github-actions[bot]"}):
        raise ValueError("CI dispatch actor is not trusted")
    if not isinstance(run.get("repository"), dict) or run["repository"].get("full_name") != repository:
        raise ValueError("CI run repository does not match")
    return run["run_attempt"]


def _jobs(pages: list) -> list:
    if not isinstance(pages, list) or not pages:
        raise ValueError("CI job pages are missing")
    jobs, totals = [], []
    for page in pages:
        if (not isinstance(page, dict) or type(page.get("total_count")) is not int
                or page["total_count"] < 0 or not isinstance(page.get("jobs"), list)):
            raise ValueError("CI job page is invalid")
        totals.append(page["total_count"])
        jobs.extend(page["jobs"])
    if any(total != len(jobs) for total in totals):
        raise ValueError("CI job pagination is incomplete or inconsistent")
    return jobs


def _job_identity(job: dict, sha: str, run_id: int, attempt: int, identities: set) -> int:
    if not isinstance(job, dict) or not _positive(job.get("id")) or job["id"] in identities:
        raise ValueError("CI jobs have invalid or duplicate identities")
    identities.add(job["id"])
    if (type(job.get("run_id")) is not int or job["run_id"] != run_id
            or not _positive(job.get("run_attempt")) or job["run_attempt"] > attempt
            or job.get("head_sha") != sha or not isinstance(job.get("name"), str)):
        raise ValueError("CI job is not bound to the current source and run history")
    return job["run_attempt"]


def _verify_jobs(jobs: list, sha: str, run_id: int, attempt: int) -> None:
    latest, seen, identities, current = {}, set(), set(), False
    for job in jobs:
        observed = _job_identity(job, sha, run_id, attempt, identities)
        current = current or observed == attempt
        name = job["name"]
        if name in REQUIRED_JOBS:
            if (name, observed) in seen:
                raise ValueError("required CI job is duplicated in the same attempt")
            seen.add((name, observed))
            if name not in latest or observed > latest[name]["run_attempt"]:
                latest[name] = job
        elif name.startswith("Test and Build"):
            raise ValueError("CI test matrix has an unrecognized job")
    if not current or set(latest) != REQUIRED_JOBS:
        raise ValueError("CI did not complete every required platform job")
    if any(job.get("status") != "completed" or job.get("conclusion") != "success"
           for job in latest.values()):
        raise ValueError("the latest required CI job did not execute successfully")


def verify_run(before: dict, pages: list, after: dict, sha: str,
               run_id: int, repository: str) -> tuple[int, int]:
    attempt = verify_identity(before, sha, run_id, repository)
    _verify_jobs(_jobs(pages), sha, run_id, attempt)
    if verify_identity(after, sha, run_id, repository) != attempt:
        raise ValueError("CI attempt changed while collecting evidence")
    return run_id, attempt


def verify_full_run(before: dict, pages: list, after: dict, sha: str,
                    run_id: int, repository: str) -> dict:
    _, attempt = verify_run(before, pages, after, sha, run_id, repository)
    latest, identities = {}, set()
    for job in _jobs(pages):
        observed = _job_identity(job, sha, run_id, attempt, identities)
        name = job["name"]
        if name in FULL_JOBS:
            if name in latest and observed == latest[name]["run_attempt"]:
                raise ValueError("required CI job is duplicated in the same attempt")
            if name not in latest or observed > latest[name]["run_attempt"]:
                latest[name] = job
        elif name.startswith(("Test and Build", "Lint")):
            raise ValueError("CI matrix contains an unrecognized job")
    if set(latest) != FULL_JOBS:
        raise ValueError("full CI jobs are incomplete")
    for name, job in latest.items():
        if job.get("status") != "completed" or job.get("conclusion") != "success":
            raise ValueError("a required full CI job did not succeed")
        steps = job.get("steps")
        if not isinstance(steps, list):
            raise ValueError("required CI step evidence is missing")
        if name in REQUIRED_JOBS:
            expected = {"Run tests", "Test screenshot harness contracts", "Run multi-format headless acceptance"}
            if name == "Test and Build (macos-latest)":
                expected |= {"Install coverage tool", "Run coverage"}
        elif name in LINT_JOBS:
            expected = {"Run linter"}
            if name == "Lint (ubuntu-latest)":
                expected.add("Check formatting")
        elif name == "Dependency Supply Chain":
            expected = {"Check advisories, licenses, and sources"}
        else:
            expected = {"Initialize CodeQL", "Auto-build", "Perform CodeQL Analysis"}
        for step_name in expected:
            matches = [step for step in steps if isinstance(step, dict) and step.get("name") == step_name]
            if len(matches) != 1 or matches[0].get("status") != "completed" or matches[0].get("conclusion") != "success":
                raise ValueError("required CI step did not execute successfully: " + step_name)
    return latest
