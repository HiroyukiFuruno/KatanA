#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
HELPER="$ROOT_DIR/scripts/ci/with-office-test-worker.sh"
TESTS_JUST="$ROOT_DIR/just/tests.just"
MAINTENANCE_JUST="$ROOT_DIR/just/maintenance.just"

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

[[ -f "$HELPER" ]] || fail "native Office worker helper is missing: $HELPER"
[[ -f "$TESTS_JUST" ]] || fail "test recipes are missing: $TESTS_JUST"
[[ -f "$MAINTENANCE_JUST" ]] || fail "maintenance recipes are missing: $MAINTENANCE_JUST"

HELPER_SOURCE=$(<"$HELPER")
for required_marker in \
    'cargo metadata --locked --format-version 1 --no-deps' \
    'target_directory' \
    'cargo build --locked -p katana-ui --bin kdv-office-worker' \
    '[[ -x "$OFFICE_WORKER" ]]' \
    'export KATANA_KDV_OFFICE_WORKER="$OFFICE_WORKER"'; do
    [[ "$HELPER_SOURCE" == *"$required_marker"* ]] || {
        fail "native Office worker helper must contain: $required_marker"
    }
done

for recipe_file in "$TESTS_JUST" "$MAINTENANCE_JUST"; do
    grep -qF 'with-office-test-worker.sh' "$recipe_file" || {
        fail "native test recipe must use the Office worker helper: $recipe_file"
    }
done

echo "PASS: native Office worker test contract is explicit"
