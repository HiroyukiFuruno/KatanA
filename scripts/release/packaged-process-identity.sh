#!/usr/bin/env bash

packaged_sha256() {
    local path="$1"
    if command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$path" | awk '{print $1}'
    elif command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$path" | awk '{print $1}'
    else
        echo "FAIL: no SHA-256 utility is available" >&2
        return 1
    fi
}

packaged_canonical_path() {
    local path="$1"
    local candidate target depth=0
    [[ -f "$path" ]] || {
        echo "FAIL: packaged executable is not a file: $path" >&2
        return 1
    }
    if command -v realpath >/dev/null 2>&1; then
        realpath "$path"
        return
    fi
    candidate="$path"
    while [[ -L "$candidate" ]]; do
        depth=$((depth + 1))
        (( depth <= 32 )) || {
            echo "FAIL: executable symlink depth exceeds 32: $path" >&2
            return 1
        }
        target=$(readlink "$candidate") || return 1
        if [[ "$target" == /* ]]; then
            candidate="$target"
        else
            candidate="$(dirname "$candidate")/$target"
        fi
    done
    (cd -P "$(dirname "$candidate")" && printf '%s/%s\n' "$PWD" "$(basename "$candidate")")
}

packaged_process_image() {
    local pid="$1"
    [[ "$pid" =~ ^[1-9][0-9]*$ ]] || return 1
    if [[ -r "/proc/${pid}/exe" ]]; then
        readlink "/proc/${pid}/exe"
        return
    fi
    if command -v lsof >/dev/null 2>&1; then
        lsof -a -p "$pid" -d txt -Fn 2>/dev/null | sed -n 's/^n//p' | head -1
        return
    fi
    return 1
}

packaged_find_matching_pid() {
    local root_pid="$1"
    local expected_path="$2"
    local image child match
    [[ "$root_pid" =~ ^[1-9][0-9]*$ ]] || return 1
    image=$(packaged_process_image "$root_pid" || true)
    if [[ -n "$image" ]]; then
        image=$(packaged_canonical_path "$image" 2>/dev/null || true)
    fi
    if [[ -n "$image" && "$image" == "$expected_path" ]]; then
        printf '%s\n' "$root_pid"
        return 0
    fi
    while IFS= read -r child; do
        [[ -n "$child" ]] || continue
        if match=$(packaged_find_matching_pid "$child" "$expected_path"); then
            printf '%s\n' "$match"
            return 0
        fi
    done < <(pgrep -P "$root_pid" 2>/dev/null || true)
    return 1
}

packaged_verify_process_identity() {
    local pid="$1"
    local expected_path="$2"
    local expected_sha256="$3"
    local actual_image actual_path actual_sha256
    actual_image=$(packaged_process_image "$pid" || true)
    [[ -n "$actual_image" ]] || return 1
    actual_path=$(packaged_canonical_path "$actual_image" 2>/dev/null || true)
    [[ "$actual_path" == "$expected_path" ]] || return 1
    actual_sha256=$(packaged_sha256 "$actual_path" 2>/dev/null || true)
    [[ "$actual_sha256" == "$expected_sha256" ]]
}
