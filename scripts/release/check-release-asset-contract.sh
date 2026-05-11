#!/usr/bin/env bash
# =============================================================================
# KatanA — Release Asset Contract Verifier
# =============================================================================
# Purpose:
#   ビルド直後のローカルアーティファクトが、配布契約 (Asset Contract) で
#   定められたトップレベルファイル名を持つことを検証する。
#
#   この契約に違反するとインストーラやパッケージマネージャの整合性が崩れ、
#   自動更新失敗等の事故につながるため、CI でビルド成果物アップロード前に
#   実行することを必須とする。
#
# Asset Contract:
#   - Linux  tar.gz: top-level file 'KatanA'
#   - Win    zip   : top-level file 'KatanA.exe'
#   - Win    msi   : msi 自体は契約検証対象外 (中身は wix で固定)
#   - macOS  zip   : top-level dir  'KatanA Desktop.app/'
#   - macOS  dmg   : dmg 自体は検証対象外 (中身は create-dmg で固定)
#
# Usage:
#   ./scripts/release/check-release-asset-contract.sh linux   [path]
#   ./scripts/release/check-release-asset-contract.sh windows [path]
#   ./scripts/release/check-release-asset-contract.sh macos   [path]
#
#   `path` を省略するとビルドのデフォルト出力先を見る。
#   - Linux  default: target/release/KatanA-linux-x86_64.tar.gz
#   - Win    default: KatanA-windows-x86_64.zip
#   - macOS  default: target/release/KatanA-macOS.zip
# =============================================================================

set -euo pipefail

PLATFORM="${1:-}"
ASSET_PATH="${2:-}"

if [[ -z "$PLATFORM" ]]; then
    echo "Usage: $0 <linux|windows|macos> [asset-path]" >&2
    exit 2
fi

fail() {
    echo "::error::ASSET CONTRACT VIOLATION: $*" >&2
    echo "FAIL: $*" >&2
    exit 1
}

ok() { echo "OK: $*"; }

check_linux() {
    local path="${ASSET_PATH:-target/release/KatanA-linux-x86_64.tar.gz}"
    [[ -f "$path" ]] || fail "Linux artifact not found at: $path"
    local entries top
    entries=$(tar -tzf "$path")
    if ! echo "$entries" | grep -qxF "KatanA"; then
        echo "Actual top-level entries:" >&2
        echo "$entries" | awk -F/ '{print $1}' | sort -u >&2
        fail "Linux tar.gz must contain top-level file 'KatanA'"
    fi
    top=$(echo "$entries" | awk -F/ '{print $1}' | sort -u | grep -cv '^$')
    if [[ "$top" -ne 1 ]]; then
        echo "Top-level entries:" >&2
        echo "$entries" | awk -F/ '{print $1}' | sort -u >&2
        fail "Linux tar.gz must contain exactly one top-level entry (got $top)"
    fi
    ok "Linux tar.gz contract satisfied ($path)"
}

check_windows() {
    local path="${ASSET_PATH:-KatanA-windows-x86_64.zip}"
    [[ -f "$path" ]] || fail "Windows artifact not found at: $path"
    if ! command -v unzip >/dev/null 2>&1; then
        fail "unzip command required but not available"
    fi
    local names
    names=$(unzip -Z1 "$path")
    if ! echo "$names" | grep -qxF "KatanA.exe"; then
        echo "Actual entries:" >&2
        echo "$names" >&2
        fail "Windows zip must contain top-level file 'KatanA.exe'"
    fi
    ok "Windows zip contract satisfied ($path)"
}

check_macos() {
    local path="${ASSET_PATH:-target/release/KatanA-macOS.zip}"
    [[ -f "$path" ]] || fail "macOS artifact not found at: $path"
    local names
    names=$(unzip -Z1 "$path")
    if ! echo "$names" | grep -qE '^KatanA Desktop\.app/'; then
        echo "Actual entries (head):" >&2
        echo "$names" | head -20 >&2
        fail "macOS zip must contain top-level 'KatanA Desktop.app/'"
    fi
    if ! echo "$names" | grep -qxF "KatanA Desktop.app/Contents/Info.plist"; then
        fail "macOS bundle must contain Contents/Info.plist"
    fi
    if ! echo "$names" | grep -qxF "KatanA Desktop.app/Contents/MacOS/KatanA"; then
        fail "macOS bundle must contain Contents/MacOS/KatanA executable"
    fi
    ok "macOS zip contract satisfied ($path)"
}

case "$PLATFORM" in
    linux)   check_linux ;;
    windows) check_windows ;;
    macos)   check_macos ;;
    *) echo "Unknown platform: $PLATFORM" >&2; exit 2 ;;
esac
