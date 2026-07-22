#!/bin/bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
TMP_ROOT=$(mktemp -d)
trap 'rm -rf "$TMP_ROOT"' EXIT

FIXTURE_DIR="$TMP_ROOT/fixture"
BUNDLE_DIR="$TMP_ROOT/bundle/KatanA Desktop.app/Contents/MacOS"
mkdir -p "$FIXTURE_DIR" "$BUNDLE_DIR"
printf '%s\n' 'fixture' >"$BUNDLE_DIR/KatanA"
(
    cd "$TMP_ROOT/bundle"
    zip -qr "$FIXTURE_DIR/KatanA-macOS.zip" "KatanA Desktop.app"
)

OUTPUT=$(
    KATANA_RELEASE_ASSET_DIR="$FIXTURE_DIR" \
        bash "$ROOT_DIR/scripts/dev/inspect-release-asset.sh" v-test macos
)

grep -qF "Asset contract OK: 'KatanA Desktop.app/' present" <<<"$OUTPUT"
if grep -qF "Asset contract VIOLATION" <<<"$OUTPUT"; then
    printf '%s\n' "$OUTPUT" >&2
    exit 1
fi

echo "PASS: release asset inspector preserves ZIP entry names containing spaces"
