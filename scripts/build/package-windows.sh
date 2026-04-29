#!/bin/bash
# =============================================================================
# KatanA — Windows Artifact Packager
# =============================================================================

set -euo pipefail

# ── Colours ──────────────────────────────────────────────────────────────────
GREEN='\033[0;32m'
CYAN='\033[0;36m'
RESET='\033[0m'

info()    { echo -e "${CYAN}[INFO]${RESET}  $*"; }
success() { echo -e "${GREEN}[OK]${RESET}    $*"; }

info "Packaging Windows artifacts..."

CURRENT_VERSION=$(grep '^version =' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')

# Install cargo-wix if not present (assuming cargo-binstall is available or will install it)
if ! command -v cargo-wix &> /dev/null; then
    info "Installing cargo-wix..."
    cargo binstall cargo-wix -y
fi

info "Removing stale Windows package artifacts..."
rm -f target/wix/*.msi
rm -f KatanA-windows-x86_64.zip KatanA-windows-x86_64.msi
rm -f target/release/KatanA-windows-x86_64.zip

info "Building MSI Installer with WiX..."
# WHY: cargo wix runs WiX linker (light.exe) from CWD, and main.wxs references
# resources as 'wix\Product.ico' / 'wix\License.rtf' relative to the crate root.
cd crates/katana-ui
cargo wix --package katana-ui --nocapture
cd ../..

info "Packaging ZIP archive..."
cd target/release
7z a KatanA-windows-x86_64.zip KatanA.exe
cd ../..

info "Copying artifacts to project root..."
cp target/release/KatanA-windows-x86_64.zip ./

# WHY: target/wix can be restored from cache; only the current release MSI is valid.
MSI_FILES=$(find target/wix -maxdepth 1 -name "*${CURRENT_VERSION}*.msi" -type f | sort)
MSI_COUNT=$(printf '%s\n' "$MSI_FILES" | sed '/^$/d' | wc -l | tr -d ' ')
if [ "$MSI_COUNT" -ne 1 ]; then
    echo "ERROR: Expected exactly one MSI for version ${CURRENT_VERSION}, found ${MSI_COUNT}." >&2
    find target/wix -maxdepth 1 -name '*.msi' -type f -print >&2
    exit 1
fi
MSI_FILE=$(printf '%s\n' "$MSI_FILES" | head -n 1)
cp "$MSI_FILE" ./KatanA-windows-x86_64.msi

success "Successfully built Windows packages (.zip and .msi)"
