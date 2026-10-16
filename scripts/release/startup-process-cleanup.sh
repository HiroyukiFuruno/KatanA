#!/usr/bin/env bash

launcher_is_owned_child() {
    local pid="$1"
    local parent
    [[ "$pid" =~ ^[1-9][0-9]*$ ]] || return 1
    parent=$(ps -o ppid= -p "$pid" 2>/dev/null | tr -d '[:space:]')
    [[ "$parent" == "$$" ]]
}

startup_wait_for_exit() {
    local pid="$1" attempt
    for attempt in {1..40}; do
        if ! kill -0 "$pid" 2>/dev/null; then
            wait "$pid" 2>/dev/null || true
            return 0
        fi
        sleep 0.05
    done
    return 1
}

startup_terminate_process() {
    local pid="$1" identity_check="$2"
    "$identity_check" "$pid" || return 0
    kill -TERM "$pid" 2>/dev/null || true
    startup_wait_for_exit "$pid" && return 0
    # 終了待ちの間に対象が変わっていないことを確認してから強制終了する。
    if "$identity_check" "$pid"; then
        kill -KILL "$pid" 2>/dev/null || true
    fi
    startup_wait_for_exit "$pid" || echo "WARN: startup cleanup could not reap PID $pid" >&2
}

startup_main_identity() {
    packaged_verify_process_identity "$1" "$EXPECTED_EXECUTABLE" "$EXPECTED_SHA256"
}

terminate_verified_main() {
    local candidate="${MAIN_PID:-}"
    if [[ -z "$candidate" && -n "${APP_PID:-}" ]] && launcher_is_owned_child "$APP_PID"; then
        candidate=$(packaged_find_matching_pid "$APP_PID" "$EXPECTED_EXECUTABLE" || true)
    fi
    if [[ -n "$candidate" ]] && kill -0 "$candidate" 2>/dev/null \
        && packaged_verify_process_identity "$candidate" "$EXPECTED_EXECUTABLE" "$EXPECTED_SHA256"; then
        startup_terminate_process "$candidate" startup_main_identity
    fi
    MAIN_PID=""
}

startup_process_cleanup() {
    terminate_verified_main
    if [[ -n "${APP_PID:-}" ]]; then
        if launcher_is_owned_child "$APP_PID"; then
            startup_terminate_process "$APP_PID" launcher_is_owned_child
        fi
        APP_PID=""
    fi
    mv "$HEARTBEAT" "$EVIDENCE_DIR/heartbeat.txt"
    mv "$LOG_PATH" "$EVIDENCE_DIR/startup.log"
    mv "$CONFIG_DIR" "$EVIDENCE_DIR/config"
    echo "Startup evidence retained: $EVIDENCE_DIR"
}
