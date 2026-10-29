"""Parse the shared release branch grammar without normalizing versions."""

import re

_RELEASE_BRANCH = re.compile(r"release/v([0-9]+\.[0-9]+\.[0-9]+)(-[A-Za-z0-9][A-Za-z0-9._-]*)?")


def release_branch_version(branch: object) -> str | None:
    # WHY: 既存のBash parserと同じ完全一致grammarでsuffixを版数から分離する。
    match = _RELEASE_BRANCH.fullmatch(branch) if isinstance(branch, str) else None
    return match[1] if match is not None else None
