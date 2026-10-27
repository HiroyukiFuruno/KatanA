#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
SMOKE="$ROOT_DIR/scripts/release/smoke-launch-packaged-app.sh"
IDENTITY_HELPER="$ROOT_DIR/scripts/release/packaged-process-identity.sh"
CLEANUP_HELPER="$ROOT_DIR/scripts/release/startup-process-cleanup.sh"
EXPECTED_FAILURE='FAIL: unsupported startup smoke mode: invalid-mode'

bash -n "$SMOKE"
if OUTPUT=$(bash "$SMOKE" invalid-mode /usr/bin/false 2>&1); then
    echo 'FAIL: invalid startup mode was accepted' >&2
    exit 1
fi
[[ "$OUTPUT" == *"$EXPECTED_FAILURE"* ]] || {
    echo "FAIL: unexpected startup rejection: $OUTPUT" >&2
    exit 1
}
EVIDENCE_DIR=$(printf '%s\n' "$OUTPUT" | sed -n 's/^Startup evidence retained: //p')
[[ "$EVIDENCE_DIR" == "$ROOT_DIR/tmp/trash/"* ]] || {
    echo 'FAIL: startup evidence is outside the recoverable trash root' >&2
    exit 1
}
[[ -f "$EVIDENCE_DIR/heartbeat.txt" && -f "$EVIDENCE_DIR/startup.log" && -d "$EVIDENCE_DIR/config" ]] || {
    echo 'FAIL: startup evidence or isolated configuration was lost' >&2
    exit 1
}
if grep -Eq '^[[:space:]]*rm[[:space:]]' "$SMOKE"; then
    echo 'FAIL: startup smoke contains destructive cleanup' >&2
    exit 1
fi

grep -Fq 'startup-process-cleanup.sh' "$SMOKE" || {
    echo 'FAIL: startup smoke does not verify the main process before cleanup' >&2
    exit 1
}
grep -Fq 'terminate_verified_main' "$CLEANUP_HELPER" || {
    echo 'FAIL: cleanup helper does not verify the main process before cleanup' >&2
    exit 1
}
grep -Fq 'MAIN_PID=""' "$CLEANUP_HELPER" || {
    echo 'FAIL: startup smoke does not clear the verified main PID' >&2
    exit 1
}
grep -Fq 'APP_PID=""' "$CLEANUP_HELPER" || {
    echo 'FAIL: startup smoke does not clear the owned launcher PID' >&2
    exit 1
}

# shellcheck source=scripts/release/packaged-process-identity.sh
source "$IDENTITY_HELPER"
# shellcheck source=scripts/release/startup-process-cleanup.sh
source "$CLEANUP_HELPER"
launcher_pid=""
unrelated_pid=""
grace_pid=""
cleanup_process_regression() {
    [[ -z "$launcher_pid" ]] || kill "$launcher_pid" 2>/dev/null || true
    [[ -z "$launcher_pid" ]] || wait "$launcher_pid" 2>/dev/null || true
    [[ -z "$grace_pid" ]] || kill -KILL "$grace_pid" 2>/dev/null || true
    [[ -z "$grace_pid" ]] || wait "$grace_pid" 2>/dev/null || true
    [[ -z "$unrelated_pid" ]] || kill "$unrelated_pid" 2>/dev/null || true
    [[ -z "$unrelated_pid" ]] || wait "$unrelated_pid" 2>/dev/null || true
}
trap cleanup_process_regression EXIT

expected_path=$(packaged_canonical_path "$(command -v sleep)")
expected_sha=$(packaged_sha256 "$expected_path")
sleep 30 &
unrelated_pid=$!
EXPECTED_EXECUTABLE="$expected_path"
EXPECTED_SHA256="$expected_sha"

wait_for_owned_child() {
    local root_pid="$1"
    local candidate=""
    for _ in {1..40}; do
        if candidate=$(packaged_find_matching_pid "$root_pid" "$expected_path"); then
            printf '%s\n' "$candidate"
            return 0
        fi
        sleep 0.05
    done
    return 1
}

wait_for_exit() {
    local pid="$1"
    for _ in {1..40}; do
        kill -0 "$pid" 2>/dev/null || return 0
        sleep 0.05
    done
    return 1
}

sleep() {
    command sleep "$@"
    if [[ "${STARTUP_COARSE_POLL_SLEEP:-0}" == 1 ]]; then
        command sleep 0.2
    fi
}

run_cleanup_with_identity_trace() {
    local trace_path="$1"
    local saved_ps4="$PS4" was_xtrace=0
    [[ $- == *x* ]] && was_xtrace=1
    {
        PS4='+STARTUP_CLEANUP_TIME=${SECONDS}
+STARTUP_CLEANUP_TRACE: '
        set -x
        startup_process_cleanup >/dev/null
        if (( was_xtrace == 0 )); then
            set +x
        fi
        PS4="$saved_ps4"
    } 2>"$trace_path"
}

