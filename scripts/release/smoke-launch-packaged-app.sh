#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-}"
EXECUTABLE="${2:-}"
[[ -x "$EXECUTABLE" ]] || { echo "FAIL: executable not found: $EXECUTABLE" >&2; exit 1; }

HEARTBEAT=$(mktemp -t katana-startup-heartbeat.XXXXXX)
LOG_PATH=$(mktemp -t katana-startup-log.XXXXXX)
CONFIG_DIR=$(mktemp -d -t katana-startup-config.XXXXXX)
MAX_RSS_MIB=${KATANA_EMPTY_WORKSPACE_MAX_RSS_MIB:-512}
MAX_FONT_BYTES=${KATANA_EMPTY_WORKSPACE_MAX_FONT_BYTES:-134217728}
APP_PID=""

source "$(dirname "${BASH_SOURCE[0]}")/startup-heartbeat.sh"
source "$(dirname "${BASH_SOURCE[0]}")/packaged-process-identity.sh"

EXPECTED_EXECUTABLE=$(packaged_canonical_path "$EXECUTABLE")
EXPECTED_SHA256=$(packaged_sha256 "$EXPECTED_EXECUTABLE")
[[ "$EXPECTED_SHA256" =~ ^[[:xdigit:]]{64}$ ]] || {
    echo "FAIL: could not capture packaged executable SHA-256 ($MODE)" >&2
    exit 1
}

cleanup() {
    if [[ -n "$APP_PID" ]]; then
        kill "$APP_PID" 2>/dev/null || true
        wait "$APP_PID" 2>/dev/null || true
    fi
    rm -f "$HEARTBEAT" "$LOG_PATH"
    rm -rf "$CONFIG_DIR"
}
trap cleanup EXIT

process_tree_pids() {
    local parent="$1"
    echo "$parent"
    local child
    for child in $(pgrep -P "$parent" 2>/dev/null || true); do
        process_tree_pids "$child"
    done
}

case "$MODE" in
    linux-x86_64)
        KATANA_CONFIG_DIR="$CONFIG_DIR" KATANA_STARTUP_HEARTBEAT="$HEARTBEAT" DEBUG=true \
            timeout 30s xvfb-run -a "$EXECUTABLE" \
            >"$LOG_PATH" 2>&1 &
        ;;
    macos-arm64)
        [[ "$(uname -m)" == "arm64" ]] || {
            echo "FAIL: macos-arm64 smoke requires an arm64 runner" >&2
            exit 1
        }
        KATANA_CONFIG_DIR="$CONFIG_DIR" KATANA_STARTUP_HEARTBEAT="$HEARTBEAT" DEBUG=true \
            arch -arm64 "$EXECUTABLE" \
            >"$LOG_PATH" 2>&1 &
        ;;
    macos-x86_64)
        [[ "$(uname -m)" == "x86_64" ]] || {
            echo "FAIL: macos-x86_64 smoke requires an x86_64 runner" >&2
            exit 1
        }
        KATANA_CONFIG_DIR="$CONFIG_DIR" KATANA_STARTUP_HEARTBEAT="$HEARTBEAT" DEBUG=true \
            arch -x86_64 "$EXECUTABLE" \
            >"$LOG_PATH" 2>&1 &
        ;;
    *)
        echo "FAIL: unsupported startup smoke mode: $MODE" >&2
        exit 2
        ;;
esac

APP_PID=$!
READY=false
for _ in {1..40}; do
    if grep -qxF "first-frame" "$HEARTBEAT" && heartbeat_frame "$HEARTBEAT" >/dev/null; then
        READY=true
        break
    fi
    if ! kill -0 "$APP_PID" 2>/dev/null; then
        wait "$APP_PID" || true
        echo "FAIL: packaged app exited before first frame ($MODE)" >&2
        cat "$LOG_PATH" >&2
        exit 1
    fi
    sleep 0.5
done

if [[ "$READY" != "true" ]]; then
    echo "FAIL: packaged app did not reach first frame ($MODE)" >&2
    cat "$LOG_PATH" >&2
    exit 1
fi

LAST_HEARTBEAT_FRAME=$(heartbeat_frame "$HEARTBEAT")

MAIN_PID=""
for _ in {1..10}; do
    if MAIN_PID=$(packaged_find_matching_pid "$APP_PID" "$EXPECTED_EXECUTABLE"); then
        break
    fi
    sleep 0.1
done
if [[ -z "$MAIN_PID" ]]; then
    echo "FAIL: launched process image did not match packaged executable ($MODE)" >&2
    cat "$LOG_PATH" >&2
    exit 1
fi
if ! packaged_verify_process_identity "$MAIN_PID" "$EXPECTED_EXECUTABLE" "$EXPECTED_SHA256"; then
    echo "FAIL: running process identity mismatch ($MODE): expected=${EXPECTED_EXECUTABLE}; sha256=${EXPECTED_SHA256}" >&2
    exit 1
fi

PEAK_RSS_KIB=0
for _ in {1..10}; do
    sleep 0.2
    if ! LAST_HEARTBEAT_FRAME=$(require_heartbeat_advance "$HEARTBEAT" "$LAST_HEARTBEAT_FRAME"); then
        cat "$LOG_PATH" >&2
        exit 1
    fi
    PROCESS_IDS=$(process_tree_pids "$APP_PID" | paste -sd, -)
    RSS_KIB=$(ps -o rss= -p "$PROCESS_IDS" | awk '{ total += $1 } END { print total + 0 }')
    if (( RSS_KIB > PEAK_RSS_KIB )); then
        PEAK_RSS_KIB=$RSS_KIB
    fi
done
MAX_RSS_KIB=$((MAX_RSS_MIB * 1024))
if (( PEAK_RSS_KIB > MAX_RSS_KIB )); then
    echo "FAIL: empty-workspace RSS ${PEAK_RSS_KIB} KiB exceeds ${MAX_RSS_KIB} KiB ($MODE)" >&2
    exit 1
fi

PROCESS_IDS=$(process_tree_pids "$APP_PID")
while IFS= read -r pid; do
    PROCESS_NAME=$(ps -o comm= -p "$pid" 2>/dev/null || true)
    if [[ "$PROCESS_NAME" == *kdv-office-worker* ]]; then
        echo "FAIL: Office worker is live in an empty workspace ($MODE)" >&2
        exit 1
    fi
done <<<"$PROCESS_IDS"

FONT_BYTES=$(sed -n 's/.*event=ui_fonts_loaded .*owned_bytes=\([0-9][0-9]*\).*/\1/p' "$LOG_PATH" | tail -1)
if [[ -z "$FONT_BYTES" ]]; then
    echo "FAIL: UI-owned font metric was not emitted ($MODE)" >&2
    cat "$LOG_PATH" >&2
    exit 1
fi
if (( FONT_BYTES > MAX_FONT_BYTES )); then
    echo "FAIL: UI-owned fonts ${FONT_BYTES} bytes exceed ${MAX_FONT_BYTES} bytes ($MODE)" >&2
    exit 1
fi
if ! kill -0 "$MAIN_PID" 2>/dev/null || ! packaged_verify_process_identity "$MAIN_PID" "$EXPECTED_EXECUTABLE" "$EXPECTED_SHA256"; then
    echo "FAIL: packaged main process identity was not retained through smoke ($MODE)" >&2
    exit 1
fi

echo "OK: packaged app made continued UI progress ($MODE); executable=${EXPECTED_EXECUTABLE}; sha256=${EXPECTED_SHA256}; pid=${MAIN_PID}; peak_rss_kib=${PEAK_RSS_KIB}; font_bytes=${FONT_BYTES}; office_workers=0"
