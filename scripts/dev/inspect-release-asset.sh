#!/usr/bin/env bash
# =============================================================================
# KatanA — Release Asset Inspector
# =============================================================================
# Purpose:
#   リリース成果物の構造を実測で確認するための調査スクリプト。
#   リリース関連バグ(自動更新失敗、起動失敗、パッケージング異常等)を
#   調査する AI エージェント / 開発者は、コード読解より先に本スクリプトを
#   実行し、出力をトリアージ資料として添付すること。
#
# Usage:
#   ./scripts/dev/inspect-release-asset.sh <tag>            # 全アセット
#   ./scripts/dev/inspect-release-asset.sh <tag> linux      # 個別
#   ./scripts/dev/inspect-release-asset.sh <tag> windows
#   ./scripts/dev/inspect-release-asset.sh <tag> macos
#
# Examples:
#   ./scripts/dev/inspect-release-asset.sh v0.22.13
#   ./scripts/dev/inspect-release-asset.sh latest linux
#
# Output:
#   - 各アセットの SHA-256 / サイズ / 中身トップレベル構造
#   - checksums.txt との照合結果
#   - 期待ファイル名(`KatanA` / `KatanA.exe` / `KatanA Desktop.app`)の存在有無
# =============================================================================

set -euo pipefail

REPO="${KATANA_RELEASE_REPO:-HiroyukiFuruno/KatanA}"
TAG="${1:-}"
FILTER="${2:-all}"

if [[ -z "$TAG" ]]; then
    echo "Usage: $0 <tag> [linux|windows|macos|all]" >&2
    exit 1
fi

if ! command -v gh >/dev/null 2>&1 && ! command -v curl >/dev/null 2>&1; then
    echo "Error: gh または curl のいずれかが必要です。" >&2
    exit 1
fi

WORK_DIR="$(mktemp -d -t katana-release-inspect.XXXXXX)"
trap 'rm -rf "$WORK_DIR"' EXIT
cd "$WORK_DIR"

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
BOLD='\033[1m'
RESET='\033[0m'

info()    { printf "${CYAN}[INFO]${RESET}  %s\n" "$*"; }
ok()      { printf "${GREEN}[OK]${RESET}    %s\n" "$*"; }
warn()    { printf "${YELLOW}[WARN]${RESET}  %s\n" "$*"; }
fail()    { printf "${RED}[FAIL]${RESET}  %s\n" "$*"; }
section() { printf "\n${BOLD}== %s ==${RESET}\n" "$*"; }

# ---------------------------------------------------------------------------
# Asset list resolution
# ---------------------------------------------------------------------------

resolve_tag() {
    local raw="$1"
    if [[ "$raw" == "latest" ]]; then
        if command -v gh >/dev/null 2>&1; then
            gh release view --repo "$REPO" --json tagName -q .tagName
        else
            curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
                | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -1
        fi
    else
        echo "$raw"
    fi
}

TAG="$(resolve_tag "$TAG")"
info "Repo: $REPO"
info "Tag : $TAG"

ASSET_BASE_URL="https://github.com/$REPO/releases/download/$TAG"

assets_for_filter() {
    case "$FILTER" in
        linux)   echo "KatanA-linux-x86_64.tar.gz" ;;
        windows) echo "KatanA-windows-x86_64.zip KatanA-windows-x86_64.msi" ;;
        macos)   echo "KatanA-macOS.zip" ;;
        all)
            echo "KatanA-linux-x86_64.tar.gz KatanA-windows-x86_64.zip KatanA-windows-x86_64.msi KatanA-macOS.zip"
            ;;
        *)
            fail "Unknown filter: $FILTER"
            exit 1
            ;;
    esac
}

# ---------------------------------------------------------------------------
# Download helpers
# ---------------------------------------------------------------------------

