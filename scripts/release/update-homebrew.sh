#!/bin/zsh
# =============================================================================
# KatanA — Homebrew Cask Updater
# =============================================================================
# Usage: ./scripts/release/update-homebrew.sh <VERSION> <SHA256> <DMG_NAME>
# =============================================================================

set -euo pipefail

VERSION=$1
SHA256=$2
DMG_NAME=$3
VERSION_ONLY=0

if [[ "${4:-}" == "--version-only" ]]; then
    VERSION_ONLY=1
fi

if [[ -z "$VERSION" || -z "$SHA256" || -z "$DMG_NAME" ]]; then
    echo "Usage: $0 <VERSION> <SHA256> <DMG_NAME> [--version-only]" >&2
    exit 1
fi

VERSION_NUM="${VERSION#v}"

# Token Selection
TOKEN="${HOMEBREW_KATANA_GIT_TOKEN:-${HOMEBREW_TAP_TOKEN:-}}"
if [[ -z "$TOKEN" ]]; then
    echo "Error: Homebrew update token is not set." >&2
    exit 1
fi

REPO="HiroyukiFuruno/homebrew-katana"

function upload_cask() {
    local PATH_FI=$1
    local CASK_TOKEN=$2
    local IS_VERSIONED=$3

    local CASK_CONTENT=""
    CASK_CONTENT+="cask \"${CASK_TOKEN}\" do\n"
    CASK_CONTENT+="  version \"${VERSION_NUM}\"\n"
    CASK_CONTENT+="  sha256 \"${SHA256}\"\n\n"
    CASK_CONTENT+="  url \"https://github.com/HiroyukiFuruno/KatanA/releases/download/v#{version}/KatanA-Desktop-#{version}.dmg\"\n"
    CASK_CONTENT+="  name \"KatanA Desktop\"\n"
    CASK_CONTENT+="  desc \"Lightweight Markdown viewer with live preview, Mermaid diagrams, and syntax highlighting\"\n"
    CASK_CONTENT+="  homepage \"https://github.com/HiroyukiFuruno/KatanA\"\n\n"
    
    if [[ "$IS_VERSIONED" == "0" ]]; then
        CASK_CONTENT+="  livecheck do\n"
        CASK_CONTENT+="    url :url\n"
        CASK_CONTENT+="    strategy :github_latest\n"
        CASK_CONTENT+="  end\n\n"
    fi

    CASK_CONTENT+="  depends_on macos: \">= :ventura\"\n\n"
    CASK_CONTENT+="  app \"KatanA Desktop.app\"\n\n"
    CASK_CONTENT+="  # Remove quarantine attribute (required for ad-hoc signed apps without Apple notarization)\n"
    CASK_CONTENT+="  postflight do\n"
    CASK_CONTENT+="    system_command \"/usr/bin/xattr\",\n"
    CASK_CONTENT+="                   args: [\"-cr\", \"#{appdir}/KatanA Desktop.app\"]\n"
    CASK_CONTENT+="  end\n\n"
    CASK_CONTENT+="  zap trash: [\n"
    CASK_CONTENT+="    \"~/Library/Preferences/com.katana.desktop.plist\",\n"
    CASK_CONTENT+="    \"~/Library/Caches/com.katana.desktop\",\n"
    CASK_CONTENT+="  ]\n"
    CASK_CONTENT+="end\n"

    local ENCODED=$(printf "%b" "$CASK_CONTENT" | base64 | tr -d '\n')
    
    echo "[INFO] Fetching current Cask at $PATH_FI from $REPO..."
    local CURRENT=$(curl -s -H "Authorization: token $TOKEN" "https://api.github.com/repos/$REPO/contents/$PATH_FI")
    
    local FILE_SHA=""
    if printf "%s" "$CURRENT" | python3 -c "import sys,json; d=json.load(sys.stdin); sys.exit(0 if 'sha' in d else 1)" 2>/dev/null; then
        FILE_SHA=$(printf "%s" "$CURRENT" | python3 -c "import sys,json; print(json.load(sys.stdin)['sha'])")
    fi

    local JSON_PAYLOAD=""
    if [[ -n "$FILE_SHA" ]]; then
        JSON_PAYLOAD="{\"message\": \"chore: update $CASK_TOKEN to $VERSION_NUM\", \"content\": \"$ENCODED\", \"sha\": \"$FILE_SHA\"}"
    else
        JSON_PAYLOAD="{\"message\": \"chore: add $CASK_TOKEN ($VERSION_NUM)\", \"content\": \"$ENCODED\"}"
    fi

    echo "[INFO] Updating Cask in $REPO..."
    local RESPONSE=$(curl -s -w "\n%{http_code}" -X PUT \
      -H "Authorization: token $TOKEN" \
      -H "Content-Type: application/json" \
      "https://api.github.com/repos/$REPO/contents/$PATH_FI" \
      -d "$JSON_PAYLOAD")

    local HTTP_CODE=$(echo "$RESPONSE" | tail -1)
    local BODY=$(echo "$RESPONSE" | sed '$d')

    if [[ "$HTTP_CODE" -ge 200 && "$HTTP_CODE" -lt 300 ]]; then
        local URL=$(echo "$BODY" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('content',{}).get('html_url','unknown'))")
        echo "[OK] Updated Homebrew Cask: $URL"
    else
        echo "Error: Failed to update Cask $CASK_TOKEN (HTTP $HTTP_CODE)." >&2
        echo "$BODY" >&2
        exit 1
    fi
}

# Always create the version-specific cask:
upload_cask "Casks/katana-desktop@${VERSION_NUM}.rb" "katana-desktop@${VERSION_NUM}" "1"

# If not --version-only, also update the main latest cask:
if [[ "$VERSION_ONLY" == "0" ]]; then
    upload_cask "Casks/katana-desktop.rb" "katana-desktop" "0"
fi

