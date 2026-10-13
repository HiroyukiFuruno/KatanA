#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$ROOT_DIR"

[[ $# -eq 1 ]] || {
    echo "FAIL: expected one test command" >&2
    exit 1
}

TARGET_DIR=$(cargo metadata --locked --format-version 1 --no-deps | python3 "$ROOT_DIR/scripts/ci/office_worker_target.py")

cargo build --locked -p katana-ui --bin kdv-office-worker

case "${OS:-}:${OSTYPE:-}" in
    Windows_NT:*|*:msys*|*:cygwin*) OFFICE_WORKER_NAME="kdv-office-worker.exe" ;;
    *) OFFICE_WORKER_NAME="kdv-office-worker" ;;
esac
OFFICE_WORKER="$TARGET_DIR/debug/$OFFICE_WORKER_NAME"
[[ -x "$OFFICE_WORKER" ]] || {
    echo "FAIL: Office worker is missing or not executable: $OFFICE_WORKER" >&2
    exit 1
}

export KATANA_KDV_OFFICE_WORKER="$OFFICE_WORKER"
exec bash -c "$1"