download() {
    local name="$1"
    local url="$ASSET_BASE_URL/$name"
    if [[ -n "${KATANA_RELEASE_ASSET_DIR:-}" ]]; then
        local source_path="$KATANA_RELEASE_ASSET_DIR/$name"
        if [[ ! -f "$source_path" ]]; then
            fail "Local asset not found: $source_path"
            return 1
        fi
        cp "$source_path" "$name"
        ok "Loaded local $name ($(du -h "$name" | awk '{print $1}'))"
        return
    fi
    info "Downloading $name ..."
    if ! curl -fsSL --retry 2 -o "$name" "$url"; then
        fail "Could not download $name"
        return 1
    fi
    ok "Downloaded $name ($(du -h "$name" | awk '{print $1}'))"
}

sha256_of() {
    if command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | awk '{print $1}'
    else
        sha256sum "$1" | awk '{print $1}'
    fi
}

# ---------------------------------------------------------------------------
# Per-asset inspection
# ---------------------------------------------------------------------------

inspect_tar_gz() {
    local name="$1"
    section "$name (tar.gz)"
    download "$name" || return
    echo "SHA-256: $(sha256_of "$name")"
    echo "--- top-level entries ---"
    tar -tzvf "$name" | awk '{ if (NF >= 6) print $0 }' | head -50
    echo
    if tar -tzf "$name" | grep -qxF "KatanA"; then
        ok "Asset contract OK: top-level 'KatanA' present"
    else
        fail "Asset contract VIOLATION: top-level 'KatanA' NOT found"
        echo "Actual top-level entries:"
        tar -tzf "$name" | awk -F/ '{print $1}' | sort -u
    fi
}

inspect_zip() {
    local name="$1"
    local expected="$2"
    local entries
    section "$name (zip)"
    download "$name" || return
    echo "SHA-256: $(sha256_of "$name")"
    echo "--- entries (head 50) ---"
    entries=$(unzip -Z1 "$name")
    sed -n '1,50p' <<<"$entries"
    echo
    if grep -qxF "$expected" <<<"$entries"; then
        ok "Asset contract OK: '$expected' present"
    else
        fail "Asset contract VIOLATION: '$expected' NOT found"
        echo "Actual top-level entries:"
        awk -F/ '{print $1}' <<<"$entries" | sort -u | grep -v '^$' | sed -n '1,20p'
    fi
}

inspect_msi() {
    local name="$1"
    section "$name (msi)"
    download "$name" || return
    echo "SHA-256: $(sha256_of "$name")"
    info "MSI inspection requires lessmsi/msitools (skipped in lightweight inspector). Size: $(du -h "$name" | awk '{print $1}')"
}

# ---------------------------------------------------------------------------
# Checksums verification
# ---------------------------------------------------------------------------

verify_checksums() {
    section "checksums.txt verification"
    if ! download "checksums.txt"; then
        warn "checksums.txt not published for this tag"
        return
    fi
    cat checksums.txt
    echo "--- recomputed ---"
    for f in *; do
        [[ "$f" == "checksums.txt" ]] && continue
        [[ -f "$f" ]] || continue
        local h
        h="$(sha256_of "$f")"
        if grep -qF "$h" checksums.txt; then
            ok "$f matches published checksum"
        else
            fail "$f checksum mismatch (recomputed: $h)"
        fi
    done
}

# ---------------------------------------------------------------------------
# Dispatch
# ---------------------------------------------------------------------------

for asset in $(assets_for_filter); do
    case "$asset" in
        *.tar.gz) inspect_tar_gz "$asset" ;;
        KatanA-linux*.zip)   inspect_zip "$asset" "KatanA" ;;
        KatanA-windows*.zip) inspect_zip "$asset" "KatanA.exe" ;;
        KatanA-macOS*.zip)   inspect_zip "$asset" "KatanA Desktop.app/" ;;
        *.msi)               inspect_msi "$asset" ;;
        *) warn "Unknown asset type: $asset" ;;
    esac
done

verify_checksums

section "Summary"
ok "Inspection complete. Use this output as the verification baseline."
echo
echo "Expected asset contract:"
echo "  Linux  tar.gz: top-level file 'KatanA'"
echo "  Win    zip   : top-level file 'KatanA.exe'"
echo "  macOS  zip   : top-level dir  'KatanA Desktop.app/'"
echo
echo "If actual entries diverge from the contract, file it under"
echo "  scripts/release/check-release-asset-contract.sh"
echo "or fix the packaging scripts (scripts/build/package-*.sh)."
