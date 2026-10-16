"""Distinguish an already-absent target from transport failure or a changed tip."""

from pathlib import Path

from release_cleanup_audit import CleanupRefused, git


def remote_tip(root: Path, destination: str, branch: str, expected: str) -> str | None:
    reference = "refs/heads/" + branch
    raw = git(root, "ls-remote", destination, reference)
    if not raw:
        return None
    if raw.split() != [expected.encode("ascii"), reference.encode("utf-8")]:
        raise CleanupRefused("remote cleanup target changed or its advertisement is ambiguous")
    return expected
