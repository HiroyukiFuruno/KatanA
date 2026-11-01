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

# Expected version and task gate from arguments
EXPECTED_VERSION=""
TASK_GATE_MODE=""
for argument in "$@"; do
    case "$argument" in
        --pr-bootstrap) TASK_GATE_MODE="pr-bootstrap" ;;
        --automated-release) TASK_GATE_MODE="automated-release" ;;
        --post-release-automated) TASK_GATE_MODE="post-release-automated" ;;
        --strict-document-evidence) TASK_GATE_MODE="strict" ;;
        --release-artifact-pending) TASK_GATE_MODE="release-artifact-pending" ;;
        --post-release-evidence) TASK_GATE_MODE="post-release-evidence" ;;
        --post-release-host) TASK_GATE_MODE="post-release-host" ;;
        --document-source) TASK_GATE_MODE="source" ;;
        --packaged-host) TASK_GATE_MODE="packaged-host" ;;
        --*) error "Unknown option: $argument"; exit 2 ;;
        *)
            if [[ -n "$EXPECTED_VERSION" ]]; then
                error "Expected at most one version argument."
                exit 2
            fi
            EXPECTED_VERSION="$argument"
            ;;
    esac
done
HAS_EXPECTED_VERSION=false
if [[ -n "$EXPECTED_VERSION" ]]; then
    HAS_EXPECTED_VERSION=true
    EXPECTED_VERSION="${EXPECTED_VERSION#v}" # Strip leading v
fi

# Helpers for checking file consistency
is_ci() { [[ "${GITHUB_ACTIONS:-false}" == "true" ]]; }

TASK_BRANCH_PATTERN='^release/v[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9][A-Za-z0-9._-]*)?-task-?[0-9A-Za-z]+(-fix)?$'
RELEASE_BRANCH_VERSION_SCRIPT='scripts/release/extract-release-branch-version.sh'
READINESS_MODE_SELECTOR='scripts/release/select-pr-readiness-mode.sh'

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
elif BRANCH_VERSION=$(bash "$RELEASE_BRANCH_VERSION_SCRIPT" "$CURRENT_BRANCH" 2>/dev/null); then
    # Task branches do not bump Cargo.toml until final release preparation.
    if [[ "$CURRENT_BRANCH" =~ $TASK_BRANCH_PATTERN ]]; then
        success "Task branch detected (${CURRENT_BRANCH}). Skipping final PR readiness checks."
        exit 0
    fi
    TARGET_VERSION="$BRANCH_VERSION"
    if [[ "$HAS_EXPECTED_VERSION" == "false" && ! is_ci && "$CUR_VERSION" != "$TARGET_VERSION" ]]; then
        warn "Release integration branch detected before final release preparation."
        warn "Skipping final PR readiness checks until an explicit version is provided."
        info "Run ./scripts/release/check-pr-ready.sh ${TARGET_VERSION} after release prep."
        exit 0
    fi
    info "Target version detected from branch: v${TARGET_VERSION}"
else
    TARGET_VERSION="$CUR_VERSION"
    if [[ "$CURRENT_BRANCH" == "master" ]]; then
        warn "Running on master without version argument. Using Cargo.toml version (v${TARGET_VERSION}) as reference."
        info "If you intend to release, please run: ./scripts/release/check-pr-ready.sh <version>"
    fi
fi

# WHY: target確定後にだけ厳密な公開証跡branchの自動検証modeを選択し、他branchのmodeを変更しない。
TASK_GATE_MODE=$(bash "$READINESS_MODE_SELECTOR" "$CURRENT_BRANCH" "$TARGET_VERSION" "$TASK_GATE_MODE")

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

# Check Cargo.lock sync. Keep this to dependency resolution only; compile/type checks
# run in the regular CI matrix and can fail for unrelated platform reasons.
LOCK_CHECK_OUTPUT=$(mktemp)
if ! cargo metadata --locked --format-version=1 >"$LOCK_CHECK_OUTPUT" 2>&1; then
    cat "$LOCK_CHECK_OUTPUT" >&2
    rm -f "$LOCK_CHECK_OUTPUT"
    error "Cargo.lock is not resolvable with Cargo.toml. Please run 'cargo update --workspace'."
    exit 1
fi
rm -f "$LOCK_CHECK_OUTPUT"
success "Cargo.lock is synced."

# 自動検証で公開する承認済み版だけを既定とし、手動証跡は明示指定に残す。
if [[ -z "$TASK_GATE_MODE" ]]; then
    if [[ "$TARGET_VERSION" == "0.22.42" ]]; then
        TASK_GATE_MODE="automated-release"
    else
        TASK_GATE_MODE="strict"
    fi
fi

# 5. Version increment or published-release guard
if [[ "$TARGET_VERSION" == "0.22.47" &&
      ( "$TASK_GATE_MODE" == "post-release-host" || "$TASK_GATE_MODE" == "post-release-automated" ||
        ( "$TASK_GATE_MODE" == "post-release-evidence" &&
          "$CURRENT_BRANCH" != "release/v0.22.47-public-release-evidence" ) ) ]]; then
    error "v0.22.47 post-release evidence requires the exact records branch."
    exit 1
