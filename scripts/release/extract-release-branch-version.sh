#!/bin/bash
set -euo pipefail

branch=${1:-}

if [[ "$branch" =~ ^release/v([0-9]+\.[0-9]+\.[0-9]+)(-[A-Za-z0-9][A-Za-z0-9._-]*)?$ ]]; then
    printf '%s\n' "${BASH_REMATCH[1]}"
    exit 0
fi

printf '[ERROR] Invalid release branch: %s\n' "$branch" >&2
exit 1
