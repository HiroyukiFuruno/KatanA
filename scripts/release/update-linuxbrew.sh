#!/bin/zsh
# =============================================================================
# KatanA — Linuxbrew Formula Updater
# =============================================================================
# Usage: ./scripts/release/update-linuxbrew.sh <VERSION> <SHA256> <TAR_NAME>
# =============================================================================

set -euo pipefail

VERSION=$1
SHA256=$2
TAR_NAME=$3

if [[ -z "$VERSION" || -z "$SHA256" || -z "$TAR_NAME" ]]; then
    echo "Usage: $0 <VERSION> <SHA256> <TAR_NAME>" >&2
    exit 1
fi

VERSION_NUM="${VERSION#v}"

# Token Selection
TOKEN="${HOMEBREW_KATANA_GIT_TOKEN:-${GITHUB_TOKEN:-}}"
if [[ -z "$TOKEN" ]]; then
    echo "Error: Homebrew update token is not set." >&2
    exit 1
fi

REPO="HiroyukiFuruno/homebrew-katana"

function upload_formula() {
    local PATH_FI=$1
    local FORMULA_NAME=$2

    # macOS formula vs Linux formula (Actually, Formula works for both, but we only have it for Linux since macOS uses Cask)
    local FORMULA_CONTENT=""
    FORMULA_CONTENT+="class Katana < Formula\n"
    FORMULA_CONTENT+="  desc \"Lightweight Markdown viewer with live preview, Mermaid diagrams, and syntax highlighting\"\n"
    FORMULA_CONTENT+="  homepage \"https://github.com/HiroyukiFuruno/KatanA\"\n"
    FORMULA_CONTENT+="  version \"${VERSION_NUM}\"\n\n"
    
    FORMULA_CONTENT+="  if OS.linux?\n"
    FORMULA_CONTENT+="    url \"https://github.com/HiroyukiFuruno/KatanA/releases/download/${VERSION}/${TAR_NAME}\"\n"
    FORMULA_CONTENT+="    sha256 \"${SHA256}\"\n"
    FORMULA_CONTENT+="  end\n\n"

    FORMULA_CONTENT+="  def install\n"
    FORMULA_CONTENT+="    if OS.linux?\n"
    FORMULA_CONTENT+="      bin.install \"KatanA\"\n"
    FORMULA_CONTENT+="    end\n"
    FORMULA_CONTENT+="  end\n"
    FORMULA_CONTENT+="end\n"

    local ENCODED=$(printf "%b" "$FORMULA_CONTENT" | base64 | tr -d '\n')
    
    echo "[INFO] Fetching current Formula at $PATH_FI from $REPO..."
    local CURRENT=$(curl -s -H "Authorization: token $TOKEN" "https://api.github.com/repos/$REPO/contents/$PATH_FI")
    
    local FILE_SHA=""
    if printf "%s" "$CURRENT" | python3 -c "import sys,json; d=json.load(sys.stdin); sys.exit(0 if 'sha' in d else 1)" 2>/dev/null; then
        FILE_SHA=$(printf "%s" "$CURRENT" | python3 -c "import sys,json; print(json.load(sys.stdin)['sha'])")
    fi

    local JSON_PAYLOAD=""
    if [[ -n "$FILE_SHA" ]]; then
        JSON_PAYLOAD="{\"message\": \"chore: update $FORMULA_NAME to $VERSION_NUM\", \"content\": \"$ENCODED\", \"sha\": \"$FILE_SHA\"}"
    else
        JSON_PAYLOAD="{\"message\": \"chore: add $FORMULA_NAME ($VERSION_NUM)\", \"content\": \"$ENCODED\"}"
    fi

    echo "[INFO] Updating Formula in $REPO..."
    local RESPONSE=$(curl -s -w "\n%{http_code}" -X PUT \
      -H "Authorization: token $TOKEN" \
      -H "Content-Type: application/json" \
      "https://api.github.com/repos/$REPO/contents/$PATH_FI" \
      -d "$JSON_PAYLOAD")

    local HTTP_CODE=$(echo "$RESPONSE" | tail -1)
    local BODY=$(echo "$RESPONSE" | sed '$d')

    if [[ "$HTTP_CODE" -ge 200 && "$HTTP_CODE" -lt 300 ]]; then
        local URL=$(echo "$BODY" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('content',{}).get('html_url','unknown'))")
        echo "[OK] Updated Homebrew Formula: $URL"
    else
        echo "Error: Failed to update Formula $FORMULA_NAME (HTTP $HTTP_CODE)." >&2
        echo "$BODY" >&2
        exit 1
    fi
}

upload_formula "Formula/katana.rb" "katana"
