#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
COLLECTOR="${ROOT_DIR}/scripts/release/collect-artifacts.sh"
TMP_ROOT=$(mktemp -d -t katana-collect-artifacts.XXXXXX)

cleanup() {
    mkdir -p "$ROOT_DIR/tmp/trash"
    mv "$TMP_ROOT" "$ROOT_DIR/tmp/trash/"
}
trap cleanup EXIT

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

assert_file() {
    [[ -f "$1" ]] || fail "expected file is missing: $1"
}

assert_not_file() {
    [[ ! -e "$1" ]] || fail "unexpected file exists: $1"
}

populate_fixture() {
    local root="$1"
    mkdir -p "$root/artifacts-macos" "$root/artifacts-linux" "$root/artifacts-windows"
    printf 'mac zip collection fixture\n' >"$root/artifacts-macos/KatanA-macOS.zip"
    printf 'mac dmg collection fixture\n' >"$root/artifacts-macos/KatanA-Desktop-0.22.42.dmg"
    printf 'linux tar collection fixture\n' >"$root/artifacts-linux/KatanA-linux-x86_64.tar.gz"
    printf 'windows zip collection fixture\n' >"$root/artifacts-windows/KatanA-windows-x86_64.zip"
    printf 'windows msi collection fixture\n' >"$root/artifacts-windows/KatanA-windows-x86_64.msi"
}

run_collector() {
    local root="$1"
    (cd "$root" && "$COLLECTOR" release-files)
}

test_complete_collection() {
    local root="$TMP_ROOT/complete"
    populate_fixture "$root"
    run_collector "$root" >/dev/null
    for artifact in \
        KatanA-macOS.zip \
        KatanA-Desktop-0.22.42.dmg \
        KatanA-linux-x86_64.tar.gz \
        KatanA-windows-x86_64.zip \
        KatanA-windows-x86_64.msi; do
        assert_file "$root/release-files/$artifact"
    done
    assert_file "$root/release-files/checksums.txt"
    [[ "$(wc -l <"$root/release-files/checksums.txt" | tr -d ' ')" -eq 5 ]] || \
        fail "checksums.txt does not contain exactly five artifact entries"
    (cd "$root/release-files" && shasum -a 256 -c checksums.txt >/dev/null)
    [[ "$(find "$root/artifacts-macos" "$root/artifacts-linux" "$root/artifacts-windows" -type f | wc -l | tr -d ' ')" -eq 0 ]] || \
        fail "source artifacts were not fully collected"
}

test_rejects_missing_artifact() {
    local root="$TMP_ROOT/missing"
    populate_fixture "$root"
    mv "$root/artifacts-windows/KatanA-windows-x86_64.msi" "$root/missing.msi"
    if run_collector "$root" >/dev/null 2>&1; then
        fail "missing artifact was accepted"
    fi
    assert_not_file "$root/release-files/KatanA-macOS.zip"
}

test_rejects_empty_artifact() {
    local root="$TMP_ROOT/empty"
    populate_fixture "$root"
    : >"$root/artifacts-linux/KatanA-linux-x86_64.tar.gz"
    if run_collector "$root" >/dev/null 2>&1; then
        fail "empty artifact was accepted"
    fi
    assert_not_file "$root/release-files/KatanA-macOS.zip"
}

test_rejects_unknown_artifact() {
    local root="$TMP_ROOT/unknown"
    populate_fixture "$root"
    printf 'unexpected collection fixture\n' >"$root/artifacts-linux/unexpected.txt"
    if run_collector "$root" >/dev/null 2>&1; then
        fail "unknown artifact was accepted"
    fi
    assert_not_file "$root/release-files/KatanA-macOS.zip"
}

test_rejects_output_collision() {
    local root="$TMP_ROOT/collision"
    populate_fixture "$root"
    mkdir -p "$root/release-files"
    printf 'pre-existing release file\n' >"$root/release-files/existing.txt"
    if run_collector "$root" >/dev/null 2>&1; then
        fail "output collision was accepted"
    fi
    assert_file "$root/release-files/existing.txt"
    assert_file "$root/artifacts-macos/KatanA-macOS.zip"
}

test_rejects_symlinked_artifact() {
    local root="$TMP_ROOT/artifact-symlink"
    populate_fixture "$root"
    mv "$root/artifacts-linux/KatanA-linux-x86_64.tar.gz" "$root/linux-real.tar.gz"
    ln -s ../linux-real.tar.gz "$root/artifacts-linux/KatanA-linux-x86_64.tar.gz"
    if run_collector "$root" >/dev/null 2>&1; then
        fail "symlinked artifact was accepted"
    fi
    assert_not_file "$root/release-files/KatanA-macOS.zip"
}

test_rejects_symlinked_directories() {
    local root="$TMP_ROOT/directory-symlink"
    populate_fixture "$root"
    mv "$root/artifacts-linux" "$root/artifacts-linux-real"
    ln -s artifacts-linux-real "$root/artifacts-linux"
    if run_collector "$root" >/dev/null 2>&1; then
        fail "symlinked artifact directory was accepted"
    fi

    root="$TMP_ROOT/output-symlink"
    populate_fixture "$root"
    mkdir -p "$root/existing-output"
    ln -s existing-output "$root/release-files"
    if run_collector "$root" >/dev/null 2>&1; then
        fail "symlinked output directory was accepted"
    fi
}

test_rejects_hidden_entries() {
    local root="$TMP_ROOT/hidden-entry"
    populate_fixture "$root"
    printf 'hidden collection fixture\n' >"$root/artifacts-linux/.unexpected"
    if run_collector "$root" >/dev/null 2>&1; then
        fail "hidden artifact was accepted"
    fi

    root="$TMP_ROOT/hidden-output"
    populate_fixture "$root"
    mkdir -p "$root/release-files"
    printf 'hidden existing release file\n' >"$root/release-files/.existing"
    if run_collector "$root" >/dev/null 2>&1; then
        fail "hidden output collision was accepted"
    fi
}

test_complete_collection
test_rejects_missing_artifact
test_rejects_empty_artifact
test_rejects_unknown_artifact
test_rejects_output_collision
test_rejects_symlinked_artifact
test_rejects_symlinked_directories
test_rejects_hidden_entries
echo "PASS: collect-artifacts requires all five non-empty release assets and writes checksums"
