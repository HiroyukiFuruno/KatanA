#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
source "${ROOT_DIR}/scripts/release/packaged-process-identity.sh"

child=""
pid=""
cleanup_identity_test() {
    [[ -z "$pid" ]] || kill "$pid" 2>/dev/null || true
    [[ -z "$child" ]] || kill "$child" 2>/dev/null || true
    [[ -z "$child" ]] || wait "$child" 2>/dev/null || true
}
trap cleanup_identity_test EXIT

expected=$(packaged_canonical_path "$(command -v sleep)")
expected_sha=$(packaged_sha256 "$expected")

bash -c 'sleep 30 & wait' &
child=$!

for _ in {1..20}; do
    if pid=$(packaged_find_matching_pid "$child" "$expected"); then
        [[ "$pid" != "$child" ]]
        packaged_verify_process_identity "$pid" "$expected" "$expected_sha"
        if packaged_verify_process_identity "$pid" "/definitely/wrong" "$expected_sha"; then
            echo "FAIL: wrong expected path was accepted" >&2
            exit 1
        fi
        if packaged_verify_process_identity "$pid" "$expected" "0000000000000000000000000000000000000000000000000000000000000000"; then
            echo "FAIL: wrong expected hash was accepted" >&2
            exit 1
        fi
        kill "$pid"
        wait "$child" 2>/dev/null || true
        child=""
        if packaged_verify_process_identity "$pid" "$expected" "$expected_sha"; then
            echo "FAIL: exited PID was accepted" >&2
            exit 1
        fi
        if packaged_find_matching_pid "" "$expected"; then
            echo "FAIL: empty root PID was accepted" >&2
            exit 1
        fi
        echo "PASS: packaged process identity matched pid=${pid} sha256=${expected_sha}"
        pid=""
        exit 0
    fi
    sleep 0.05
done
echo "FAIL: local child process identity was not observable" >&2
exit 1
