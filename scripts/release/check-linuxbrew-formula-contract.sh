#!/usr/bin/env bash
# =============================================================================
# KatanA — Linuxbrew Formula Contract Verifier
# =============================================================================
# Purpose:
#   Linuxbrew の Formula が、ユーザー向けコマンド名 `katana-desktop` を
#   継続してインストールすることを検証する。
#
#   リリース成果物内の実行ファイル名は `KatanA` だが、Homebrew 経由の
#   公開コマンド名は `katana-desktop` で固定する。
# =============================================================================

set -euo pipefail

UPDATER_PATH="scripts/release/update-linuxbrew.sh"
EXPECTED_INSTALL='bin.install \"KatanA\" => \"katana-desktop\"'
EXPECTED_LINK_OVERWRITE='link_overwrite \"bin/katana-desktop\"'

fail() {
    echo "::error::LINUXBREW FORMULA CONTRACT VIOLATION: $*" >&2
    echo "FAIL: $*" >&2
    exit 1
}

ok() { echo "OK: $*"; }

[[ -f "$UPDATER_PATH" ]] || fail "Linuxbrew updater not found at: $UPDATER_PATH"

if ! grep -qF "$EXPECTED_INSTALL" "$UPDATER_PATH"; then
    fail "Linuxbrew formula must install 'KatanA' as the public 'katana-desktop' command"
fi

if ! grep -qF "$EXPECTED_LINK_OVERWRITE" "$UPDATER_PATH"; then
    fail "Linuxbrew formula must overwrite stale 'katana-desktop' links from older versions"
fi

ok "Linuxbrew formula command contract satisfied ($UPDATER_PATH)"
