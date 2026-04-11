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

if [ "${TARGET_VERSION}" != "${CURRENT_VERSION}" ]; then
    echo "🚀 Bumping workspace version to ${TARGET_VERSION}..."
    # Update Cargo.toml
    perl -pi -e 's/^version = ".+"/version = "'"${TARGET_VERSION}"'"/' Cargo.toml
fi

# Update Cargo.lock
cargo update --workspace >/dev/null 2>&1 || true

# Update Info.plist surgically
# This targets ONLY the CFBundleShortVersionString value
INFO_PLIST="crates/katana-ui/Info.plist"
if [ -f "$INFO_PLIST" ]; then
    perl -i -0pe 's/(<key>CFBundleShortVersionString<\/key>\s*<string>).*?(<\/string>)/$1v'"${TARGET_VERSION}"'$2/' "$INFO_PLIST"
fi

# Git operations
if [ "${GITHUB_ACTIONS:-false}" = "true" ]; then
    echo "🤖 Running in GitHub Actions. Using 'gh api' for verified commit..."
    
    BRANCH_NAME=$(git rev-parse --abbrev-ref HEAD)
    COMMIT_MSG="chore: Release v${TARGET_VERSION} [skip ci]"
    
    # Helper to update a file via GitHub API
    update_file() {
        local path=$1
        if [ ! -f "$path" ]; then return; fi
        
        echo "Updating $path via API..."
        local content_b64
        if [[ "$OSTYPE" == "darwin"* ]]; then
            content_b64=$(base64 "$path")
        else
            content_b64=$(base64 -w0 "$path")
        fi
        
        local sha=$(gh api "repos/:owner/:repo/contents/$path?ref=$BRANCH_NAME" -q '.sha' 2>/dev/null || echo "")
        
        if [ -n "$sha" ]; then
            gh api --method PUT "repos/:owner/:repo/contents/$path" \
                -f message="$COMMIT_MSG" \
                -f content="$content_b64" \
                -f sha="$sha" \
                -f branch="$BRANCH_NAME" >/dev/null
        else
            # New file
             gh api --method PUT "repos/:owner/:repo/contents/$path" \
                -f message="$COMMIT_MSG" \
                -f content="$content_b64" \
                -f branch="$BRANCH_NAME" >/dev/null
        fi
    }

    if ! git diff --quiet; then
        # Updating files one by one (this creates multiple commits, but they are all Verified)
        # Note: In a release branch, this is acceptable for consistency.
        update_file "Cargo.toml"
        update_file "Cargo.lock"
        update_file "$INFO_PLIST"
        echo "✅ Successfully updated files via GitHub API (Verified)."
        
        # Pull the changes back to the runner to keep local state synced (optional but good)
        git pull origin "$BRANCH_NAME"
    else
        echo "No changes detected."
    fi
else
    # Local development: use standard git (likely signed by user)
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
fi
