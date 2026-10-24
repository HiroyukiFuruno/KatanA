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

source "$IDENTITY_HELPER"
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

verify_wait_grace_uses_elapsed_time() {
    local pid wait_started
    bash -c 'trap "" TERM; exec sleep 30' &
    pid=$!
    grace_pid="$pid"
    wait_started=$SECONDS
    sleep() {
        command sleep "$1"
        command sleep 0.2
    }
    if startup_wait_for_exit "$pid"; then
        echo 'FAIL: startup wait accepted a live TERM-ignoring process' >&2
        exit 1
    fi
    unset -f sleep
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
    local launched_pid owned_pid cleanup_started
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
    startup_process_cleanup >/dev/null
    (( SECONDS - cleanup_started <= 6 )) || {
        echo 'FAIL: startup cleanup exceeded its bounded termination grace' >&2
        exit 1
    }
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
