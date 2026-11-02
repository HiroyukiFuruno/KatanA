#!/bin/bash
set -euo pipefail

# Usage: ./scripts/release/bump-version.sh <target_version>
TARGET_VERSION=${1:-}

if [ -z "$TARGET_VERSION" ]; then
    echo "Error: TARGET_VERSION is required."
    exit 1
fi
if [[ ! "$TARGET_VERSION" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]; then
    echo "Error: TARGET_VERSION must be a full numeric version." >&2
    exit 1
fi
if [[ "${GITHUB_ACTIONS:-false}" != "true" && ! "${KATANA_RELEASE_ISSUE:-}" =~ ^[1-9][0-9]*$ ]]; then
    echo "Error: set KATANA_RELEASE_ISSUE to the release Issue number before running the local release." >&2
    exit 1
fi

CURRENT_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')

if [[ "${GITHUB_ACTIONS:-false}" == "true" ]]; then
    if ! git diff --quiet || ! git diff --cached --quiet || [[ -n "$(git ls-files --others --exclude-standard)" ]]; then
        echo "Error: version bump requires a clean checkout." >&2
        exit 1
    fi
fi

echo "Target Release: v${TARGET_VERSION}"
echo "Current Cargo.toml: v${CURRENT_VERSION}"

if [ "${TARGET_VERSION}" != "${CURRENT_VERSION}" ]; then
    echo "🚀 Bumping workspace version to ${TARGET_VERSION}..."
    # Update Cargo.toml
    perl -pi -e 's/^version = ".+"/version = "'"${TARGET_VERSION}"'"/' Cargo.toml
    # WHY: lockfileは実際に版を変更した時だけ再解決し、失敗を隠さない。
    cargo update --workspace
fi

# Update Info.plist surgically
# This targets ONLY the CFBundleShortVersionString value
INFO_PLIST="crates/katana-ui/Info.plist"
if [ -f "$INFO_PLIST" ]; then
    perl -i -0pe 's/(<key>CFBundleShortVersionString<\/key>\s*<string>).*?(<\/string>)/$1v'"${TARGET_VERSION}"'$2/' "$INFO_PLIST"
fi

# Git operations
if [ "${GITHUB_ACTIONS:-false}" = "true" ]; then
    echo "🤖 Running in GitHub Actions. Using 'gh api' for verified commit..."
    
    BRANCH_NAME="${GITHUB_REF_NAME:-}"
    if ! git diff --quiet; then
        if [[ "${GITHUB_REPOSITORY:-}" != "HiroyukiFuruno/KatanA" || "${GITHUB_EVENT_NAME:-}" != "workflow_dispatch" || "${GITHUB_REF:-}" != "refs/heads/master" || "$BRANCH_NAME" != "master" || ! "${GITHUB_RUN_ID:-}" =~ ^[1-9][0-9]*$ || ! "${GITHUB_RUN_ATTEMPT:-}" =~ ^[1-9][0-9]*$ ]]; then
            echo "Error: generated version commits require a manual master dispatch." >&2
            exit 1
        fi
        ORIGINAL_COMMIT=$(git rev-parse HEAD)
        REMOTE_COMMIT=$(gh api "repos/HiroyukiFuruno/KatanA/git/ref/heads/master" --jq '.object.sha')
        if [[ "$ORIGINAL_COMMIT" != "$REMOTE_COMMIT" || "$ORIGINAL_COMMIT" != "${GITHUB_SHA:-}" ]]; then
            echo "Error: master changed since this run was triggered." >&2
            exit 1
        fi
        python3 -B - "$ORIGINAL_COMMIT" "v${TARGET_VERSION}" <<'PY'
from pathlib import Path
import subprocess
import sys
sys.path.insert(0, "scripts/release")
from manual_bump_transition import FILES, expected_files
trigger, version = sys.argv[1:]
expected = expected_files(Path.cwd(), trigger, version)
paths = subprocess.run(["git", "diff", "--name-only", "--no-renames", "-z"],
                       check=True, capture_output=True).stdout.decode().split("\0")
if set(filter(None, paths)) != set(FILES):
    raise SystemExit("manual bump must change exactly the three version files")
if any(Path(path).is_symlink() or Path(path).read_text() != content for path, content in expected.items()):
    raise SystemExit("manual bump contains changes beyond the expected versions")
PY
        # WHY: 同時更新やAPI失敗で3ファイルが一部だけmasterへ反映されることを防ぐ。
        GENERATED_COMMIT=$(python3 -B scripts/release/manual_bump_graphql.py "$ORIGINAL_COMMIT" "v${TARGET_VERSION}")
        if [[ "$(gh api "repos/HiroyukiFuruno/KatanA/commits/$GENERATED_COMMIT" --jq '.commit.verification.verified')" != "true" ]]; then
            echo "Error: generated version commit is not verified by GitHub." >&2
            exit 1
        fi
        git fetch origin refs/heads/master
        if [[ "$(git rev-parse FETCH_HEAD)" != "$GENERATED_COMMIT" ]]; then
            echo "Error: master changed after the atomic version commit." >&2
            exit 1
        fi
        python3 -B - "$ORIGINAL_COMMIT" "$GENERATED_COMMIT" "v${TARGET_VERSION}" <<'PY'
import os
from pathlib import Path
import sys
sys.path.insert(0, "scripts/release")
from manual_bump_transition import SCHEMA, WORKFLOW, verify_transition
trigger, source, version = sys.argv[1:]
run_id, attempt = int(os.environ["GITHUB_RUN_ID"]), int(os.environ["GITHUB_RUN_ATTEMPT"])
record = {"schema": SCHEMA, "repository": os.environ["GITHUB_REPOSITORY"], "workflow": WORKFLOW,
          "run_id": run_id, "run_attempt": attempt, "event": "workflow_dispatch", "branch": "master",
          "trigger": trigger, "source": source, "version": version}
verify_transition(Path.cwd(), record, trigger, source, version, run_id, attempt)
PY
        for path in Cargo.toml Cargo.lock "$INFO_PLIST"; do
            if ! cmp -s "$path" <(git show "$GENERATED_COMMIT:$path"); then
                echo "Error: generated commit differs from the checked files: $path" >&2
                exit 1
            fi
        done
        git update-ref HEAD "$GENERATED_COMMIT" "$ORIGINAL_COMMIT"
        git add -- Cargo.toml Cargo.lock "$INFO_PLIST"
        git diff --quiet && git diff --cached --quiet
    else
        echo "No changes detected."
    fi
else
    # Local development: commit only if version-related files changed
    if ! git diff --quiet Cargo.toml Cargo.lock "$INFO_PLIST"; then
        git add Cargo.toml Cargo.lock "$INFO_PLIST"
        # Local commits must keep the developer's configured identity so GitHub can match the signature.
        git commit -S -m "chore: v${TARGET_VERSION}のリリース準備 (#${KATANA_RELEASE_ISSUE})" -- Cargo.toml Cargo.lock "$INFO_PLIST"
        
        echo "✅ Version bump committed locally."
        echo "   (Note: Use 'git push' manually if the branch is not protected)"
    else
        echo "No changes detected after attempted bump."
    fi
fi
