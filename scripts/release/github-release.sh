#!/bin/bash
set -euo pipefail

# Usage: ./scripts/release/github-release.sh <tag> <commit_sha> <notes_file> <artifacts_dir> <force_flag>
TAG=$1
COMMIT_SHA=$2
NOTES_FILE=$3
ARTIFACTS_DIR=$4
FORCE_FLAG=${5:-"false"}

if [[ -z "$TAG" || -z "$COMMIT_SHA" || -z "$NOTES_FILE" || -z "$ARTIFACTS_DIR" ]]; then
    echo "Usage: $0 <tag> <commit_sha> <notes_file> <artifacts_dir> [force_flag]"
    exit 1
fi

CLOBBER=""
if [[ "$FORCE_FLAG" == "true" ]]; then
    CLOBBER="--clobber"
    echo "⚠️ Force mode enabled: using --clobber"
fi

echo "🚀 Creating/Updating GitHub Release ${TAG}..."

# We use 'v' prefix for release titles
RELEASE_TITLE="KatanA Desktop ${TAG}"

gh release create "${TAG}" \
    ${CLOBBER} \
    --title "${RELEASE_TITLE}" \
    --notes-file "${NOTES_FILE}" \
    --target "${COMMIT_SHA}" \
    "${ARTIFACTS_DIR}"/*

echo "✅ GitHub Release ${TAG} successfully published with artifacts."
