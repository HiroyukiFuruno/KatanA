"""Bind cleanup source commits to the actual published remote tag graph."""

from pathlib import Path
import re

from release_cleanup_audit import CleanupRefused, git

OID = re.compile(r"(?:[0-9a-f]{40}|[0-9a-f]{64})\Z")


def advertised_tag(root: Path, destination: str, tag: str) -> tuple[str, str]:
    reference = "refs/tags/" + tag
    git(root, "check-ref-format", reference)
    raw = git(root, "ls-remote", "--exit-code", "--", destination, reference, reference + "^{}")
    try:
        lines = raw.decode("ascii").splitlines()
    except UnicodeError as error:
        raise CleanupRefused("published tag advertisement is not ASCII") from error
    records: dict[str, str] = {}
    for line in lines:
        fields = line.split("\t")
        if len(fields) != 2 or fields[1] not in {reference, reference + "^{}"}:
            raise CleanupRefused("published tag advertisement is malformed")
        oid, name = fields
        if OID.fullmatch(oid) is None or name in records:
            raise CleanupRefused("published tag identity is ambiguous")
        records[name] = oid
    if reference not in records:
        raise CleanupRefused("published tag is absent from the verified remote")
    tag_oid = records[reference]
    commit_oid = records.get(reference + "^{}", tag_oid)
    if len(tag_oid) != len(commit_oid):
        raise CleanupRefused("published tag object formats differ")
    return tag_oid, commit_oid


def require_published_commit(root: Path, destination: str, tag: str, expected_oid: str) -> str:
    if OID.fullmatch(expected_oid) is None:
        raise CleanupRefused("cleanup source requires a full commit identity")
    before = advertised_tag(root, destination, tag)
    # 公開tagのobjectだけ取得し、local tagやFETCH_HEADを上書きしない。
    git(root, "fetch", "--no-tags", "--no-write-fetch-head", "--no-recurse-submodules",
        "--no-prune", "--no-prune-tags", "--refmap=", "--no-auto-maintenance",
        "--no-write-commit-graph",
        "--", destination, before[0])
    if advertised_tag(root, destination, tag) != before:
        raise CleanupRefused("published tag changed during cleanup admission")
    actual = git(root, "rev-parse", "--verify", before[0] + "^{commit}").decode().strip()
    source = git(root, "rev-parse", "--verify", expected_oid + "^{commit}").decode().strip()
    if actual != before[1] or source != expected_oid:
        raise CleanupRefused("published tag does not identify the advertised commit")
    try:
        git(root, "merge-base", "--is-ancestor", expected_oid, actual)
    except CleanupRefused as error:
        raise CleanupRefused("cleanup source is not included in the published tag") from error
    return actual
