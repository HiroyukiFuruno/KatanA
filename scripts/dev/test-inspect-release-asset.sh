#!/bin/bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
TMP_ROOT=$(mktemp -d)
trap 'rm -rf "$TMP_ROOT"' EXIT

FIXTURE_DIR="$TMP_ROOT/fixture"
CONTENTS_DIR="$TMP_ROOT/bundle/KatanA Desktop.app/Contents"
BUNDLE_DIR="$CONTENTS_DIR/MacOS"
mkdir -p "$FIXTURE_DIR" "$BUNDLE_DIR"
python3 - "$BUNDLE_DIR/KatanA" "$BUNDLE_DIR/kdv-office-worker" "$CONTENTS_DIR/Info.plist" <<'PY'
import plistlib
import struct
import sys
from pathlib import Path

def thin_macho(cpu_type):
    command = struct.pack("<6I", 0x32, 24, 1, 13 << 16, 0, 0)
    header = struct.pack("<8I", 0xFEEDFACF, cpu_type, 0, 2, 1, len(command), 0, 0)
    return header + command

def fat_macho():
    slices = [
        (0x0100000C, thin_macho(0x0100000C)),
        (0x01000007, thin_macho(0x01000007)),
    ]
    header_size = 8 + 20 * len(slices)
    entries = []
    payload = bytearray()
    offset = header_size
    for cpu_type, thin in slices:
        entries.append(struct.pack(">5I", cpu_type, 0, offset, len(thin), 0))
        payload.extend(thin)
        offset += len(thin)
    return b"\xca\xfe\xba\xbe" + struct.pack(">I", len(slices)) + b"".join(entries) + payload

for raw_path in sys.argv[1:3]:
    path = Path(raw_path)
    path.write_bytes(fat_macho())
    path.chmod(0o755)
Path(sys.argv[3]).write_bytes(plistlib.dumps({"LSMinimumSystemVersion": "13.0"}))
PY
(
    cd "$TMP_ROOT/bundle"
    zip -qr "$FIXTURE_DIR/KatanA-macOS.zip" "KatanA Desktop.app"
)

OUTPUT=$(
    KATANA_RELEASE_ASSET_DIR="$FIXTURE_DIR" \
        bash "$ROOT_DIR/scripts/dev/inspect-release-asset.sh" v-test macos
)

grep -qF "Asset contract OK: 'KatanA Desktop.app/' present" <<<"$OUTPUT"
grep -qF "Asset contract OK: 'KatanA Desktop.app/Contents/MacOS/kdv-office-worker' present" <<<"$OUTPUT"
grep -qF "Binary architecture contract satisfied" <<<"$OUTPUT"
if grep -qF "Asset contract VIOLATION" <<<"$OUTPUT"; then
    printf '%s\n' "$OUTPUT" >&2
    exit 1
fi

echo "PASS: release asset inspector preserves ZIP entry names containing spaces"
