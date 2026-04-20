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
PACKAGE_ID="HiroyukiFuruno.katana-desktop"
MSI_URL="https://github.com/HiroyukiFuruno/KatanA/releases/download/${TAG}/KatanA-windows-x86_64.msi"

if [ -z "${WINGET_GH_TOKEN:-}" ]; then
    echo "⚠️ WINGET_GH_TOKEN (classic PAT with public_repo) is not set, skipping winget sync."
elif [ ! -f "${ARTIFACTS_DIR}/KatanA-windows-x86_64.msi" ]; then
    echo "⚠️ Windows MSI artifact not found, skipping winget sync."
else
    # We check if komac is available, otherwise try to install it
    if ! command -v komac &> /dev/null; then
        echo "📦 Installing komac..."
        brew install komac 2>/dev/null || true
    fi

    if ! command -v komac &> /dev/null; then
        echo "⚠️ komac not found, skipping winget sync."
    else
        echo "🪟 Checking Winget package..."
        if komac list "${PACKAGE_ID}" --token "${WINGET_GH_TOKEN}" >/dev/null 2>&1; then
            echo "🪟 Updating Winget package..."
            komac update "${PACKAGE_ID}" \
                --version "${VERSION}" \
                --urls "${MSI_URL}" \
                --release-notes-url "https://github.com/HiroyukiFuruno/KatanA/releases/tag/${TAG}" \
                --submit \
                --token "${WINGET_GH_TOKEN}" || echo "Warning: Winget update failed but won't block release."
        else
            echo "⚠️ ${PACKAGE_ID} does not exist in microsoft/winget-pkgs yet."
            echo "⚠️ Initial winget bootstrap is required; skipping automated update flow."
        fi
    fi
fi

echo "✅ External registry sync finished."
