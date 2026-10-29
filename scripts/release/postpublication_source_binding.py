"""Bind published tags to trusted release workflow sources."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
import re

from release_branch_version import release_branch_version

REPOSITORY = "HiroyukiFuruno/KatanA"
WORKFLOW = ".github/workflows/build-and-release.yml"
VERSION_PATTERN = re.compile(r"v(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\Z")
COMMIT_PATTERN = re.compile(r"[0-9a-f]{40}\Z")


@dataclass(frozen=True)
class SourceBinding:
    kind: str
    product_commit: str
    tag: str
    run_head_sha: str
    head_branch: str
    event: str
    pull_request_number: int | None = None
    merged_at: datetime | None = None


class PreMergePullRequestRun(ValueError):
    """A matching PR head run started before the PR was merged."""


def validate_version(version: object) -> str:
    if not isinstance(version, str) or not VERSION_PATTERN.fullmatch(version):
        raise ValueError("release tag must be a full vX.Y.Z version")
    return version


def _full_commit(value: object, label: str) -> str:
    if not isinstance(value, str) or not COMMIT_PATTERN.fullmatch(value):
        raise ValueError(f"{label} must be a full lowercase commit")
    return value


def verify_tag_source(tag_record: object, source_commit: str) -> str:
    source_commit = _full_commit(source_commit, "published tag source")
    if not isinstance(tag_record, dict):
        raise ValueError("published tag metadata is invalid")
    tag_commit = _full_commit(tag_record.get("sha"), "published tag target")
    if tag_commit != source_commit:
        raise ValueError("published tag target differs from the supplied source commit")
    return tag_commit


def _positive_id(value: object, label: str) -> int:
    if type(value) is not int or value <= 0:
        raise ValueError(f"{label} must be a positive integer")
    return value


def _same_repository(value: object, label: str) -> None:
    if not isinstance(value, dict) or not isinstance(value.get("full_name"), str):
        raise ValueError(f"{label} repository metadata is missing")
    if value["full_name"].casefold() != REPOSITORY.casefold():
        raise ValueError(f"{label} repository is not {REPOSITORY}")


def parse_utc_timestamp(value: object, label: str) -> datetime:
    if not isinstance(value, str):
        raise ValueError(f"{label} timestamp is missing")
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as error:
        raise ValueError(f"{label} timestamp is invalid") from error
    if parsed.tzinfo is None or parsed.utcoffset() is None:
        raise ValueError(f"{label} timestamp must include a timezone")
    return parsed.astimezone(timezone.utc)


def bind_merged_pull_request(pull_request: object, product_commit: str,
                             version: str) -> SourceBinding:
    product_commit = _full_commit(product_commit, "product source")
    version = validate_version(version)
    if not isinstance(pull_request, dict):
        raise ValueError("associated pull request metadata is invalid")
    if (pull_request.get("state") != "closed" or pull_request.get("merged") is not True
            or pull_request.get("merge_commit_sha") != product_commit):
        raise ValueError("pull request is not merged into the published tag commit")
    number = _positive_id(pull_request.get("number"), "pull request number")
    merged_at = parse_utc_timestamp(pull_request.get("merged_at"), "pull request merge")
    base, head = pull_request.get("base"), pull_request.get("head")
    if not isinstance(base, dict) or not isinstance(head, dict):
        raise ValueError("pull request base or head metadata is missing")
    _same_repository(base.get("repo"), "pull request base")
    _same_repository(head.get("repo"), "pull request head")
    if (base.get("ref") != "master"
            or release_branch_version(head.get("ref")) != version.removeprefix("v")):
        raise ValueError("pull request branches do not match the published release")
    head_sha = _full_commit(head.get("sha"), "pull request head")
    return SourceBinding("merged_pull_request", product_commit, version, head_sha,
                         head["ref"], "pull_request", number, merged_at)


def select_merged_pull_request(pull_requests: object, product_commit: str,
                               version: str) -> SourceBinding | None:
    if not isinstance(pull_requests, list):
        raise ValueError("associated pull request records are invalid")
    bindings = []
    for pull_request in pull_requests:
        if not isinstance(pull_request, dict):
            raise ValueError("associated pull request record is invalid")
        if pull_request.get("merge_commit_sha") == product_commit:
            bindings.append(bind_merged_pull_request(pull_request, product_commit, version))
    if len(bindings) > 1:
        raise ValueError("published tag commit has ambiguous merged pull requests")
    return bindings[0] if bindings else None


def _validated_workflow_run(run: object) -> tuple[dict, str]:
    if not isinstance(run, dict):
        raise ValueError("workflow run metadata is invalid")
    _positive_id(run.get("id"), "workflow run id")
    _positive_id(run.get("run_attempt"), "workflow run attempt")
    _same_repository(run.get("repository"), "workflow run")
    _same_repository(run.get("head_repository"), "workflow run head")
    if (run.get("path") != WORKFLOW or run.get("status") != "completed"
            or run.get("conclusion") not in {"success", "failure", "cancelled"}):
        raise ValueError("workflow run path or terminal state is invalid")
    head_sha = _full_commit(run.get("head_sha"), "workflow run head")
    return run, head_sha


def _bind_direct_run(run: dict, product_commit: str, version: str,
                     head_sha: str) -> SourceBinding:
    if (run.get("event") != "workflow_dispatch" or run.get("head_branch") != "master"
            or head_sha != product_commit):
        raise ValueError("direct workflow run is not bound to the published master commit")
    return SourceBinding("direct_master_dispatch", product_commit, version, head_sha,
                         "master", "workflow_dispatch")


def _bind_pull_request_run(run: dict, product_commit: str, version: str, head_sha: str,
                           pull_request: SourceBinding) -> SourceBinding:
    if (pull_request.kind != "merged_pull_request"
            or pull_request.product_commit != product_commit
            or pull_request.tag != version
            or release_branch_version(pull_request.head_branch) != version.removeprefix("v")):
        raise ValueError("verified pull request binding does not match release source")
    if (run.get("event") != "pull_request"
            or run.get("head_branch") != pull_request.head_branch
            or head_sha != pull_request.run_head_sha):
        raise ValueError("pull request workflow run does not match verified head and branch")
    created_at = parse_utc_timestamp(run.get("created_at"), "workflow run creation")
    if pull_request.merged_at is None or created_at < pull_request.merged_at:
        raise PreMergePullRequestRun("pull request workflow run predates its verified merge")
    return SourceBinding("merged_pull_request", product_commit, version, head_sha,
                         pull_request.head_branch, "pull_request",
                         pull_request.pull_request_number, pull_request.merged_at)


def bind_workflow_run(run: object, product_commit: str, version: str,
                      pull_request: SourceBinding | None = None) -> SourceBinding:
    product_commit = _full_commit(product_commit, "product source")
    version = validate_version(version)
    run, head_sha = _validated_workflow_run(run)
    if pull_request is None:
        return _bind_direct_run(run, product_commit, version, head_sha)
    return _bind_pull_request_run(run, product_commit, version, head_sha, pull_request)
