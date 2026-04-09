#!/bin/zsh
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

info()    { echo "${CYAN}[INFO]${RESET}  $*"; }
success() { echo "${GREEN}[OK]${RESET}    $*"; }
error()   { echo "${RED}[ERROR]${RESET} $*" >&2; }
header()  { echo "\n${BOLD}${CYAN}==> $*${RESET}"; }

VERSION=${1:-}

if [[ -z "$VERSION" ]]; then
    error "VERSION is required. Usage: scripts/release/preflight.sh x.y.z"
    exit 1
fi

# Strip leading 'v' if present
VERSION="${VERSION#v}"

header "Preflight checks for v${VERSION}"

# 1. Artifact Naming Validation
info "1/4 Verifying Cargo.toml version..."
CARGO_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
if [[ "$CARGO_VERSION" != "$VERSION" ]]; then
    error "Cargo.toml version ($CARGO_VERSION) does not match target release version ($VERSION)."
    exit 1
fi
success "Cargo.toml version matches."

info "2/4 Verifying Info.plist version..."
PLIST_VERSION=$(awk '/CFBundleShortVersionString/{getline; gsub(/.*<string>v?|<\/string>.*/, ""); print}' crates/katana-ui/Info.plist | xargs)
if [[ "$PLIST_VERSION" != "$VERSION" ]]; then
    error "Info.plist CFBundleShortVersionString ($PLIST_VERSION) does not match target release version ($VERSION)."
    exit 1
fi
success "Info.plist version matches."

# 2. CHANGELOG Validation
info "3/4 Validating CHANGELOG via AST Linter..."
if ! cargo test -p katana-linter --test ast_linter ast_linter_changelog_contains_current_workspace_version -q >/dev/null 2>&1; then
    error "AST Linter failed: Version v${VERSION} not found in CHANGELOG.md."
    exit 1
fi
success "CHANGELOG.md contains notes for v${VERSION}."

if ! grep -q "^## \[${VERSION}\]" CHANGELOG.ja.md; then
    error "Version v${VERSION} not found in CHANGELOG.ja.md."
    exit 1
fi
success "CHANGELOG.ja.md contains notes for v${VERSION}."

# 3. OpenSpec Validation
info "4/4 Validating OpenSpec task completion..."
VERSION_DASHED=$(echo "$VERSION" | tr '.' '-')
for CHANGE_DIR in openspec/changes/v${VERSION_DASHED}-*(N); do
    if [[ -d "$CHANGE_DIR" ]]; then
        CHANGE_NAME=$(basename "$CHANGE_DIR")
        if [[ -f "$CHANGE_DIR/tasks.md" ]]; then
            if grep -E '^\s*-\s*\[(\s|\/)\]' "$CHANGE_DIR/tasks.md" >/dev/null 2>&1; then
                error "OpenSpec change '$CHANGE_NAME' has incomplete tasks."
                error "Please complete all tasks (all done) or rename the change directory before releasing."
                exit 1
            fi
        fi
        success "OpenSpec change '$CHANGE_NAME' is fully complete."
    fi
done

success "All preflight checks passed for v${VERSION}!"
