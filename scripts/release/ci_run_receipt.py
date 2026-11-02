"""Produce a bounded receipt for one verified master CI run."""

from datetime import datetime, timedelta, timezone
import hashlib
import json
import re

from ci_run_admission import FULL_JOBS, verify_full_run, verify_run


def make_receipt(before, pages, after, sha, run_id, repository, tree, *, now=None):
    run_id, attempt = verify_run(before, pages, after, sha, run_id, repository)
    latest = verify_full_run(before, pages, after, sha, run_id, repository)
    if any(before.get(key) != after.get(key) for key in ("event", "actor", "run_started_at")):
        raise ValueError("CI run identity changed while collecting its receipt")
    if not re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", tree):
        raise ValueError("source tree identity is invalid")
    current = now or datetime.now(timezone.utc)
    if current.tzinfo is None:
        raise ValueError("receipt clock must be timezone aware")
    started = before.get("run_started_at")
    if not isinstance(started, str):
        raise ValueError("CI run start time is missing")
    try:
        started_at = datetime.fromisoformat(started.replace("Z", "+00:00"))
    except ValueError as error:
        raise ValueError("CI run start time is invalid") from error
    # WHY: runner・ツール・脆弱性情報の経時変化を考慮し、古い結果の採用を防ぐ。
    if (started_at.tzinfo is None or started_at > current + timedelta(minutes=5)
            or started_at < current - timedelta(days=7)):
        raise ValueError("CI run is expired or has a future timestamp")
    snapshot = json.dumps({"before": before, "pages": pages, "after": after},
                          sort_keys=True, separators=(",", ":")).encode()
    return {"schema": 1, "scope": "master-three-os-ci", "repository": repository,
            "event": before["event"], "actor": before.get("actor"),
            "source_commit": sha, "source_tree": tree, "run_id": run_id, "run_attempt": attempt,
            "workflow": before["path"], "run_started_at": started,
            "observed_at": current.astimezone(timezone.utc).isoformat(),
            "snapshot_sha256": hashlib.sha256(snapshot).hexdigest(),
            "jobs": [{"name": name, "id": latest[name]["id"],
                      "attempt": latest[name]["run_attempt"], "conclusion": latest[name]["conclusion"],
                      "steps": [{"name": step["name"], "status": step["status"], "conclusion": step["conclusion"]}
                                for step in latest[name]["steps"] if isinstance(step, dict)
                                and step.get("name") and step.get("status") and step.get("conclusion")]}
                     for name in sorted(FULL_JOBS)],
            "generic_reuse_authorized": False}
