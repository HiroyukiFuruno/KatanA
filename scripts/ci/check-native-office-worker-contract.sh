#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
HELPER="$ROOT_DIR/scripts/ci/with-office-test-worker.sh"
TESTS_JUST="$ROOT_DIR/just/tests.just"
MAINTENANCE_JUST="$ROOT_DIR/just/maintenance.just"
WORKFLOW="$ROOT_DIR/.github/workflows/test-and-build.yml"

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

[[ -f "$HELPER" ]] || fail "native Office worker helper is missing: $HELPER"
[[ -f "$TESTS_JUST" ]] || fail "test recipes are missing: $TESTS_JUST"
[[ -f "$MAINTENANCE_JUST" ]] || fail "maintenance recipes are missing: $MAINTENANCE_JUST"
[[ -f "$WORKFLOW" ]] || fail "test workflow is missing: $WORKFLOW"

HELPER_SOURCE=$(<"$HELPER")
for required_marker in \
    'cargo metadata --locked --format-version 1 --no-deps' \
    'office_worker_target.py' \
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

RUN_TEST_STEP=$(awk '
    /^      - name: Run tests$/ { in_step = 1; print; next }
    in_step && /^      - name: / { exit }
    in_step { print }
' "$WORKFLOW")
[[ "$RUN_TEST_STEP" == *'shell: bash'* ]] || {
    fail "Run tests workflow step must use bash for the native Office worker helper"
}
[[ "$RUN_TEST_STEP" == *"bash scripts/ci/with-office-test-worker.sh 'cargo test --workspace'"* ]] || {
    fail "Run tests workflow step must invoke the native Office worker helper with bash for the full workspace suite"
}

python3 "$ROOT_DIR/scripts/ci/test_office_worker_target.py"
echo "PASS: native Office worker test contract is explicit"
