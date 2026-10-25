"""Bind an Issue lookup payload to a real commit, not to publication proof."""

from pathlib import Path

from downstream_git_binding import GitBindingError, _source_commit, validate_commit_tree_bindings
from downstream_issue_evidence import parse_issue_evidence
from issue_reference_live import IssueLookupError, candidate_numbers, is_genuine_issue
from push_ref_updates import OID_RE


class IssueCommitBindingError(GitBindingError):
    """Issue lookup metadata and its declared source cannot be bound."""


def _body(payload: dict[str, object], repository: str, number: int) -> str:
    try:
        candidate_numbers([number])
        if not is_genuine_issue(payload, repository, number):
            raise IssueCommitBindingError("a pull request is not a genuine Issue")
    except IssueLookupError as error:
        raise IssueCommitBindingError("Issue lookup identity does not match") from error
    if "body" not in payload or not isinstance(payload["body"], (str, type(None))):
        raise IssueCommitBindingError("Issue body is missing or has an invalid type")
    return payload["body"] or ""


def bind_issue_commit(payload: dict[str, object], repository: str, number: int,
                      root: Path, commit: str) -> dict[str, object] | None:
    """Bind lookup data only; None remains unclassified, never admission proof."""
    if not isinstance(commit, str) or OID_RE.fullmatch(commit) is None:
        raise IssueCommitBindingError("expected source must be a full lowercase OID")
    body = _body(payload, repository, number)
    _source_commit(root, commit)
    document = parse_issue_evidence(body, repository)
    if document is None:
        return None
    if not any(change["commit"] == commit for change in document["changes"]):
        raise IssueCommitBindingError("Issue evidence does not include the expected source commit")
    validate_commit_tree_bindings(document, root, repository)
    return document
