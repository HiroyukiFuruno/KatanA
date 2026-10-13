#!/bin/bash
set -euo pipefail

# Usage: ./scripts/release/get-version.sh <event_name> <input_version> <pr_head_ref> <github_ref> <output_file>
EVENT_NAME=$1
INPUT_VERSION=$2
PR_HEAD_REF=$3
GITHUB_REF=$4
GITHUB_OUTPUT=${5:-"/dev/stdout"}

VERSION=""
BARE=""

if [ "${EVENT_NAME}" = "workflow_dispatch" ]; then
    # Manual trigger
    BARE="${INPUT_VERSION#v}"
    VERSION="v${BARE}"
elif [ "${EVENT_NAME}" = "pull_request" ]; then
    # PR from release branch: release/v0.18.5            -> 0.18.5
    # PR from release branch: release/v0.22.22-something -> 0.22.22
    BARE="${PR_HEAD_REF#release/v}"
    # WHY: Branch names sometimes carry a human-readable descriptor suffix
    # (e.g. release/v0.22.22-headless-process-enforcement). check-pr-ready.sh's
    # RELEASE_BRANCH_PATTERN already treats that suffix as optional and compares
    # only the leading X.Y.Z against Cargo.toml. Mirror that contract here so the
    # version_bare output passed downstream is the canonical X.Y.Z form.
    # NOTE: This intentionally drops pre-release tokens such as `-rc.1`. KatanA
    # does not ship pre-releases today; if that changes, this strip must be
    # refined to preserve recognised pre-release suffixes.
    if [[ "$BARE" =~ ^([0-9]+\.[0-9]+\.[0-9]+) ]]; then
        BARE="${BASH_REMATCH[1]}"
    fi
    VERSION="v${BARE}"
else
    # Tag push or other
    TAG="${GITHUB_REF#refs/tags/}"
    BARE="${TAG#v}"
    VERSION="${TAG}"
fi

echo "version=${VERSION}" >> "${GITHUB_OUTPUT}"
echo "version_bare=${BARE}" >> "${GITHUB_OUTPUT}"

echo "Detected Version: ${VERSION} (bare: ${BARE})"
