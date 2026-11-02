#!/bin/bash
set -euo pipefail

COMMIT_SHA=${1:-}
REQUIRED=${2:-}
RECEIPT=${3:-}
DISPATCH_MODE=${4:-}
if [[ -n "$REQUIRED" && "$REQUIRED" != "--required" ]]; then
    echo "[ERROR] Invalid CI gate mode." >&2
    exit 2
fi
if [[ "$REQUIRED" == "--required" && -z "$RECEIPT" ]]; then
    echo "[ERROR] Required CI gate needs a receipt path." >&2
    exit 2
fi
if [[ -n "$DISPATCH_MODE" && ( "$REQUIRED" != "--required" || "$DISPATCH_MODE" != "--dispatch-if-missing" ) ]]; then
    echo "[ERROR] Invalid CI dispatch mode." >&2
    exit 2
fi
# The Windows release matrix regularly exceeds one hour. Keep the default
# longer than the full three-platform gate so a valid release cannot time out.
MAX_ATTEMPTS=${CI_GATE_WAIT_ATTEMPTS:-360}
WAIT_SECONDS=${CI_GATE_WAIT_SECONDS:-30}
VERSION_FILE_PATTERN='^(Cargo\.toml|Cargo\.lock|crates/katana-ui/Info\.plist)$'

error() { printf '[ERROR] %s\n' "$*" >&2; }
info() { printf '[INFO]  %s\n' "$*"; }
success() { printf '[OK]    %s\n' "$*"; }

if [ -z "$COMMIT_SHA" ]; then
    error "Commit SHA is required."
    exit 1
fi
if [[ ! "$COMMIT_SHA" =~ ^[0-9a-f]{40}$ ]]; then
    error "Commit SHA must be a full canonical SHA."
    exit 1
fi

if ! git rev-parse "${COMMIT_SHA}^" >/dev/null 2>&1; then
    error "Cannot inspect parent commit for ${COMMIT_SHA}."
    exit 1
fi
if [[ "$REQUIRED" == "--required" ]]; then
    MASTER_SHA=$(gh api "repos/${GITHUB_REPOSITORY:?}/git/ref/heads/master" --jq '.object.sha')
    if [[ "$MASTER_SHA" != "$COMMIT_SHA" ]]; then
        error "Release source is not the current master head."
        exit 1
    fi
fi

CHANGED_FILES=$(git diff --name-only "${COMMIT_SHA}^" "$COMMIT_SHA")
if [[ "$REQUIRED" != "--required" ]] && ! printf '%s\n' "$CHANGED_FILES" | grep -Eq "$VERSION_FILE_PATTERN"; then
    success "No version file changes detected in ${COMMIT_SHA}; CI gate wait is not required."
    exit 0
fi

info "Version file changes detected in ${COMMIT_SHA}; waiting for CI success on the same commit."
DISPATCHED=false

for ATTEMPT in $(seq 1 "$MAX_ATTEMPTS"); do
    RUN_RECORD=$(gh run list \
        --workflow test-and-build.yml \
        --commit "$COMMIT_SHA" \
        --limit 50 \
        --json status,conclusion,url,headSha,databaseId,event,createdAt \
        --jq "[.[] | select(.headSha == \"${COMMIT_SHA}\" and (.event == \"push\" or .event == \"workflow_dispatch\"))] | sort_by(.createdAt) | last // empty | [.status, (.conclusion // \"-\"), .url, .databaseId] | @tsv")

    if [ -n "$RUN_RECORD" ]; then
        IFS=$'\t' read -r STATUS CONCLUSION URL RUN_ID <<<"$RUN_RECORD"
        if [ "$STATUS" = "completed" ] && [ "$CONCLUSION" = "success" ]; then
            EXTRA=()
            if [[ "$REQUIRED" == "--required" ]]; then
                EXTRA=(--receipt "${RECEIPT}.pending")
            fi
            python3 scripts/release/verify-ci-run.py --sha "$COMMIT_SHA" --run-id "$RUN_ID" "${EXTRA[@]}"
            if [[ "$REQUIRED" == "--required" ]]; then
                LATEST_RUN=$(gh run list --workflow test-and-build.yml --commit "$COMMIT_SHA" --limit 50 \
                    --json headSha,event,createdAt,databaseId \
                    --jq "[.[] | select(.headSha == \"${COMMIT_SHA}\" and (.event == \"push\" or .event == \"workflow_dispatch\"))] | sort_by(.createdAt) | last | .databaseId")
                if [[ "$LATEST_RUN" != "$RUN_ID" ]]; then
                    error "A newer CI run appeared during verification."
                    exit 1
                fi
                MASTER_SHA=$(gh api "repos/${GITHUB_REPOSITORY:?}/git/ref/heads/master" --jq '.object.sha')
                if [[ "$MASTER_SHA" != "$COMMIT_SHA" ]]; then
                    error "Master changed during CI verification."
                    exit 1
                fi
                mv "${RECEIPT}.pending" "$RECEIPT"
            fi
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
        if [[ "$DISPATCH_MODE" == "--dispatch-if-missing" && "$DISPATCHED" == false ]]; then
            MASTER_SHA=$(gh api "repos/${GITHUB_REPOSITORY:?}/git/ref/heads/master" --jq '.object.sha')
            if [[ "$MASTER_SHA" != "$COMMIT_SHA" ]]; then
                error "Generated source is not the current master head."
                exit 1
            fi
            EXISTING=$(gh run list --workflow test-and-build.yml --commit "$COMMIT_SHA" --limit 50 \
                --json headSha,event --jq "[.[] | select(.headSha == \"${COMMIT_SHA}\" and (.event == \"push\" or .event == \"workflow_dispatch\"))] | length")
            if [[ "$EXISTING" != 0 ]]; then
                continue
            fi
            gh workflow run test-and-build.yml --ref master
            DISPATCHED=true
            info "Dispatched one full master CI run for ${COMMIT_SHA}."
        fi
    fi

    sleep "$WAIT_SECONDS"
done

error "Timed out waiting for CI success for ${COMMIT_SHA}."
exit 1
