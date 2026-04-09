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

# Install cargo-wix if not present (assuming cargo-binstall is available or will install it)
if ! command -v cargo-wix &> /dev/null; then
    info "Installing cargo-wix..."
    cargo binstall cargo-wix -y
fi

info "Building MSI Installer with WiX..."
cargo wix --package katana-ui --nocapture

info "Packaging ZIP archive..."
cd target/release
7z a KatanA-windows-x86_64.zip KatanA.exe
cd ../..

info "Copying MSI to release root..."
cp target/wix/*.msi ./KatanA-windows-x86_64.msi

success "Successfully built Windows packages (.zip and .msi)"
