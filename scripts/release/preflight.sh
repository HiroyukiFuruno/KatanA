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

# 1. Version Increment Contract
info "1/8 Verifying version increment contract..."
bash scripts/release/test-version-increment.sh
success "Version increment contract is enforced."

# 2. Browser-equivalent HTML release contract
info "2/8 Verifying browser-equivalent HTML release contract..."
bash scripts/release/test-html-browser-release-contract.sh
if [[ "$VERSION" == "0.22.35" ]]; then
    scripts/release/check-html-browser-release-contract.sh "$VERSION"
fi
success "Browser-equivalent HTML release contract is enforced."

# 3. Release Asset Inspector Validation
info "3/8 Verifying release asset inspector..."
bash scripts/dev/test-inspect-release-asset.sh
success "Release asset inspector preserves bundle paths."

# 4-5. Artifact Naming Validation
info "4/8 Verifying Cargo.toml version..."
CARGO_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
if [[ "$CARGO_VERSION" != "$VERSION" ]]; then
    error "Cargo.toml version ($CARGO_VERSION) does not match target release version ($VERSION)."
    exit 1
fi
success "Cargo.toml version matches."

info "5/8 Verifying Info.plist version..."
PLIST_VERSION=$(awk '/CFBundleShortVersionString/{getline; gsub(/.*<string>v?|<\/string>.*/, ""); print}' crates/katana-ui/Info.plist | xargs)
if [[ "$PLIST_VERSION" != "$VERSION" ]]; then
    error "Info.plist CFBundleShortVersionString ($PLIST_VERSION) does not match target release version ($VERSION)."
    exit 1
fi
success "Info.plist version matches."

# 6. CHANGELOG Validation
info "6/8 Validating CHANGELOG via AST Linter..."
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

# 7. Linuxbrew Formula Validation
info "7/8 Verifying Linuxbrew formula contract..."
scripts/release/check-linuxbrew-formula-contract.sh

# 8. OpenSpec Validation
info "8/8 Validating OpenSpec task completion..."
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
