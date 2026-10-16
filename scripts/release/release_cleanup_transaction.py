"""Validate the locked reference before an admitted native branch deletion."""

import argparse
from pathlib import Path
import sys

from release_cleanup_audit import CleanupRefused, common_directory, git
from release_cleanup_tag import OID

MAX_TRANSACTION_BYTES = 65536


def verify_prepared_deletion(root: Path, branch: str, expected: str, payload: bytes) -> None:
    reference = "refs/heads/" + branch
    if not branch.startswith("release/") or OID.fullmatch(expected) is None:
        raise CleanupRefused("cleanup transaction requires an explicit release branch and full OID")
    git(root, "check-ref-format", reference)
    zero = "0" * len(expected)
    if len(payload) > MAX_TRANSACTION_BYTES:
        raise CleanupRefused("cleanup transaction exceeds its input bound")
    try:
        records = [line.split() for line in payload.decode("ascii").splitlines()]
    except UnicodeError as error:
        raise CleanupRefused("cleanup transaction input is not ASCII") from error
    if len(records) != 1 or records[0] not in ([zero, zero, reference], [expected, zero, reference]):
        raise CleanupRefused("cleanup transaction does not match the admitted deletion")
    if git(root, "for-each-ref", "--format=%(symref)", reference).strip():
        raise CleanupRefused("cleanup transaction refuses a symbolic branch")
    # preparedではGitがrefをlock済みだがold値はzeroの場合があるため、実tipを照合する。
    actual = git(root, "rev-parse", "--verify", reference).decode().strip()
    if actual != expected:
        raise CleanupRefused("cleanup branch moved after its admission audit")


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--branch", required=True)
    parser.add_argument("--expected", required=True)
    parser.add_argument("phase", choices=("preparing", "prepared", "committed", "aborted"))
    options = parser.parse_args(arguments)
    payload = sys.stdin.buffer.read(MAX_TRANSACTION_BYTES + 1)
    try:
        if options.phase == "prepared":
            if common_directory(options.root) != common_directory(Path.cwd()):
                raise CleanupRefused("cleanup root differs from the locked transaction repository")
            verify_prepared_deletion(options.root, options.branch, options.expected, payload)
    except (CleanupRefused, OSError, UnicodeError) as error:
        print(f"cleanup transaction refused: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