fi
if [[ "$TARGET_VERSION" == "0.22.46" &&
      ( "$TASK_GATE_MODE" == "post-release-host" || "$TASK_GATE_MODE" == "post-release-automated" ||
        ( "$TASK_GATE_MODE" == "post-release-evidence" &&
          "$CURRENT_BRANCH" != "release/v0.22.46-public-release-evidence" ) ) ]]; then
    error "v0.22.46 post-release evidence requires the exact records branch."
    exit 1
fi
if [[ "$TASK_GATE_MODE" == "post-release-evidence" || "$TASK_GATE_MODE" == "post-release-host" || "$TASK_GATE_MODE" == "post-release-automated" ]]; then
    RELEASE_STATE=$(gh release view "v${TARGET_VERSION}" \
        --repo "${KATANA_RELEASE_REPO:-HiroyukiFuruno/KatanA}" \
        --json tagName,isDraft,isPrerelease \
        --jq '.tagName + " draft=" + (.isDraft | tostring) + " prerelease=" + (.isPrerelease | tostring)' 2>/dev/null) || {
        error "Published GitHub Release v${TARGET_VERSION} is required for post-release evidence."
        exit 1
    }
    if [[ "$RELEASE_STATE" != "v${TARGET_VERSION} draft=false prerelease=false" ]]; then
        error "v${TARGET_VERSION} is not a published stable GitHub Release: ${RELEASE_STATE}"
        exit 1
    fi
    success "Published stable GitHub Release v${TARGET_VERSION} is present."
else
    if ! ./scripts/release/check-version-increment.sh "$TARGET_VERSION"; then
        error "Version increment check failed for v${TARGET_VERSION}."
        exit 1
    fi
fi

# WHY: v43の公開後記録だけを公開sourceと実在するtaskfileへ結び付ける。
if [[ "$CURRENT_BRANCH" == "release/v0.22.43-public-release-evidence" &&
      "$TARGET_VERSION" == "0.22.43" && "$TASK_GATE_MODE" == "post-release-evidence" ]]; then
    python3 -B scripts/release/check-v043-postpublication-scope.py \
        --root "$PWD" --branch "$CURRENT_BRANCH" --version "$TARGET_VERSION" \
        --repository "${KATANA_RELEASE_REPO:-HiroyukiFuruno/KatanA}"
fi

# WHY: v44も公開sourceに固定し、製品を変更しない記録と整理だけを許可する。
if [[ "$CURRENT_BRANCH" == "release/v0.22.44-public-release-evidence" &&
      "$TARGET_VERSION" == "0.22.44" && "$TASK_GATE_MODE" == "post-release-evidence" ]]; then
    python3 -B scripts/release/check-v043-postpublication-scope.py \
        --root "$PWD" --branch "$CURRENT_BRANCH" --version "$TARGET_VERSION" \
        --repository "${KATANA_RELEASE_REPO:-HiroyukiFuruno/KatanA}"
fi

# WHY: 指定版の公開後記録を実tag/run/公開identity証跡へ結合し、製品sourceを変更させない。
if [[ ( "$CURRENT_BRANCH" == "release/v0.22.47-public-release-evidence" && "$TARGET_VERSION" == "0.22.47" ) ||
      ( "$CURRENT_BRANCH" == "release/v0.22.45-public-release-evidence" && "$TARGET_VERSION" == "0.22.45" ) ||
      ( "$CURRENT_BRANCH" == "release/v0.22.46-public-release-evidence" && "$TARGET_VERSION" == "0.22.46" ) ]] &&
      [[ "$TASK_GATE_MODE" == "post-release-evidence" ]]; then
    python3 -B scripts/release/future_publication_records.py \
        --root "$PWD" --branch "$CURRENT_BRANCH" --version "$TARGET_VERSION" \
        --repository "${KATANA_RELEASE_REPO:-HiroyukiFuruno/KatanA}"
fi

# 6. Branch naming vs Target Version for Release branches
if [[ "$CURRENT_BRANCH" =~ ^release/ ]]; then
    if ! BRANCH_VERSION=$(bash "$RELEASE_BRANCH_VERSION_SCRIPT" "$CURRENT_BRANCH" 2>/dev/null); then
         error "Branch name '$CURRENT_BRANCH' does not follow release/vX.Y.Z or release/vX.Y.Z-name format."
         exit 1
    fi

    if [[ "$BRANCH_VERSION" != "$TARGET_VERSION" ]]; then
        error "Branch version (v${BRANCH_VERSION}) does not match target version (v${TARGET_VERSION})."
        exit 1
    fi
    success "Branch version matches target version."
fi

# 7. Run preflight
if ! KATANA_OPENSPEC_TASK_GATE="$TASK_GATE_MODE" ./scripts/release/preflight.sh "$TARGET_VERSION"; then
    error "Preflight checks failed for v${TARGET_VERSION}."
    exit 1
fi

success "Mechanical pre-PR checks passed for v${TARGET_VERSION}."

success "Mechanical pre-PR checks passed."
