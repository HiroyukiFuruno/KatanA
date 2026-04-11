#!/bin/zsh
set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RESET='\033[0m'

error() { echo "${RED}[ERROR]${RESET} $*" >&2; }
success() { echo "${GREEN}[OK]${RESET}    $*"; }
warn() { echo "${YELLOW}[WARN]${RESET}  $*"; }

header() { echo "\n${BOLD}${CYAN}==> $*${RESET}"; }

# Check for staged/unstaged changes in critical files
STAGED_TOTAL=$(git diff --cached --name-only | wc -l | xargs)
UNSTAGED_TOTAL=$(git diff --name-only | wc -l | xargs)

# 1. Check for uncommitted changes in critical files
CRITICAL_FILES=("Cargo.toml" "Cargo.lock" "rustfmt.toml" "CHANGELOG.md" "CHANGELOG.ja.md")
UNSTAGED_FILES=$(git diff --name-only)

for file in "${CRITICAL_FILES[@]}"; do
    if [[ -f "$file" ]] && echo "$UNSTAGED_FILES" | grep -q "^$file$"; then
        error "$file has unstaged changes. Please commit them before creating a PR."
        exit 1
    fi
done

# 2. Check for Cargo.toml vs Cargo.lock sync
if git diff --cached --name-only | grep -q "^Cargo.toml$"; then
    if ! git diff --cached --name-only | grep -q "^Cargo.lock$"; then
        warn "Cargo.toml is staged but Cargo.lock is not. Verifying sync..."
        if ! cargo check --locked >/dev/null 2>&1; then
            error "Cargo.lock is out of sync with Cargo.toml. Please run 'cargo check' and stage Cargo.lock."
            exit 1
        fi
    fi
fi

# 3. Branch naming and Version Consistency
CURRENT_BRANCH=$(git branch --show-current)

# If it's a release-related branch (starts with release/ or hotfix/ and involves version bump)
IS_RELEASE_BRANCH=0
if [[ "$CURRENT_BRANCH" =~ ^release/ ]]; then
    IS_RELEASE_BRANCH=1
fi

# If Cargo.toml version changed in staged changes
if git diff --cached Cargo.toml | grep -q "^\+version ="; then
    NEW_VERSION=$(git diff --cached Cargo.toml | grep "^\+version =" | sed 's/.*"\(.*\)"/\1/')
    success "Detected version bump to v${NEW_VERSION}."
    
    # 4. Enforce branch name for release version bumps
    if [[ "$IS_RELEASE_BRANCH" -eq 0 && ! "$CURRENT_BRANCH" =~ ^hotfix/ ]]; then
         error "Version bump detected on branch '$CURRENT_BRANCH'. Release PRs must be on a 'release/vX.Y.Z' or 'hotfix/...' branch."
         exit 1
    fi

    # 5. Run preflight
    if ! ./scripts/release/preflight.sh "$NEW_VERSION"; then
        error "Preflight checks failed for v${NEW_VERSION}."
        exit 1
    fi
else
    # Even if no version bump, if it's a release branch, it should follow the pattern
    if [[ "$IS_RELEASE_BRANCH" -eq 1 ]]; then
        if [[ ! "$CURRENT_BRANCH" =~ ^release/v[0-9]+\.[0-9]+\.[0-9]+ ]]; then
             error "Branch name '$CURRENT_BRANCH' does not follow release/vX.Y.Z format (required for release/ branches)."
             exit 1
        fi
        VERSION="${CURRENT_BRANCH#release/v}"
        if ! ./scripts/release/preflight.sh "$VERSION"; then
            error "Preflight checks failed for branch version."
            exit 1
        fi
    fi
fi

success "Mechanical pre-PR checks passed."
