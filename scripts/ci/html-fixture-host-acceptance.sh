#!/usr/bin/env bash
# Runs the supplied HTML through the in-process host harness under an external
# supervisor. The supervisor is intentional: a renderer that blocks a single
# harness step cannot satisfy an in-process timeout.
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$ROOT_DIR"

: "${KATANA_HTML_FIXTURE:?KATANA_HTML_FIXTURE must name the supplied requirements HTML fixture}"
[[ -r "$KATANA_HTML_FIXTURE" ]] || {
    echo "KATANA_HTML_FIXTURE is not readable: $KATANA_HTML_FIXTURE" >&2
    exit 1
}

REQUEST="$ROOT_DIR/scripts/screenshot/examples/supplied-html-s15-host-acceptance.json"
CONTRACT="$ROOT_DIR/scripts/ci/html_fixture_host_contract.py"
if [[ "$#" -gt 1 || ( "$#" -eq 1 && "$1" != "--validate-input" ) ]]; then
    echo "Usage: html-fixture-host-acceptance.sh [--validate-input]" >&2
    exit 2
fi
if [[ "${1:-}" == "--validate-input" ]]; then
    python3 "$CONTRACT" "$KATANA_HTML_FIXTURE" "$REQUEST"
    exit
fi
# 別原本や異なるviewportを、同じ受入の証跡として記録しない。
python3 "$CONTRACT" "$KATANA_HTML_FIXTURE" "$REQUEST" >/dev/null
TARGET_DIR=${CARGO_TARGET_DIR:-"$ROOT_DIR/target"}
if [[ "$TARGET_DIR" != /* ]]; then
    TARGET_DIR="$ROOT_DIR/$TARGET_DIR"
fi
# The screenshot manifest has its own lock graph. Sharing the application's
# target directory can otherwise link stale proc-macro artifacts from either
# graph into the other build.
SCREENSHOT_TARGET_DIR="$TARGET_DIR/screenshot-harness"
RUNNER="$SCREENSHOT_TARGET_DIR/release/katana-screenshot"
OFFICE_WORKER="$TARGET_DIR/release/kdv-office-worker"
if [[ "${OS:-}" == "Windows_NT" ]]; then
    RUNNER+=".exe"
    OFFICE_WORKER+=".exe"
fi

OUTPUT_DIR=${KATANA_HTML_ACCEPTANCE_OUTPUT_DIR:-"$(mktemp -d "${TMPDIR:-/tmp}/katana-html-acceptance.XXXXXX")"}
mkdir -p "$OUTPUT_DIR"
LOG="$OUTPUT_DIR/runner.log"
EVIDENCE="$OUTPUT_DIR/evidence.txt"
source "$ROOT_DIR/scripts/release/packaged-process-identity.sh"

# Build is deliberately outside the 60-second document contract.
CARGO_TARGET_DIR="$TARGET_DIR" cargo build --locked --release -p katana-ui --bin kdv-office-worker
CARGO_TARGET_DIR="$SCREENSHOT_TARGET_DIR" cargo build --locked --release --manifest-path "$ROOT_DIR/scripts/screenshot/Cargo.toml"
[[ -x "$RUNNER" ]] || { echo "screenshot runner is not executable: $RUNNER" >&2; exit 1; }
[[ -x "$OFFICE_WORKER" ]] || { echo "release office worker is not executable: $OFFICE_WORKER" >&2; exit 1; }
python3 "$CONTRACT" "$KATANA_HTML_FIXTURE" "$REQUEST" >"$OUTPUT_DIR/input-identity.json"
printf 'mode=in_process_host\nsource_head=%s\nrunner_path=%s\nrunner_sha256=%s\nworker_path=%s\nworker_sha256=%s\nlock_sha256=%s\n' \
    "$(git rev-parse HEAD)" "$(packaged_canonical_path "$RUNNER")" "$(packaged_sha256 "$RUNNER")" \
    "$(packaged_canonical_path "$OFFICE_WORKER")" "$(packaged_sha256 "$OFFICE_WORKER")" \
    "$(packaged_sha256 "$ROOT_DIR/scripts/screenshot/Cargo.lock")" >"$OUTPUT_DIR/artifact-identity.txt"

KATANA_KDV_OFFICE_WORKER="$OFFICE_WORKER" "$RUNNER" --request "$REQUEST" --output "$OUTPUT_DIR" >"$LOG" 2>&1 &
RUNNER_PID=$!
started_epoch=$(date +%s)
max_cpu=0
frame_or_error=0

while kill -0 "$RUNNER_PID" 2>/dev/null; do
    elapsed=$(( $(date +%s) - started_epoch ))
    cpu=$(ps -o %cpu= -p "$RUNNER_PID" | tr -d ' ' || true)
    cpu=${cpu:-0}
    if awk "BEGIN { exit !($cpu > $max_cpu) }"; then
        max_cpu="$cpu"
    fi
    if rg -q 'HTML browser (first frame ready|did not produce an initial frame)|opening URL .* failed' "$LOG"; then
        frame_or_error=1
    fi
    if (( elapsed >= 60 )); then
        break
    fi
    sleep 1
done

elapsed=$(( $(date +%s) - started_epoch ))
if kill -0 "$RUNNER_PID" 2>/dev/null; then
    printf 'result=timeout_without_frame_or_error\nelapsed_seconds=%s\nmax_cpu_percent=%s\n' "$elapsed" "$max_cpu" >"$EVIDENCE"
    kill -TERM "$RUNNER_PID"
    wait "$RUNNER_PID" || true
    printf 'close=terminated_after_timeout\n' >>"$EVIDENCE"
    cat "$EVIDENCE" >&2
    exit 1
fi

set +e
wait "$RUNNER_PID"
status=$?
set -e
printf 'result=runner_exited\nexit_status=%s\nelapsed_seconds=%s\nmax_cpu_percent=%s\nframe_or_error_observed=%s\nclose=completed\n' \
    "$status" "$elapsed" "$max_cpu" "$frame_or_error" >"$EVIDENCE"
cat "$EVIDENCE"
[[ "$frame_or_error" == 1 ]] || { echo "runner exited without a frame or typed error" >&2; exit 1; }
exit "$status"
