#!/bin/zsh
# =============================================================================
# KatanA — macOS Packaging (App Bundle)
# =============================================================================

set -euo pipefail

# ── Configuration ─────────────────────────────────────────────────────────────
APP_NAME="KatanA Desktop"
APP_BUNDLE="target/release/bundle/osx/${APP_NAME}.app"
CONTENTS="${APP_BUNDLE}/Contents"

# ── Colours ──────────────────────────────────────────────────────────────────
GREEN='\033[0;32m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

# ── Helpers ───────────────────────────────────────────────────────────────────
info()    { echo "${CYAN}[INFO]${RESET}  $*"; }
success() { echo "${GREEN}[OK]${RESET}    $*"; }

# ── Argument Validation ───────────────────────────────────────────────────────
VERSION=$1
if [[ -z "$VERSION" ]]; then
    # Fallback to Cargo.toml version if not provided
    VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
fi

# ── Execution ─────────────────────────────────────────────────────────────────
info "Packaging macOS .app bundle (release)..."

cargo bundle --release --format osx --package katana-ui

info "Overlaying project-specific Info.plist..."
cp crates/katana-ui/Info.plist "${CONTENTS}/Info.plist"

info "Syncing version v${VERSION} to bundle Info.plist..."
perl -i -0pe 's/(<key>CFBundleShortVersionString<\/key>\s*<string>).*?(<\/string>)/$1v'"${VERSION}"'$2/' "${CONTENTS}/Info.plist"

info "Adding Resources (icon.icns)..."
mkdir -p "${CONTENTS}/Resources"
cp assets/icon.icns "${CONTENTS}/Resources/icon.icns"

info "Applying Ad-hoc Code Signature (Required after modifying Info.plist to prevent 'damaged' Gatekeeper error)..."
codesign --force --deep --sign - "${APP_BUNDLE}"

success "Created ${APP_BUNDLE}"
