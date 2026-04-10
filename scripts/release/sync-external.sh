#!/bin/bash
set -euo pipefail

# Usage: ./scripts/release/sync-external.sh <version> <tag> <artifacts_dir>
VERSION=$1
TAG=$2
ARTIFACTS_DIR=$3

if [[ -z "$VERSION" || -z "$TAG" || -z "$ARTIFACTS_DIR" ]]; then
    echo "Usage: $0 <version> <tag> <artifacts_dir>"
    exit 1
fi

echo "🌏 Syncing with external registries for v${VERSION}..."

# 1. Homebrew / Linuxbrew
if [ -f "scripts/release/update-homebrew.sh" ] && [ -n "${HOMEBREW_KATANA_GIT_TOKEN:-}" ]; then
    DMG=$(find "${ARTIFACTS_DIR}" -name "KatanA-Desktop-*.dmg" -maxdepth 1 | sort -V | tail -1)
    if [ -f "$DMG" ]; then
        SHA256=$(shasum -a 256 "$DMG" | awk '{print $1}')
        echo "🍺 Updating Homebrew Cask..."
        ./scripts/release/update-homebrew.sh "${TAG}" "${SHA256}" "$(basename "$DMG")"
    fi
fi

if [ -f "scripts/release/update-linuxbrew.sh" ] && [ -n "${HOMEBREW_KATANA_GIT_TOKEN:-}" ]; then
    TAR=$(find "${ARTIFACTS_DIR}" -name "KatanA-linux-*.tar.gz" -maxdepth 1 | sort -V | tail -1)
    if [ -f "$TAR" ]; then
        SHA256=$(shasum -a 256 "$TAR" | awk '{print $1}')
        echo "🍺 Updating Linuxbrew Formula..."
        ./scripts/release/update-linuxbrew.sh "${TAG}" "${SHA256}" "$(basename "$TAR")"
    fi
fi

# 2. Winget
if [ -n "${GITHUB_TOKEN:-}" ]; then
    # We check if komac is available, otherwise try to install it
    if ! command -v komac &> /dev/null; then
        echo "📦 Installing komac..."
        brew install nicehash/tap/komac 2>/dev/null || true
    fi
    
    if command -v komac &> /dev/null; then
        echo "🪟 Updating Winget package..."
        komac update HiroyukiFuruno.katana-desktop \
            --version "${VERSION}" \
            --urls "https://github.com/HiroyukiFuruno/KatanA/releases/download/${TAG}/KatanA-windows-x86_64.msi" \
            --submit || echo "Warning: Winget update failed but won't block release."
    else
        echo "⚠️ komac not found, skipping Winget update."
    fi
fi

echo "✅ External registry sync finished."
