"""Read and authenticate a same-run manual version transition artifact."""
from __future__ import annotations

import hashlib
import io
import json
import subprocess
import zipfile

from manual_bump_transition import REPOSITORY


def _unique_keys(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate manual transition field")
        result[key] = value
    return result


def validate_artifact(artifact: dict, run: dict, archive: bytes) -> tuple[dict, bytes]:
    expected_name = f"manual-bump-transition-{run['run_attempt']}"
    workflow = artifact.get("workflow_run")
    digest = "sha256:" + hashlib.sha256(archive).hexdigest()
    if (artifact.get("name") != expected_name or artifact.get("expired") is not False
            or type(artifact.get("id")) is not int or artifact["id"] <= 0
            or artifact.get("digest") != digest or not isinstance(workflow, dict)
            or workflow.get("id") != run["id"] or workflow.get("head_sha") != run["head_sha"]):
        raise ValueError("manual transition artifact identity or digest differs")
    if len(archive) > 1_048_576:
        raise ValueError("manual transition artifact exceeds size limit")
    with zipfile.ZipFile(io.BytesIO(archive)) as package:
        entries = package.infolist()
        if len(entries) != 1 or entries[0].filename != "transition.json" or entries[0].file_size > 8192:
            raise ValueError("manual transition artifact must contain exactly one small record")
        raw = package.read(entries[0])
        record = json.loads(raw, object_pairs_hook=_unique_keys)
    if not isinstance(record, dict):
        raise ValueError("manual transition record is not an object")
    return record, raw


def download_artifact(artifact: dict) -> bytes:
    artifact_id = artifact.get("id")
    size = artifact.get("size_in_bytes")
    if (type(artifact_id) is not int or artifact_id <= 0
            or type(size) is not int or not 0 < size <= 1_048_576):
        raise ValueError("manual artifact ID or size is invalid")
    return subprocess.run(["gh", "api", f"repos/{REPOSITORY}/actions/artifacts/{artifact_id}/zip"],
                          check=True, capture_output=True, timeout=60).stdout


def manual_steps_succeeded(jobs: list) -> None:
    matches = [job for job in jobs if job.get("name") == "Preflight Check"]
    if len(matches) != 1 or not isinstance(matches[0].get("steps"), list):
        raise ValueError("manual transition preflight steps are missing")
    steps = matches[0]["steps"]
    for name in ("Bump version and push", "Record manual bump transition", "Upload manual bump transition"):
        found = [step for step in steps if isinstance(step, dict) and step.get("name") == name]
        if len(found) != 1 or found[0].get("status") != "completed" or found[0].get("conclusion") != "success":
            raise ValueError(f"manual transition step did not succeed: {name}")


def validate_commit_metadata(records: list, commits: list[str], trigger: str) -> None:
    if len(records) != len(commits):
        raise ValueError("manual bump commit metadata is incomplete")
    parent = trigger
    for record, commit in zip(records, commits):
        if (not isinstance(record, dict) or record.get("sha") != commit
                or not isinstance(record.get("parents"), list)
                or len(record["parents"]) != 1
                or not isinstance(record["parents"][0], dict)
                or [item.get("sha") for item in record["parents"] if isinstance(item, dict)] != [parent]
                or not isinstance(record.get("commit"), dict)
                or not isinstance(record["commit"].get("verification"), dict)
                or record["commit"]["verification"].get("verified") is not True):
            raise ValueError("manual bump commit is not verified or has a different parent")
        parent = commit
