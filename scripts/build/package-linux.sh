#!/bin/bash
# =============================================================================
# KatanA — Linux Artifact Packager
# =============================================================================

set -euo pipefail

# ── Colours ──────────────────────────────────────────────────────────────────
GREEN='\033[0;32m'
CYAN='\033[0;36m'
RESET='\033[0m'

info()    { echo -e "${CYAN}[INFO]${RESET}  $*"; }
success() { echo -e "${GREEN}[OK]${RESET}    $*"; }

info "Packaging Linux artifact..."

cd target/release
zip KatanA-linux-x86_64.zip KatanA

success "Successfully built target/release/KatanA-linux-x86_64.zip"
