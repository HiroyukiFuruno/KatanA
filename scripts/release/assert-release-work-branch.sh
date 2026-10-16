#!/usr/bin/env bash
set -euo pipefail

branch="$(git branch --show-current)"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ -z "$branch" ]]; then
  printf '%s\n' 'error: detached HEAD cannot be used for KatanA work commits.' >&2
  exit 1
fi

if bash "$script_dir/extract-release-branch-version.sh" "$branch" >/dev/null 2>&1; then
  exit 0
fi

printf '%s\n' "error: work commits must use release/vX.Y.Z branches; current branch is '$branch'." >&2
printf '%s\n' 'Create or switch to the target release branch before editing.' >&2
exit 1
