#!/bin/zsh
set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

error() { printf "${RED}[ERROR]${RESET} %s\n" "$*" >&2; }
success() { printf "${GREEN}[OK]${RESET}    %s\n" "$*"; }
warn() { printf "${YELLOW}[WARN]${RESET}  %s\n" "$*"; }
info() { printf "${CYAN}[INFO]${RESET}  %s\n" "$*"; }

header() { printf "\n${BOLD}${CYAN}==> %s${RESET}\n" "$*"; }

# Expected version from argument
EXPECTED_VERSION=${1:-}
if [[ -n "$EXPECTED_VERSION" ]]; then
    EXPECTED_VERSION="${EXPECTED_VERSION#v}" # Strip leading v
fi

# Helpers for checking file consistency
is_ci() { [[ "${GITHUB_ACTIONS:-false}" == "true" ]]; }

# 1. Check for uncommitted changes (Local only)
if ! is_ci; then
    CRITICAL_FILES=("Cargo.toml" "Cargo.lock" "crates/katana-ui/Info.plist" "CHANGELOG.md" "CHANGELOG.ja.md")
    UNSTAGED_FILES=$(git diff --name-only)

    for file in "${CRITICAL_FILES[@]}"; do
        if [[ -f "$file" ]] && echo "$UNSTAGED_FILES" | grep -q "^$file$"; then
            error "$file has unstaged changes. Please commit them before creating a PR."
            exit 1
        fi
    done
fi

# 2. Extract Version Info
# Current version in Cargo.toml
CUR_VERSION=$(grep '^version =' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
success "Current version in Cargo.toml: v${CUR_VERSION}"

# 3. Determine Target Version
if is_ci; then
    CURRENT_BRANCH="${GITHUB_HEAD_REF:-$(git branch --show-current)}"
else
    CURRENT_BRANCH=$(git branch --show-current)
fi

# Strategy: 1. Arg, 2. Branch Name (if release/*), 3. Cargo.toml
TARGET_VERSION=""
if [[ -n "$EXPECTED_VERSION" ]]; then
    TARGET_VERSION="$EXPECTED_VERSION"
    info "Target version set by argument: v${TARGET_VERSION}"
elif [[ "$CURRENT_BRANCH" =~ ^release/v([0-9]+\.[0-9]+\.[0-9]+)(-task-[0-9]+(-fix)?)?$ ]]; then
    # Skip full checks for task branches because Cargo.toml version bump 
    # happens at final release preparation.
    if [[ -n "${match[2]}" ]]; then
        success "Task branch detected (${CURRENT_BRANCH}). Skipping final PR readiness checks."
        exit 0
    fi
    TARGET_VERSION="${match[1]}"
    info "Target version detected from branch: v${TARGET_VERSION}"
else
    TARGET_VERSION="$CUR_VERSION"
    if [[ "$CURRENT_BRANCH" == "master" ]]; then
        warn "Running on master without version argument. Using Cargo.toml version (v${TARGET_VERSION}) as reference."
        info "If you intend to release, please run: ./scripts/release/check-pr-ready.sh <version>"
    fi
fi

# 4. Check for Consistency against TARGET_VERSION
# Check Cargo.toml matches TARGET_VERSION
if [[ "$CUR_VERSION" != "$TARGET_VERSION" ]]; then
    error "Cargo.toml version (v${CUR_VERSION}) does not match target version (v${TARGET_VERSION})."
    error "Please run: ./scripts/release/bump-version.sh $TARGET_VERSION"
    exit 1
fi

# Check Info.plist consistency
INFO_PLIST="crates/katana-ui/Info.plist"
if [[ -f "$INFO_PLIST" ]]; then
    PLIST_VERSION=$(grep -A 1 "CFBundleShortVersionString" "$INFO_PLIST" | grep "string" | sed 's/.*<string>\(.*\)<\/string>.*/\1/')
    if [[ "$PLIST_VERSION" != "v${TARGET_VERSION}" ]]; then
        error "$INFO_PLIST version ($PLIST_VERSION) does not match target version (v${TARGET_VERSION})."
        error "Please run: ./scripts/release/bump-version.sh $TARGET_VERSION"
        exit 1
    fi
    success "$INFO_PLIST is consistent with v${TARGET_VERSION}."
fi

# Check Cargo.lock sync
if ! cargo check --locked >/dev/null 2>&1; then
    error "Cargo.lock is out of sync with Cargo.toml. Please run 'cargo update --workspace'."
    exit 1
fi
success "Cargo.lock is synced."

# 5. Branch naming vs Target Version for Release branches
if [[ "$CURRENT_BRANCH" =~ ^release/ ]]; then
    if [[ ! "$CURRENT_BRANCH" =~ ^release/v[0-9]+\.[0-9]+\.[0-9]+(-task-[0-9]+(-fix)?)?$ ]]; then
         error "Branch name '$CURRENT_BRANCH' does not follow release/vX.Y.Z or release/vX.Y.Z-task-N format."
         exit 1
    fi
    
    BRANCH_VERSION="${CURRENT_BRANCH#release/v}"
    BRANCH_VERSION="${BRANCH_VERSION%-task*}" # strip the task suffix
    if [[ "$BRANCH_VERSION" != "$TARGET_VERSION" ]]; then
        error "Branch version (v${BRANCH_VERSION}) does not match target version (v${TARGET_VERSION})."
        exit 1
    fi
    success "Branch version matches target version."
fi

# 6. Run preflight
if ! ./scripts/release/preflight.sh "$TARGET_VERSION"; then
    error "Preflight checks failed for v${TARGET_VERSION}."
    exit 1
fi

success "Mechanical pre-PR checks passed for v${TARGET_VERSION}."

success "Mechanical pre-PR checks passed."
