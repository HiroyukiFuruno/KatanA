#!/bin/bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
GUARD="$ROOT_DIR/scripts/release/check-macos-coverage-contract.sh"
TMP_ROOT=$(mktemp -d)
trap 'rm -rf "$TMP_ROOT"' EXIT

printf '%s\n' 'JOBS=1 just coverage' >"$TMP_ROOT/serial.yml"
printf '%s\n' 'just coverage' >"$TMP_ROOT/parallel.yml"

KATANA_CI_WORKFLOW_FILE="$TMP_ROOT/serial.yml" bash "$GUARD" >/dev/null
if KATANA_CI_WORKFLOW_FILE="$TMP_ROOT/parallel.yml" bash "$GUARD" >/dev/null 2>&1; then
    echo "FAIL: parallel macOS coverage was accepted" >&2
    exit 1
fi

echo "PASS: macOS coverage concurrency contract rejects parallel linking"
