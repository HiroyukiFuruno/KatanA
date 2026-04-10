#!/bin/bash
set -e

# Usage: ./scripts/release/bump-version.sh <target_version>
TARGET_VERSION=$1

if [ -z "$TARGET_VERSION" ]; then
    echo "Error: TARGET_VERSION is required."
    exit 1
fi

CURRENT_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')

echo "Target Release: v${TARGET_VERSION}"
echo "Current Cargo.toml: v${CURRENT_VERSION}"

if [ "${TARGET_VERSION}" = "${CURRENT_VERSION}" ]; then
    echo "✅ Version already matches in Cargo.toml. Skipping bump logic."
    exit 0
fi

echo "🚀 Bumping workspace version to ${TARGET_VERSION}..."

# Update Cargo.toml
perl -pi -e 's/^version = ".+"/version = "'"${TARGET_VERSION}"'"/' Cargo.toml

# Update Cargo.lock
cargo update --workspace >/dev/null 2>&1 || true

# Update Info.plist surgically
# This targets ONLY the CFBundleShortVersionString value
INFO_PLIST="crates/katana-ui/Info.plist"
if [ -f "$INFO_PLIST" ]; then
    perl -i -0pe 's/(<key>CFBundleShortVersionString<\/key>\s*<string>).*?(<\/string>)/$1v'"${TARGET_VERSION}"'$2/' "$INFO_PLIST"
fi

# Git operations
git config --global user.name "github-actions[bot]"
git config --global user.email "41898282+github-actions[bot]@users.noreply.github.com"

if ! git diff --quiet; then
    git add Cargo.toml Cargo.lock "$INFO_PLIST"
    git commit -m "chore: Release v${TARGET_VERSION} [skip ci]"
    
    BRANCH_NAME=$(git rev-parse --abbrev-ref HEAD)
    echo "Pushing to branch: $BRANCH_NAME"
    if git push origin "HEAD:$BRANCH_NAME"; then
        echo "✅ Successfully pushed version bump."
    else
        echo "⚠️ Push failed (likely due to branch protection)."
        echo "Continuing with current local state as version is already correct."
    fi
else
    echo "No changes detected after attempted bump."
fi
