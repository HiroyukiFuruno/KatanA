#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat >&2 <<'EOF'
Usage: observe-packaged-office-process.sh <main-pid> <main-executable> <sidecar-executable> <output-json>

Collects kernel process identity for a packaged main process and its expected
Office sidecar descendant for at most 30 seconds. The JSON is a kernel identity
diagnostic only; it does not prove Office rendering success. UI interaction,
Terms acceptance, clean-machine state, and normal close are not performed or
claimed by this command.
EOF
}

if [[ "$#" -ne 4 ]]; then
    usage
    exit 2
fi

MAIN_PID="$1"
MAIN_EXECUTABLE="$2"
SIDECAR_EXECUTABLE="$3"
OUTPUT_JSON="$4"

[[ "$MAIN_PID" =~ ^[1-9][0-9]*$ ]] || {
    echo "FAIL: invalid main PID: $MAIN_PID" >&2
    exit 2
}
[[ -x "$MAIN_EXECUTABLE" ]] || {
    echo "FAIL: main executable is not executable: $MAIN_EXECUTABLE" >&2
    exit 2
}
[[ -x "$SIDECAR_EXECUTABLE" ]] || {
    echo "FAIL: sidecar executable is not executable: $SIDECAR_EXECUTABLE" >&2
    exit 2
}

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "${SCRIPT_DIR}/packaged-process-identity.sh"

EXPECTED_MAIN_PATH=$(packaged_canonical_path "$MAIN_EXECUTABLE")
EXPECTED_SIDECAR_PATH=$(packaged_canonical_path "$SIDECAR_EXECUTABLE")
EXPECTED_MAIN_SHA256=$(packaged_sha256 "$EXPECTED_MAIN_PATH")
EXPECTED_SIDECAR_SHA256=$(packaged_sha256 "$EXPECTED_SIDECAR_PATH")

if ! packaged_verify_process_identity "$MAIN_PID" "$EXPECTED_MAIN_PATH" "$EXPECTED_MAIN_SHA256"; then
    echo "FAIL: main process identity mismatch: pid=${MAIN_PID}; expected=${EXPECTED_MAIN_PATH}" >&2
    exit 1
fi

if ! kill -0 "$MAIN_PID" 2>/dev/null; then
    echo "FAIL: main process is no longer alive: pid=${MAIN_PID}" >&2
    exit 1
fi

SIDECAR_PID=""
DEADLINE=$(( $(date +%s) + 30 ))
while (( $(date +%s) < DEADLINE )); do
    if ! kill -0 "$MAIN_PID" 2>/dev/null; then
        echo "FAIL: main process died before the Office sidecar was observed: pid=${MAIN_PID}" >&2
        exit 1
    fi

    SIDECAR_PID=$(packaged_find_matching_pid "$MAIN_PID" "$EXPECTED_SIDECAR_PATH" || true)
    if [[ -n "$SIDECAR_PID" ]]; then
        if [[ "$SIDECAR_PID" == "$MAIN_PID" ]]; then
            echo "FAIL: sidecar identity resolved to the main PID: pid=${MAIN_PID}" >&2
            exit 1
        fi
        if ! packaged_verify_process_identity \
            "$SIDECAR_PID" "$EXPECTED_SIDECAR_PATH" "$EXPECTED_SIDECAR_SHA256"; then
            echo "FAIL: sidecar process identity mismatch: pid=${SIDECAR_PID}; expected=${EXPECTED_SIDECAR_PATH}" >&2
            exit 1
        fi
        break
    fi
    sleep 0.1
done

if [[ -z "$SIDECAR_PID" ]]; then
    echo "FAIL: expected Office sidecar was not observed within 30 seconds" >&2
    exit 1
fi
if ! kill -0 "$MAIN_PID" 2>/dev/null; then
    echo "FAIL: main process died after the Office sidecar was observed: pid=${MAIN_PID}" >&2
    exit 1
fi

MAIN_IMAGE=$(packaged_process_image "$MAIN_PID")
SIDECAR_IMAGE=$(packaged_process_image "$SIDECAR_PID")
MAIN_PATH=$(packaged_canonical_path "$MAIN_IMAGE")
SIDECAR_PATH=$(packaged_canonical_path "$SIDECAR_IMAGE")
MAIN_SHA256=$(packaged_sha256 "$MAIN_PATH")
SIDECAR_SHA256=$(packaged_sha256 "$SIDECAR_PATH")

if [[ "$MAIN_PATH" != "$EXPECTED_MAIN_PATH" || "$MAIN_SHA256" != "$EXPECTED_MAIN_SHA256" \
    || "$SIDECAR_PATH" != "$EXPECTED_SIDECAR_PATH" || "$SIDECAR_SHA256" != "$EXPECTED_SIDECAR_SHA256" ]]; then
    echo "FAIL: process image or artifact changed during identity collection" >&2
    exit 1
fi
if [[ "$(packaged_find_matching_pid "$MAIN_PID" "$EXPECTED_SIDECAR_PATH" || true)" != "$SIDECAR_PID" ]]; then
    echo "FAIL: observed sidecar is no longer the expected main descendant" >&2
    exit 1
fi

MAIN_PID="$MAIN_PID" \
MAIN_PATH="$MAIN_PATH" \
MAIN_SHA256="$MAIN_SHA256" \
SIDECAR_PID="$SIDECAR_PID" \
SIDECAR_PATH="$SIDECAR_PATH" \
SIDECAR_SHA256="$SIDECAR_SHA256" \
OUTPUT_JSON="$OUTPUT_JSON" \
python3 - "$OUTPUT_JSON" <<'PY'
import json
import os
from datetime import datetime, timezone
from pathlib import Path

payload = {
    "mode": "packaged_main",
    "main_pid": int(os.environ["MAIN_PID"]),
    "main_path": os.environ["MAIN_PATH"],
    "main_sha256": os.environ["MAIN_SHA256"],
    "sidecar_pid": int(os.environ["SIDECAR_PID"]),
    "sidecar_path": os.environ["SIDECAR_PATH"],
    "sidecar_sha256": os.environ["SIDECAR_SHA256"],
    "observed_at": datetime.now(timezone.utc).isoformat(),
    "clean_machine": None,
    "normal_close": None,
}
output = Path(os.environ["OUTPUT_JSON"])
output.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
PY

echo "OK: packaged main and Office sidecar kernel identities observed; output=${OUTPUT_JSON}"
