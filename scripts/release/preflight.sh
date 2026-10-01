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
info "1/13 Verifying version increment contract..."
bash scripts/release/test-version-increment.sh
success "Version increment contract is enforced."

# 2. Post-merge CI gate contract
info "2/13 Verifying post-merge CI gate contract..."
bash scripts/release/test-version-bump-ci-gate-contract.sh
success "Post-merge CI gate covers the full three-platform release window."

# 3. Browser-equivalent HTML release contract
info "3/13 Verifying browser-equivalent HTML release contract..."
bash scripts/release/test-html-browser-release-contract.sh
if [[ "$VERSION" == "0.22.38" ]]; then
    scripts/release/check-html-browser-release-contract.sh "$VERSION"
fi
success "Browser-equivalent HTML release contract is enforced."

# 4. Multi-format document release contract
info "4/13 Verifying multi-format document release contract..."
python3 scripts/release/check-multi-format-document-contract.py --self-test
python3 scripts/screenshot/test_generate_data_descriptor_docx.py
if [[ "$VERSION" == "0.22.41" ]]; then
    python3 scripts/release/check-multi-format-document-contract.py "$VERSION"
fi
success "Multi-format document ownership and packaging contract is enforced."

# 5. Published renderer dependency graph
info "5/13 Verifying published renderer dependency graph..."
python3 scripts/release/test-render-dependency-contract.py
python3 scripts/release/check-render-dependency-contract.py
success "Renderer dependencies are registry-published, exact, and use one V8 runtime."

# 6. Dependency and source supply chain
info "6/13 Verifying dependency advisories, licenses, and sources..."
if ! command -v cargo-deny >/dev/null 2>&1; then
    error "cargo-deny is required. Install cargo-deny 0.20.2 before release preflight."
    exit 127
fi
cargo deny check --hide-inclusion-graph
success "Dependency advisories, licenses, and sources satisfy policy."

# 7. Release Asset Inspector Validation
info "7/13 Verifying release asset inspector..."
bash scripts/dev/test-inspect-release-asset.sh
bash scripts/release/test-packaged-startup-contract.sh
bash scripts/release/test-collect-artifacts.sh
success "Release asset inspector preserves bundle paths."

# 8. macOS Coverage Linker Concurrency
info "8/13 Verifying macOS coverage linker concurrency..."
bash scripts/release/test-macos-coverage-contract.sh
bash scripts/release/check-macos-coverage-contract.sh
python3 scripts/ci/check-document-surface-coverage.py --self-test
success "macOS coverage linker concurrency is constrained."

# 9. Artifact Naming Validation
info "9/13 Verifying Cargo.toml version..."
CARGO_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
if [[ "$CARGO_VERSION" != "$VERSION" ]]; then
    error "Cargo.toml version ($CARGO_VERSION) does not match target release version ($VERSION)."
    exit 1
fi
success "Cargo.toml version matches."

info "10/13 Verifying Info.plist version..."
PLIST_VERSION=$(awk '/CFBundleShortVersionString/{getline; gsub(/.*<string>v?|<\/string>.*/, ""); print}' crates/katana-ui/Info.plist | xargs)
if [[ "$PLIST_VERSION" != "$VERSION" ]]; then
    error "Info.plist CFBundleShortVersionString ($PLIST_VERSION) does not match target release version ($VERSION)."
    exit 1
fi
success "Info.plist version matches."

# 10. CHANGELOG Validation
info "11/13 Validating CHANGELOG via AST Linter..."
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

# 11. Linuxbrew Formula Validation
info "12/13 Verifying Linuxbrew formula contract..."
scripts/release/check-linuxbrew-formula-contract.sh

# 12. OpenSpec Validation
info "13/13 Validating OpenSpec task completion..."
# WHY（日本語）: v0.22.42のdocument-fidelityはpost-変更名のため、版番号globだけでは公開前必須受入を見落とす。
python3 scripts/release/test-document-fidelity-release-gate.py
python3 scripts/release/check-document-fidelity-release-gate.py "$VERSION" "$TASK_GATE_MODE"
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
            elif [[ "$TASK_GATE_MODE" == "release-artifact-pending" && "$CHANGE_NAME" == "v0-22-39-office2pdf-official-intake" ]]; then
                # Public release verification can only run after GitHub publishes assets.
                TASK_GATE_ARGS=(--allow 4.1)
            elif [[ "$TASK_GATE_MODE" == "post-release-evidence" && "$CHANGE_NAME" == "v0-22-39-office2pdf-official-intake" ]]; then
                # The post-release evidence change must have no incomplete OpenSpec tasks.
                TASK_GATE_ARGS=()
            elif [[ "$TASK_GATE_MODE" == "pr-bootstrap" && "$CHANGE_NAME" == "v0-22-40-pptx-external-hyperlink-intake" ]]; then
                # CI screenshots and public release evidence do not exist before the PR runs.
                TASK_GATE_ARGS=(--allow 3.3 --allow 4.2 --allow 4.3 --allow 5.1)
            elif [[ "$TASK_GATE_MODE" == "release-artifact-pending" && "$CHANGE_NAME" == "v0-22-40-pptx-external-hyperlink-intake" ]]; then
                # Public release verification can only run after GitHub publishes assets.
                TASK_GATE_ARGS=(--allow 5.1)
            elif [[ "$TASK_GATE_MODE" == "post-release-evidence" && "$CHANGE_NAME" == "v0-22-40-pptx-external-hyperlink-intake" ]]; then
                # The post-release evidence change must have no incomplete OpenSpec tasks.
                TASK_GATE_ARGS=()
            elif [[ "$CHANGE_NAME" == "v0-22-41-document-viewer-defects" ]] && \
                [[ "$TASK_GATE_MODE" == "pr-bootstrap" || \
                    "$TASK_GATE_MODE" == "release-artifact-pending" || \
                    "$TASK_GATE_MODE" == "post-release-evidence" ]]; then
                # v0.22.41 has no deferred task: all release modes enforce strict completion.
                TASK_GATE_ARGS=()
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
