#!/bin/bash
set -euo pipefail

COMMIT_SHA=${1:-}
MAX_ATTEMPTS=${CI_GATE_WAIT_ATTEMPTS:-60}
WAIT_SECONDS=${CI_GATE_WAIT_SECONDS:-30}
VERSION_FILE_PATTERN='^(Cargo\.toml|Cargo\.lock|crates/katana-ui/Info\.plist)$'

error() { printf '[ERROR] %s\n' "$*" >&2; }
info() { printf '[INFO]  %s\n' "$*"; }
success() { printf '[OK]    %s\n' "$*"; }

if [ -z "$COMMIT_SHA" ]; then
    error "Commit SHA is required."
    exit 1
fi

if ! git rev-parse "${COMMIT_SHA}^" >/dev/null 2>&1; then
    error "Cannot inspect parent commit for ${COMMIT_SHA}."
    exit 1
fi

CHANGED_FILES=$(git diff --name-only "${COMMIT_SHA}^" "$COMMIT_SHA")
if ! printf '%s\n' "$CHANGED_FILES" | grep -Eq "$VERSION_FILE_PATTERN"; then
    success "No version file changes detected in ${COMMIT_SHA}; CI gate wait is not required."
    exit 0
fi

info "Version file changes detected in ${COMMIT_SHA}; waiting for CI success on the same commit."

for ATTEMPT in $(seq 1 "$MAX_ATTEMPTS"); do
    RUN_RECORD=$(gh run list \
        --workflow test-and-build.yml \
        --commit "$COMMIT_SHA" \
        --event push \
        --limit 20 \
        --json status,conclusion,url,headSha \
        --jq "map(select(.headSha == \"${COMMIT_SHA}\")) | first // empty | [.status, (.conclusion // \"\"), .url] | @tsv")

    if [ -n "$RUN_RECORD" ]; then
        IFS=$'\t' read -r STATUS CONCLUSION URL <<<"$RUN_RECORD"
        if [ "$STATUS" = "completed" ] && [ "$CONCLUSION" = "success" ]; then
            success "CI passed for ${COMMIT_SHA}: ${URL}"
            exit 0
        fi

        if [ "$STATUS" = "completed" ]; then
            error "CI completed without success for ${COMMIT_SHA}: ${CONCLUSION} (${URL})"
            exit 1
        fi

        info "CI is ${STATUS} for ${COMMIT_SHA} (${ATTEMPT}/${MAX_ATTEMPTS})."
    else
        info "CI run is not visible yet for ${COMMIT_SHA} (${ATTEMPT}/${MAX_ATTEMPTS})."
    fi

    sleep "$WAIT_SECONDS"
done

error "Timed out waiting for CI success for ${COMMIT_SHA}."
exit 1
