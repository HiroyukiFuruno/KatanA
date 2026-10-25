#!/bin/zsh
# =============================================================================
# KatanA — macOS Packaging (App Bundle)
# =============================================================================

set -euo pipefail

# ── Configuration ─────────────────────────────────────────────────────────────
APP_NAME="KatanA Desktop"
APP_BUNDLE="target/release/bundle/osx/${APP_NAME}.app"
CONTENTS="${APP_BUNDLE}/Contents"
MACOS_TARGETS=("aarch64-apple-darwin" "x86_64-apple-darwin")
export MACOSX_DEPLOYMENT_TARGET="13.0"

# ── Colours ──────────────────────────────────────────────────────────────────
GREEN='\033[0;32m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

# ── Helpers ───────────────────────────────────────────────────────────────────
info()    { echo "${CYAN}[INFO]${RESET}  $*"; }
success() { echo "${GREEN}[OK]${RESET}    $*"; }

# ── Argument Validation ───────────────────────────────────────────────────────
VERSION=$1
if [[ -z "$VERSION" ]]; then
    # Fallback to Cargo.toml version if not provided
    VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
fi

# ── Execution ─────────────────────────────────────────────────────────────────
info "Packaging macOS .app bundle (release)..."

info "Building universal macOS executables..."
for target in "${MACOS_TARGETS[@]}"; do
    cargo build --release --target "$target" --package katana-ui --bin KatanA --bin kdv-office-worker
done

# 既に生成したARM構成を再利用し、同じ依存関係の三重ビルドを避ける。
ARM64_APP_BUNDLE="target/aarch64-apple-darwin/release/bundle/osx/${APP_NAME}.app"
TRASH_ROOT="tmp/trash"
mkdir -p "$(dirname "$APP_BUNDLE")" "$TRASH_ROOT"
BACKUP_DIR=$(mktemp -d "${TRASH_ROOT}/macos-package.XXXXXX")

# 古いbundleの余剰ファイルを混入させず、失敗時も以前の生成物を復元できるよう退避する。
for candidate in "$APP_BUNDLE" "$ARM64_APP_BUNDLE"; do
    if [[ -L "$candidate" ]]; then
        echo "Refusing symlink bundle path: $candidate" >&2
        exit 1
    fi
    if [[ -e "$candidate" ]]; then
        [[ -d "$candidate" ]] || { echo "Refusing non-directory bundle path: $candidate" >&2; exit 1; }
        if [[ "$candidate" == "$APP_BUNDLE" ]]; then
            destination="${BACKUP_DIR}/canonical.app"
        else
            destination="${BACKUP_DIR}/aarch64-target.app"
        fi
        mv "$candidate" "$destination"
    fi
done

cargo bundle --release --target aarch64-apple-darwin --format osx --package katana-ui --bin KatanA
[[ -d "$ARM64_APP_BUNDLE" && ! -L "$ARM64_APP_BUNDLE" ]] || {
    echo "cargo-bundle did not produce the expected aarch64 app bundle: $ARM64_APP_BUNDLE" >&2
    exit 1
}
[[ ! -e "$APP_BUNDLE" && ! -L "$APP_BUNDLE" ]] || {
    echo "Canonical bundle destination unexpectedly exists: $APP_BUNDLE" >&2
    exit 1
}
mv "$ARM64_APP_BUNDLE" "$APP_BUNDLE"

info "Overlaying project-specific Info.plist..."
cp crates/katana-ui/Info.plist "${CONTENTS}/Info.plist"

info "Syncing version v${VERSION} to bundle Info.plist..."
perl -i -0pe 's/(<key>CFBundleShortVersionString<\/key>\s*<string>).*?(<\/string>)/$1v'"${VERSION}"'$2/' "${CONTENTS}/Info.plist"

info "Adding Resources (icon.icns)..."
mkdir -p "${CONTENTS}/Resources"
cp assets/icon.icns "${CONTENTS}/Resources/icon.icns"

info "Adding the isolated Office worker..."
lipo -create \
    target/aarch64-apple-darwin/release/KatanA \
    target/x86_64-apple-darwin/release/KatanA \
    -output "${CONTENTS}/MacOS/KatanA"
lipo -create \
    target/aarch64-apple-darwin/release/kdv-office-worker \
    target/x86_64-apple-darwin/release/kdv-office-worker \
    -output "${CONTENTS}/MacOS/kdv-office-worker"
chmod 755 "${CONTENTS}/MacOS/KatanA"
chmod 755 "${CONTENTS}/MacOS/kdv-office-worker"
for architecture in arm64 x86_64; do
    lipo "${CONTENTS}/MacOS/KatanA" -verify_arch "$architecture"
    lipo "${CONTENTS}/MacOS/kdv-office-worker" -verify_arch "$architecture"
done

info "Applying Ad-hoc Code Signature (Required after modifying Info.plist to prevent 'damaged' Gatekeeper error)..."
codesign --force --deep --sign - "${APP_BUNDLE}"

success "Created ${APP_BUNDLE}"
