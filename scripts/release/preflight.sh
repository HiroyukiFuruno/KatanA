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
TASK_GATE_MODE=${KATANA_OPENSPEC_TASK_GATE:-strict}

if [[ -z "$VERSION" ]]; then
    error "VERSION is required. Usage: scripts/release/preflight.sh x.y.z"
    exit 1
fi

# Strip leading 'v' if present
VERSION="${VERSION#v}"

header "Preflight checks for v${VERSION}"

# 1. Version Increment Contract
info "1/11 Verifying version increment contract..."
bash scripts/release/test-version-increment.sh
success "Version increment contract is enforced."

# 2. Post-merge CI gate contract
info "2/12 Verifying post-merge CI gate contract..."
bash scripts/release/test-version-bump-ci-gate-contract.sh
success "Post-merge CI gate covers the full three-platform release window."

# 3. Browser-equivalent HTML release contract
info "3/12 Verifying browser-equivalent HTML release contract..."
bash scripts/release/test-html-browser-release-contract.sh
if [[ "$VERSION" == "0.22.38" ]]; then
    scripts/release/check-html-browser-release-contract.sh "$VERSION"
fi
success "Browser-equivalent HTML release contract is enforced."

# 4. Multi-format document release contract
info "4/12 Verifying multi-format document release contract..."
python3 scripts/release/check-multi-format-document-contract.py --self-test
if [[ "$VERSION" == "0.22.39" ]]; then
    python3 scripts/release/check-multi-format-document-contract.py "$VERSION"
fi
success "Multi-format document ownership and packaging contract is enforced."

# 5. Dependency and source supply chain
info "5/12 Verifying dependency advisories, licenses, and sources..."
if ! command -v cargo-deny >/dev/null 2>&1; then
    error "cargo-deny is required. Install cargo-deny 0.20.2 before release preflight."
    exit 127
fi
cargo deny check --hide-inclusion-graph
success "Dependency advisories, licenses, and sources satisfy policy."

# 6. Release Asset Inspector Validation
info "6/12 Verifying release asset inspector..."
bash scripts/dev/test-inspect-release-asset.sh
success "Release asset inspector preserves bundle paths."

# 7. macOS Coverage Linker Concurrency
info "7/12 Verifying macOS coverage linker concurrency..."
bash scripts/release/test-macos-coverage-contract.sh
bash scripts/release/check-macos-coverage-contract.sh
python3 scripts/ci/check-document-surface-coverage.py --self-test
success "macOS coverage linker concurrency is constrained."

# 8. Artifact Naming Validation
info "8/12 Verifying Cargo.toml version..."
CARGO_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
if [[ "$CARGO_VERSION" != "$VERSION" ]]; then
    error "Cargo.toml version ($CARGO_VERSION) does not match target release version ($VERSION)."
    exit 1
fi
success "Cargo.toml version matches."

info "9/12 Verifying Info.plist version..."
PLIST_VERSION=$(awk '/CFBundleShortVersionString/{getline; gsub(/.*<string>v?|<\/string>.*/, ""); print}' crates/katana-ui/Info.plist | xargs)
if [[ "$PLIST_VERSION" != "$VERSION" ]]; then
    error "Info.plist CFBundleShortVersionString ($PLIST_VERSION) does not match target release version ($VERSION)."
    exit 1
fi
success "Info.plist version matches."

# 9. CHANGELOG Validation
info "10/12 Validating CHANGELOG via AST Linter..."
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

# 10. Linuxbrew Formula Validation
info "11/12 Verifying Linuxbrew formula contract..."
scripts/release/check-linuxbrew-formula-contract.sh

# 11. OpenSpec Validation
info "12/12 Validating OpenSpec task completion..."
VERSION_DASHED=$(echo "$VERSION" | tr '.' '-')
for CHANGE_DIR in openspec/changes/v${VERSION_DASHED}-*(N); do
    if [[ -d "$CHANGE_DIR" ]]; then
        CHANGE_NAME=$(basename "$CHANGE_DIR")
        if [[ -f "$CHANGE_DIR/tasks.md" ]]; then
            TASK_GATE_ARGS=()
            if [[ "$TASK_GATE_MODE" == "pr-bootstrap" && "$CHANGE_NAME" == "v0-22-38-multi-format-document-viewer" ]]; then
                TASK_GATE_ARGS=(
                    --allow 5.7
                    --allow 7.5
                    --allow 7.6
                    --allow 7.7
                    --allow 8.2
                    --allow 8.4
                )
            elif [[ "$TASK_GATE_MODE" == "pr-bootstrap" && "$CHANGE_NAME" == "v0-22-39-office2pdf-official-intake" ]]; then
                TASK_GATE_ARGS=(
                    --allow 3.1
                    --allow 3.2
                    --allow 3.3
                    --allow 4.1
                )
            elif [[ "$TASK_GATE_MODE" != "strict" ]]; then
                error "Unsupported OpenSpec task gate mode: $TASK_GATE_MODE"
                exit 2
            fi
            if ! python3 scripts/release/check-openspec-task-completion.py \
                "$CHANGE_DIR/tasks.md" "${TASK_GATE_ARGS[@]}"; then
                error "OpenSpec change '$CHANGE_NAME' has incomplete tasks outside the permitted PR evidence phase."
                exit 1
            fi
            success "OpenSpec change '$CHANGE_NAME' satisfies the $TASK_GATE_MODE task gate."
        fi
    fi
done

success "All preflight checks passed for v${VERSION}!"
