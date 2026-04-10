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
    # PR from release branch: release/v0.18.5 -> 0.18.5
    BARE="${PR_HEAD_REF#release/v}"
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
