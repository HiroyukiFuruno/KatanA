#!/usr/bin/env bash

heartbeat_frame() {
    local heartbeat="$1"
    local frame
    local attempt=0
    while [[ "$attempt" -lt 3 ]]; do
        frame=$(sed -n 's/^frame=\([0-9][0-9]*\)$/\1/p' "$heartbeat" | tail -1)
        if [[ "$frame" =~ ^[0-9]+$ ]]; then
            printf '%s\n' "$frame"
            return 0
        fi
        sleep 0.05
        attempt=$((attempt + 1))
    done
    return 1
}

require_heartbeat_advance() {
    local heartbeat="$1"
    local previous="$2"
    local current
    if ! current=$(heartbeat_frame "$heartbeat"); then
        echo "FAIL: UI heartbeat could not be read after retries; last_frame=${previous}" >&2
        return 1
    fi
    if (( current <= previous )); then
        echo "FAIL: UI heartbeat stalled after first frame; last_frame=${previous}; current_frame=${current}" >&2
        return 1
    fi
    printf '%s\n' "$current"
}