verify_wait_grace_uses_elapsed_time() {
    local pid wait_started STARTUP_COARSE_POLL_SLEEP=1
    bash -c 'trap "" TERM; exec sleep 30' &
    pid=$!
    grace_pid="$pid"
    wait_started=$SECONDS
    if startup_wait_for_exit "$pid"; then
        echo 'FAIL: startup wait accepted a live TERM-ignoring process' >&2
        exit 1
    fi
    (( SECONDS - wait_started <= 3 )) || {
        echo 'FAIL: startup wait counted polling attempts instead of elapsed termination grace' >&2
        exit 1
    }
    kill -KILL "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
    grace_pid=""
}

run_cleanup_case() {
    local main_mode="$1"
    local launched_pid owned_pid cleanup_started cleanup_finished cleanup_delta
    local trace_path expected_identity_calls actual_identity_calls caller_ps4="$PS4"
    local actual_term_calls actual_kill_calls
    local caller_xtrace_enabled=0 restored_xtrace_enabled=0
    [[ $- == *x* ]] && caller_xtrace_enabled=1
    EVIDENCE_DIR=$(mktemp -d -t katana-cleanup-evidence.XXXXXX)
    HEARTBEAT=$(mktemp -t katana-cleanup-heartbeat.XXXXXX)
    LOG_PATH=$(mktemp -t katana-cleanup-log.XXXXXX)
    CONFIG_DIR=$(mktemp -d -t katana-cleanup-config.XXXXXX)
    printf 'first-frame\nframe=1\n' >"$HEARTBEAT"
    printf 'real cleanup regression\n' >"$LOG_PATH"
    if [[ "$main_mode" == direct_ignoring ]]; then
        bash -c 'trap "" TERM; exec sleep 30' &
    elif [[ "$main_mode" == launcher_ignoring ]]; then
        python3 -c 'import signal, subprocess; signal.signal(signal.SIGTERM, signal.SIG_IGN); subprocess.Popen(["sleep", "30"]).wait(); signal.pause()' &
    elif [[ "$main_mode" == launcher_and_main_ignoring ]]; then
        STARTUP_CLEANUP_TERM_EVENT="$EVIDENCE_DIR/launcher-term-event" python3 -c '
import os
import signal
import subprocess

event_path = os.environ["STARTUP_CLEANUP_TERM_EVENT"]
child = None

def on_term(_signum, _frame):
    try:
        waited_pid, _status = os.waitpid(child.pid, os.WNOHANG)
    except ChildProcessError:
        waited_pid = child.pid
    with open(event_path, "w", encoding="utf-8") as event:
        event.write("dead-main" if waited_pid == child.pid else "live-main")

signal.signal(signal.SIGTERM, on_term)
child = subprocess.Popen(
    ["bash", "-c", "trap \"\" TERM; IFS= read -r _; exec sleep 30"],
    stdin=subprocess.PIPE,
)
child.stdin.write(b"go\n")
child.stdin.close()
child.wait()
while True:
    signal.pause()
' &
    else
        bash -c 'sleep 30 & wait' &
    fi
    APP_PID=$!
    launched_pid="$APP_PID"
    launcher_pid="$launched_pid"
    owned_pid=$(wait_for_owned_child "$launched_pid") || {
        echo 'FAIL: real launcher child was not observed before cleanup' >&2
        exit 1
    }
    if [[ "$main_mode" != discover ]]; then
        MAIN_PID="$owned_pid"
    else
        MAIN_PID=""
    fi
    cleanup_started=$SECONDS
    trace_path="$EVIDENCE_DIR/cleanup-identity-${main_mode}.trace"
    run_cleanup_with_identity_trace "$trace_path"
    cleanup_finished=$SECONDS
    cleanup_delta=$((cleanup_finished - cleanup_started))
    [[ $- == *x* ]] && restored_xtrace_enabled=1
    if [[ "$PS4" != "$caller_ps4" ]] || (( restored_xtrace_enabled != caller_xtrace_enabled )); then
        printf 'FAIL: cleanup tracing did not restore caller state: mode=%s start=%s end=%s actual_delta=%s tracepath=%s\n' \
            "$main_mode" "$cleanup_started" "$cleanup_finished" "$cleanup_delta" "$trace_path" >&2
        exit 1
    fi
    if (( cleanup_delta > 6 )); then
        if ! cat "$trace_path" >&2; then
            printf 'WARN: startup cleanup trace could not be read: %s\n' "$trace_path" >&2
        fi
        printf 'FAIL: startup cleanup exceeded its bounded termination grace: mode=%s start=%s end=%s actual_delta=%s tracepath=%s\n' \
            "$main_mode" "$cleanup_started" "$cleanup_finished" "$cleanup_delta" "$trace_path" >&2
        exit 1
    fi
    if ! grep -Eq '^\+STARTUP_CLEANUP_TIME=[0-9]+$' "$trace_path"; then
        if ! cat "$trace_path" >&2; then
            printf 'WARN: startup cleanup trace could not be read: %s\n' "$trace_path" >&2
        fi
        printf 'FAIL: startup cleanup trace has no Bash SECONDS marker: mode=%s tracepath=%s\n' \
            "$main_mode" "$trace_path" >&2
        exit 1
    fi
    if [[ "$main_mode" == direct_ignoring || "$main_mode" == launcher_ignoring || \
        "$main_mode" == launcher_and_main_ignoring ]]; then
        expected_identity_calls=2
    else
        expected_identity_calls=1
    fi
    actual_identity_calls=$(grep -Ec \
        "^\\+STARTUP_CLEANUP_TRACE: packaged_verify_process_identity ${owned_pid} " \
        "$trace_path" || true)
    if (( actual_identity_calls != expected_identity_calls )); then
        printf 'FAIL: startup cleanup identity protocol mismatch: mode=%s start=%s end=%s actual_delta=%s identity_calls=%s expected_calls=%s tracepath=%s\n' \
            "$main_mode" "$cleanup_started" "$cleanup_finished" "$cleanup_delta" \
            "$actual_identity_calls" "$expected_identity_calls" "$trace_path" >&2
        exit 1
    fi
    if [[ "$main_mode" == direct_ignoring ]]; then
        actual_term_calls=$(grep -Ec \
            "^\\+STARTUP_CLEANUP_TRACE: kill -TERM ${owned_pid}$" "$trace_path" || true)
        actual_kill_calls=$(grep -Ec \
            "^\\+STARTUP_CLEANUP_TRACE: kill -KILL ${owned_pid}$" "$trace_path" || true)
        if (( actual_term_calls != 1 || actual_kill_calls != 1 )); then
            printf 'FAIL: shared main/launcher PID received duplicate signals: TERM=%s KILL=%s\n' \
                "$actual_term_calls" "$actual_kill_calls" >&2
            exit 1
        fi
    fi
    printf 'PASS: startup cleanup identity protocol: mode=%s start=%s end=%s actual_delta=%s identity_calls=%s tracepath=%s\n' \
        "$main_mode" "$cleanup_started" "$cleanup_finished" "$cleanup_delta" \
        "$actual_identity_calls" "$trace_path"
    [[ -z "$APP_PID" && -z "$MAIN_PID" ]] || {
        echo 'FAIL: shared cleanup helper did not clear owned PIDs' >&2
        exit 1
    }
    wait_for_exit "$owned_pid" || {
        echo 'FAIL: shared cleanup helper did not terminate the owned child' >&2
        exit 1
    }
    wait_for_exit "$launched_pid" || {
        echo 'FAIL: shared cleanup helper did not terminate the owned wrapper' >&2
        exit 1
    }
    launcher_pid=""
    [[ -f "$EVIDENCE_DIR/heartbeat.txt" && -f "$EVIDENCE_DIR/startup.log" && -d "$EVIDENCE_DIR/config" ]] || {
        echo 'FAIL: shared cleanup helper did not retain evidence' >&2
        exit 1
    }
    if [[ "$main_mode" == launcher_and_main_ignoring ]]; then
        [[ -f "$EVIDENCE_DIR/launcher-term-event" ]] || {
            echo 'FAIL: owned launcher did not record when it received TERM' >&2
            exit 1
        }
        [[ "$(<"$EVIDENCE_DIR/launcher-term-event")" == live-main ]] || {
            printf 'FAIL: launcher received TERM only after main exited: event=%s\n' \
                "$(<"$EVIDENCE_DIR/launcher-term-event")" >&2
            exit 1
        }
    fi
    kill -0 "$unrelated_pid" 2>/dev/null || {
        echo 'FAIL: shared cleanup helper killed an unrelated process' >&2
        exit 1
    }
}

verify_wait_grace_uses_elapsed_time
run_cleanup_case verified
run_cleanup_case discover
run_cleanup_case direct_ignoring
run_cleanup_case launcher_ignoring
run_cleanup_case launcher_and_main_ignoring

if [[ "$(uname -m)" == arm64 ]]; then
    if early_output=$(bash "$SMOKE" macos-arm64 /usr/bin/false 2>&1); then
        echo 'FAIL: early-exit packaged smoke was accepted' >&2
        exit 1
    fi
    [[ "$early_output" == *'FAIL: packaged app exited before first frame (macos-arm64)'* ]] || {
        echo "FAIL: unexpected early-exit smoke output: $early_output" >&2
        exit 1
    }
fi
echo 'PASS: startup smoke retains failure evidence and isolated configuration'
