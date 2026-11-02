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
    if [[ ! "$PR_HEAD_REF" =~ ^release/v([0-9]+\.[0-9]+\.[0-9]+)(-[A-Za-z0-9][A-Za-z0-9._-]*)?$ ]]; then
        echo "Invalid release branch." >&2
        exit 1
    fi
    BARE="${BASH_REMATCH[1]}"
    VERSION="v${BARE}"
elif [ "${EVENT_NAME}" = "push" ]; then
    if [[ "$GITHUB_REF" != refs/tags/v* ]]; then
        echo "Invalid release tag." >&2
        exit 1
    fi
    TAG="${GITHUB_REF#refs/tags/}"
    BARE="${TAG#v}"
    VERSION="${TAG}"
else
    echo "Unsupported release event." >&2
    exit 1
fi

if [[ ! "$BARE" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]; then
    echo "Invalid release version." >&2
    exit 1
fi

echo "version=${VERSION}" >> "${GITHUB_OUTPUT}"
echo "version_bare=${BARE}" >> "${GITHUB_OUTPUT}"

echo "Detected Version: ${VERSION} (bare: ${BARE})"
