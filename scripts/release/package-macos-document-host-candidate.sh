#!/usr/bin/env bash
# 実package前後を封じ、古いbundleのsourceを現在のHEADから推定しない。
set -euo pipefail
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
EXPECTED=${1:-}
TARGET=${2:-}
[[ "$EXPECTED" =~ ^[0-9a-f]{40}$ && "$TARGET" =~ ^macos-(arm64|x86_64)$ ]] || {
  echo 'usage: package-macos-document-host-candidate.sh <expected-clean-commit> <macos-arm64|macos-x86_64>' >&2
  exit 2
}
cd "$ROOT"
[[ "$(rtk proxy git rev-parse HEAD)" == "$EXPECTED" ]]
[[ -z "$(rtk proxy git status --porcelain=v1)" ]]
VERSION=$(rtk proxy python3 scripts/release/document_host_version.py Cargo.toml)
STAGE=$(rtk proxy mktemp -d "$ROOT/tmp/macos-host-candidate.XXXXXX")
SNAPSHOT="$ROOT/scripts/release/record-document-host-snapshot.py"
rtk proxy python3 "$SNAPSHOT" --root "$ROOT" source --output "$STAGE/source.before.json"
set +e
rtk proxy env JOBS=1 CARGO_BUILD_JOBS=1 just VERSION="$VERSION" package-mac >"$STAGE/package.log" 2>&1
PACKAGE_EXIT=$?
set -e
printf '%s\n' "$PACKAGE_EXIT" >"$STAGE/package.exit"
[[ "$PACKAGE_EXIT" == 0 ]] || exit "$PACKAGE_EXIT"
[[ "$(rtk proxy git rev-parse HEAD)" == "$EXPECTED" ]]
[[ -z "$(rtk proxy git status --porcelain=v1)" ]]
rtk proxy python3 "$SNAPSHOT" --root "$ROOT" source --output "$STAGE/source.after.json"
rtk proxy cmp "$STAGE/source.before.json" "$STAGE/source.after.json"
BUNDLE="$ROOT/target/release/bundle/osx/KatanA Desktop.app"
rtk proxy python3 scripts/release/document_host_version.py "$BUNDLE/Contents/Info.plist" "$VERSION"
rtk proxy codesign --verify --deep --strict "$BUNDLE"
rtk proxy ditto -c -k --keepParent "$BUNDLE" "$STAGE/KatanA-macOS.zip"
rtk proxy bash scripts/release/check-release-asset-contract.sh macos "$STAGE/KatanA-macOS.zip" >"$STAGE/asset-contract.log" 2>&1
rtk proxy mkdir "$STAGE/unpacked"
rtk proxy ditto -x -k "$STAGE/KatanA-macOS.zip" "$STAGE/unpacked"
rtk proxy python3 scripts/release/document_host_version.py "$STAGE/unpacked/KatanA Desktop.app/Contents/Info.plist" "$VERSION"
rtk proxy codesign --verify --deep --strict "$STAGE/unpacked/KatanA Desktop.app" >"$STAGE/codesign.log" 2>&1
rtk proxy python3 "$SNAPSHOT" --root "$ROOT" bundle --package-source-before "$STAGE/source.before.json" --package-source-after "$STAGE/source.after.json" --archive "$STAGE/KatanA-macOS.zip" --main "$STAGE/unpacked/KatanA Desktop.app/Contents/MacOS/KatanA" --worker "$STAGE/unpacked/KatanA Desktop.app/Contents/MacOS/kdv-office-worker" --target "$TARGET" --output "$STAGE/bundle.json"
printf 'HOST_CANDIDATE_STAGE=%s\n' "$STAGE"
