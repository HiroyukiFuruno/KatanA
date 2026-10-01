#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
TESTS_JUST="$ROOT_DIR/just/tests.just"
CHECK_LINUX_RECIPE=$(sed -n '/^check-linux:/,/^$/p' "$TESTS_JUST")
EXPECTED_CHAIN='cargo build --locked -p katana-ui --bin kdv-office-worker && KATANA_KDV_OFFICE_WORKER=/app/target/debug/kdv-office-worker cargo test --locked -q --workspace'

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

[[ -f "$TESTS_JUST" ]] || fail "Linux test recipe is missing: $TESTS_JUST"
[[ "$CHECK_LINUX_RECIPE" == *"$EXPECTED_CHAIN"* ]] || {
    fail "check-linux must build the Office worker and pass its absolute debug path to workspace tests"
}

echo "PASS: check-linux Office worker contract is explicit"
