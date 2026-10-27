#!/usr/bin/env bash

launcher_is_owned_child() {
    local pid="$1"
    local parent
    [[ "$pid" =~ ^[1-9][0-9]*$ ]] || return 1
    parent=$(ps -o ppid= -p "$pid" 2>/dev/null | tr -d '[:space:]')
    [[ "$parent" == "$$" ]]
}

startup_wait_for_exit() {
    local pid deadline_pid exit_trap all_exited
    # sleep の粒度が粗い環境でも、試行回数ではなく実時間で終了猶予を制限する。
    exit_trap=$(trap -p EXIT)
    trap - EXIT
    command sleep 2 &
    deadline_pid=$!
    if [[ -n "$exit_trap" ]]; then
        eval "$exit_trap"
    fi
    while :; do
        all_exited=1
        for pid in "$@"; do
            if kill -0 "$pid" 2>/dev/null; then
                all_exited=0
            else
                wait "$pid" 2>/dev/null || true
            fi
        done
        if (( all_exited )); then
            kill "$deadline_pid" 2>/dev/null || true
            wait "$deadline_pid" 2>/dev/null || true
            return 0
        fi
        if ! kill -0 "$deadline_pid" 2>/dev/null; then
            wait "$deadline_pid" 2>/dev/null || true
            return 1
        fi
        sleep 0.05
    done
}

startup_terminate_process() {
    local pid identity_check count=0 i duplicate
    local -a pids identity_checks
    while (( $# > 0 )); do
        pid="$1"
        identity_check="$2"
        shift 2
        duplicate=0
        i=0
        while (( i < count )); do
            [[ "${pids[$i]}" != "$pid" ]] || duplicate=1
            i=$((i + 1))
        done
        # 本体がlauncher自身でも、同じPIDへ同じphaseのsignalを重ねない。
        (( duplicate == 0 )) || continue
        "$identity_check" "$pid" || continue
        pids[count]="$pid"
        identity_checks[count]="$identity_check"
        count=$((count + 1))
        kill -TERM "$pid" 2>/dev/null || true
    done
    (( count > 0 )) || return 0
    # 本体と所有launcherの待ちを共有し、各対象の猶予を直列に累積させない。
    startup_wait_for_exit "${pids[@]}" && return 0
    i=0
    while (( i < count )); do
        pid="${pids[$i]}"
        # 終了待ちの間に対象が変わっていないことを確認してから強制終了する。
        if kill -0 "$pid" 2>/dev/null && "${identity_checks[$i]}" "$pid"; then
            kill -KILL "$pid" 2>/dev/null || true
        fi
        i=$((i + 1))
    done
    if ! startup_wait_for_exit "${pids[@]}"; then
        for pid in "${pids[@]}"; do
            if kill -0 "$pid" 2>/dev/null; then
                echo "WARN: startup cleanup could not reap PID $pid" >&2
            fi
        done
    fi
}

startup_main_identity() {
    packaged_verify_process_identity "$1" "$EXPECTED_EXECUTABLE" "$EXPECTED_SHA256"
}

terminate_verified_main() {
    local candidate="${MAIN_PID:-}"
    if [[ -z "$candidate" && -n "${APP_PID:-}" ]] && launcher_is_owned_child "$APP_PID"; then
        candidate=$(packaged_find_matching_pid "$APP_PID" "$EXPECTED_EXECUTABLE" || true)
    fi
    if [[ -n "$candidate" ]] && kill -0 "$candidate" 2>/dev/null; then
        startup_terminate_process "$candidate" startup_main_identity "$@"
    else
        startup_terminate_process "$@"
    fi
    MAIN_PID=""
}

startup_process_cleanup() {
    if [[ -n "${APP_PID:-}" ]] && launcher_is_owned_child "$APP_PID"; then
        terminate_verified_main "$APP_PID" launcher_is_owned_child
    else
        terminate_verified_main
    fi
    APP_PID=""
    mv "$HEARTBEAT" "$EVIDENCE_DIR/heartbeat.txt"
    mv "$LOG_PATH" "$EVIDENCE_DIR/startup.log"
    mv "$CONFIG_DIR" "$EVIDENCE_DIR/config"
    echo "Startup evidence retained: $EVIDENCE_DIR"
}
