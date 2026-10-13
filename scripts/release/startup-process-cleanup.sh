#!/usr/bin/env bash

launcher_is_owned_child() {
    local pid="$1"
    local parent
    [[ "$pid" =~ ^[1-9][0-9]*$ ]] || return 1
    parent=$(ps -o ppid= -p "$pid" 2>/dev/null | tr -d '[:space:]')
    [[ "$parent" == "$$" ]]
}

terminate_verified_main() {
    local candidate="${MAIN_PID:-}"
    if [[ -z "$candidate" && -n "${APP_PID:-}" ]] && launcher_is_owned_child "$APP_PID"; then
        candidate=$(packaged_find_matching_pid "$APP_PID" "$EXPECTED_EXECUTABLE" || true)
    fi
    if [[ -n "$candidate" ]] && kill -0 "$candidate" 2>/dev/null \
        && packaged_verify_process_identity "$candidate" "$EXPECTED_EXECUTABLE" "$EXPECTED_SHA256"; then
        kill "$candidate" 2>/dev/null || true
        wait "$candidate" 2>/dev/null || true
    fi
    MAIN_PID=""
}

startup_process_cleanup() {
    terminate_verified_main
    if [[ -n "${APP_PID:-}" ]]; then
        if launcher_is_owned_child "$APP_PID"; then
            kill "$APP_PID" 2>/dev/null || true
            wait "$APP_PID" 2>/dev/null || true
        fi
        APP_PID=""
    fi
    mv "$HEARTBEAT" "$EVIDENCE_DIR/heartbeat.txt"
    mv "$LOG_PATH" "$EVIDENCE_DIR/startup.log"
    mv "$CONFIG_DIR" "$EVIDENCE_DIR/config"
    echo "Startup evidence retained: $EVIDENCE_DIR"
}
