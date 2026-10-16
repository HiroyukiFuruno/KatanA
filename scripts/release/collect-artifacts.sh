#!/bin/bash
set -euo pipefail

# Usage: ./scripts/release/collect-artifacts.sh <output_dir>
OUTPUT_DIR=${1:-"release-files"}
shopt -s nullglob dotglob

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

require_fixture_directory() {
    local directory="$1"
    [[ -d "$directory" && ! -L "$directory" ]] || fail "artifact directory is missing or symlinked: $directory"
}

require_nonempty_file() {
    local path="$1"
    [[ -f "$path" && ! -L "$path" ]] || fail "required artifact is not a regular file: $path"
    [[ -s "$path" ]] || fail "required artifact is empty: $path"
}

reject_unknown_entries() {
    local directory="$1"
    shift
    local entry name allowed expected
    for entry in "$directory"/*; do
        [[ -e "$entry" || -L "$entry" ]] || continue
        name=${entry##*/}
        allowed=false
        for expected in "$@"; do
            if [[ "$name" == "$expected" ]]; then
                allowed=true
                break
            fi
        done
        [[ "$allowed" == true ]] || fail "unexpected artifact in $directory: $name"
    done
}

require_output_directory_is_safe() {
    if [[ -L "$OUTPUT_DIR" || ( -e "$OUTPUT_DIR" && ! -d "$OUTPUT_DIR" ) ]]; then
        fail "output path is not a real directory: $OUTPUT_DIR"
    fi
    if [[ -d "$OUTPUT_DIR" ]]; then
        local entry
        for entry in "$OUTPUT_DIR"/*; do
            [[ -e "$entry" || -L "$entry" ]] || continue
            fail "output collision; refusing to overwrite: ${entry##*/}"
        done
    fi
}

require_fixture_directory artifacts-macos
require_fixture_directory artifacts-linux
require_fixture_directory artifacts-windows
require_output_directory_is_safe

MACOS_ZIP="artifacts-macos/KatanA-macOS.zip"
LINUX_TAR="artifacts-linux/KatanA-linux-x86_64.tar.gz"
WINDOWS_ZIP="artifacts-windows/KatanA-windows-x86_64.zip"
WINDOWS_MSI="artifacts-windows/KatanA-windows-x86_64.msi"

MACOS_DMGS=(artifacts-macos/KatanA-Desktop-*.dmg)
[[ "${#MACOS_DMGS[@]}" -eq 1 ]] || {
    fail "expected exactly one macOS DMG, found ${#MACOS_DMGS[@]}"
}
MACOS_DMG="${MACOS_DMGS[0]}"

reject_unknown_entries artifacts-macos "KatanA-macOS.zip" "${MACOS_DMG##*/}"
reject_unknown_entries artifacts-linux "KatanA-linux-x86_64.tar.gz"
reject_unknown_entries artifacts-windows "KatanA-windows-x86_64.zip" "KatanA-windows-x86_64.msi"

for artifact in "$MACOS_ZIP" "$MACOS_DMG" "$LINUX_TAR" "$WINDOWS_ZIP" "$WINDOWS_MSI"; do
    require_nonempty_file "$artifact"
done

echo "📦 Organizing artifacts into ${OUTPUT_DIR}..."
mkdir -p "${OUTPUT_DIR}"
mv -- "$MACOS_ZIP" "$MACOS_DMG" "$LINUX_TAR" "$WINDOWS_ZIP" "$WINDOWS_MSI" "${OUTPUT_DIR}/"

cd "${OUTPUT_DIR}"

echo "🔐 Generating checksums.txt..."
shasum -a 256 \
    "$(basename "$MACOS_ZIP")" \
    "$(basename "$MACOS_DMG")" \
    "$(basename "$LINUX_TAR")" \
    "$(basename "$WINDOWS_ZIP")" \
    "$(basename "$WINDOWS_MSI")" > checksums.txt

echo "✅ Artifacts collected:"
ls -lh
