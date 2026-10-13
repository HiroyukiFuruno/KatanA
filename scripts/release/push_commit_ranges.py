"""Verify branch-update commit selections without accepting any Issue evidence."""

from pathlib import Path

from downstream_git_binding import GitBindingError, _git, _source_commit
from push_ref_updates import OID_RE, PushRefUpdate


class PushRangeError(GitBindingError):
    """A branch update does not have a verifiable local commit range."""


def _commit(root: Path, oid: str) -> None:
    if not isinstance(oid, str) or OID_RE.fullmatch(oid) is None:
        raise PushRangeError("range identity must be a full lowercase OID")
    _source_commit(root, oid)


def _base(root: Path, update: PushRefUpdate, new_ref_base: str | None) -> str:
    if not update.created:
        return update.remote_oid
    if new_ref_base is None:
        raise PushRangeError("new branch requires an explicit verified base")
    _commit(root, new_ref_base)
    return new_ref_base


def branch_commit_selection(root: Path, update: PushRefUpdate,
                            *, new_ref_base: str | None = None) -> tuple[str, ...]:
    """Select only verified fast-forward commits; never publication/cleanup proof."""
    if not update.remote_ref.startswith("refs/heads/"):
        raise PushRangeError("this selector verifies branch updates, not publication tags")
    if _git(root, ["rev-parse", "--is-shallow-repository"]).strip() != b"false":
        raise PushRangeError("branch selection requires complete commit history")
    if update.deleted:
        _commit(root, update.remote_oid)
        return ()
    _commit(root, update.local_oid)
    base = _base(root, update, new_ref_base)
    _commit(root, base)
    _git(root, ["merge-base", "--is-ancestor", base, update.local_oid])
    output = _git(root, ["rev-list", "--reverse", f"{base}..{update.local_oid}"])
    commits = tuple(output.decode("ascii").splitlines())
    for commit in commits:
        _commit(root, commit)
    return commits
