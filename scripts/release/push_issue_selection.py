"""Bind branch commit messages to Issue candidates; not full push admission."""

from dataclasses import dataclass
from pathlib import Path

from downstream_git_binding import GitBindingError, _git, _source_commit
from issue_reference_syntax import issue_references, parse_repository
from issue_reference_live import LiveIssueClient, candidate_numbers
from push_commit_ranges import MAX_SELECTED_COMMITS, branch_commit_selection
from push_ref_updates import PushRefUpdate


MAX_COMMIT_OBJECT_BYTES = 1_048_576


class PushIssueSelectionError(GitBindingError):
    """A source commit does not have verifiable same-repository Issue references."""


@dataclass(frozen=True)
class CommitIssueCandidates:
    commit: str
    numbers: tuple[int, ...]


@dataclass(frozen=True)
class PushIssueSelection:
    commits: tuple[CommitIssueCandidates, ...]
    outside_branch_scope: tuple[str, ...]


def _references(root: Path, commit: str, repository: str) -> CommitIssueCandidates:
    _source_commit(root, commit)
    size = _git(root, ["cat-file", "-s", commit]).strip()
    if not size.isdigit() or int(size) > MAX_COMMIT_OBJECT_BYTES:
        raise PushIssueSelectionError("commit object exceeds the Issue parsing limit")
    try:
        message = _git(root, ["show", "-s", "--no-show-signature", "--encoding=UTF-8",
                              "--format=%B", commit]).decode("utf-8")
    except UnicodeError as error:
        raise PushIssueSelectionError("commit message is not UTF-8") from error
    try:
        numbers = tuple(candidate_numbers(issue_references(message, repository)))
    except ValueError as error:
        raise PushIssueSelectionError("commit Issue candidates are invalid") from error
    if not numbers:
        raise PushIssueSelectionError("a pushed non-default commit has no Issue reference")
    return CommitIssueCandidates(commit, numbers)


def select_commit_issue_references(root: Path, updates: tuple[PushRefUpdate, ...],
                                   repository: str, *, default_ref: str,
                                   new_ref_base: str | None = None) -> PushIssueSelection:
    parse_repository(repository)
    if not isinstance(default_ref, str) or not default_ref.startswith("refs/heads/"):
        raise PushIssueSelectionError("default branch must be an explicit branch ref")
    _git(root, ["check-ref-format", default_ref])
    commits: dict[str, CommitIssueCandidates] = {}
    outside_scope: list[str] = []
    for update in updates:
        if (not update.remote_ref.startswith("refs/heads/")
                or update.deleted or update.remote_ref == default_ref):
            outside_scope.append(update.remote_ref)
            continue
        selected = branch_commit_selection(root, update, new_ref_base=new_ref_base)
        for commit in selected:
            if commit not in commits:
                if len(commits) >= MAX_SELECTED_COMMITS:
                    raise PushIssueSelectionError("pushed commit selection exceeds the limit")
                commits[commit] = _references(root, commit, repository)
    # tag/default/deletionの別契約を、Issue検査の成功へすり替えない。
    return PushIssueSelection(tuple(commits.values()), tuple(outside_scope))


def lookup_selected_issues(selection: PushIssueSelection,
                           repository: str) -> dict[str, tuple[dict[str, object], ...]]:
    numbers = candidate_numbers([number for item in selection.commits for number in item.numbers])
    payloads = LiveIssueClient(repository).genuine_payloads(numbers)
    result: dict[str, tuple[dict[str, object], ...]] = {}
    for item in selection.commits:
        verified = tuple(payloads[number] for number in item.numbers if number in payloads)
        if not verified:
            raise PushIssueSelectionError("a pushed commit has no genuine same-repository Issue")
        result[item.commit] = verified
    return result
