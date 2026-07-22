#!/bin/bash
set -euo pipefail

GUARD=scripts/release/check-version-increment.sh
BRANCH_VERSION_GUARD=scripts/release/extract-release-branch-version.sh

expect_accept() {
    local target=$1
    local latest=$2
    local changelog=$3

    bash "$GUARD" "$target" <(printf '%s\n' "$changelog") "$latest" >/dev/null
}

expect_reject() {
    local target=$1
    local latest=$2
    local changelog=$3

    if bash "$GUARD" "$target" <(printf '%s\n' "$changelog") "$latest" >/dev/null 2>&1; then
        printf '[ERROR] Unexpectedly accepted v%s after v%s.\n' "$target" "$latest" >&2
        exit 1
    fi
}

expect_accept 0.22.35 0.22.34 $'## [0.22.35]\n## [0.22.34]'
expect_reject 0.22.34 0.22.34 $'## [0.22.34]\n## [0.22.33]'
expect_reject 0.22.36 0.22.34 $'## [0.22.36]\n## [0.22.35]'
expect_reject 0.29.0 0.22.34 $'## [0.29.0]\n## [0.22.34]'
expect_reject 0.23.0 0.22.34 $'## [0.23.0]\n## [0.22.34]'
expect_reject 1.0.0 0.22.34 $'## [1.0.0]\n## [0.22.34]'
expect_reject 0.22.35 0.22.34 $'## [0.22.35]\n## [0.22.33]'

[[ "$(bash "$BRANCH_VERSION_GUARD" release/v0.22.35)" == "0.22.35" ]]
[[ "$(bash "$BRANCH_VERSION_GUARD" release/v0.22.35-html-viewer)" == "0.22.35" ]]
if bash "$BRANCH_VERSION_GUARD" release/v0.22 >/dev/null 2>&1; then
    printf '[ERROR] Invalid release branch was accepted.\n' >&2
    exit 1
fi
if grep -Fq 'match[1]' scripts/release/check-pr-ready.sh; then
    printf '[ERROR] check-pr-ready.sh depends on mutable zsh regex captures.\n' >&2
    exit 1
fi

printf '[OK]    Version increment contract tests passed.\n'
