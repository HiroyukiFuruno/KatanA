#!/usr/bin/env bash
# 提供されたmacOS bundleをfreshなTerms-free設定で起動する。
# ユーザーが通常終了するまで待機し、プロセスを強制終了しない。
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
PARSER="$ROOT_DIR/scripts/release/docker-ui-document-log-parser.py"
SAMPLER="$ROOT_DIR/scripts/release/process_memory_sampler.py"
APP_BUNDLE=${1:-}
FIXTURE_DIR=${2:-}

usage() { echo "usage: $0 /path/to/KatanA.app /path/to/fixture-directory" >&2; }
[[ -n "$APP_BUNDLE" && -n "$FIXTURE_DIR" ]] || { usage; exit 2; }
[[ "$APP_BUNDLE" == *.app && -d "$APP_BUNDLE" ]] || { echo "FAIL: .app bundle not found: $APP_BUNDLE" >&2; exit 2; }
[[ -d "$FIXTURE_DIR" ]] || { echo "FAIL: fixture directory not found: $FIXTURE_DIR" >&2; exit 2; }
[[ "$(uname -s)" == Darwin ]] || { echo "FAIL: this helper requires macOS" >&2; exit 2; }

APP_BUNDLE=$(cd "$APP_BUNDLE" && printf '%s\n' "$PWD")
MAIN="$APP_BUNDLE/Contents/MacOS/KatanA"
WORKER="$APP_BUNDLE/Contents/MacOS/kdv-office-worker"
[[ -x "$MAIN" && -x "$WORKER" ]] || { echo "FAIL: bundle is missing executable main/worker" >&2; exit 2; }
if pgrep -x KatanA >/dev/null 2>&1; then
    echo "FAIL: another KatanA main process is already running" >&2
    exit 3
fi

DOCUMENT=$(find "$FIXTURE_DIR" -maxdepth 1 -type f \( -iname '*.docx' -o -iname '*.xlsx' -o -iname '*.pptx' \
    -o -iname '*.pdf' -o -iname '*.html' -o -iname '*.htm' \) -print | sort | head -1)
[[ -n "$DOCUMENT" ]] || { echo "FAIL: fixture directory contains no regular document" >&2; exit 2; }
DOCUMENT=$(cd "$(dirname "$DOCUMENT")" && printf '%s/%s\n' "$PWD" "$(basename "$DOCUMENT")")
FIXTURE_DIR=$(cd "$FIXTURE_DIR" && printf '%s\n' "$PWD")

RUN_DIR=$(mktemp -d "$ROOT_DIR/tmp/macos-isolated-document-qa.XXXXXX")
CONFIG_DIR="$RUN_DIR/config"
RECEIPT_DIR="$RUN_DIR/resource-receipts"
mkdir -p "$CONFIG_DIR" "$RECEIPT_DIR"
HEARTBEAT="$RUN_DIR/heartbeat.txt"
LOG="$RUN_DIR/app.log"
RUN_ID=$(basename "$RUN_DIR")
printf '%s\n' "run_id=$RUN_ID" "app_bundle=$APP_BUNDLE" "fixture_dir=$FIXTURE_DIR" "document=$DOCUMENT" >"$RUN_DIR/launch.txt"

hash_files() {
    local root="$1" out="$2" file
    : >"$out"
    while IFS= read -r file; do
        shasum -a 256 "$file" | awk -v path="${file#"$root"/}" '{print $1 "  " path}' >>"$out"
    done < <(find "$root" -type f -print | sort)
}
source_snapshot() {
    local out="$1"
    {
        printf 'head='; git -C "$ROOT_DIR" rev-parse HEAD
        printf 'tree='; git -C "$ROOT_DIR" ls-tree -r HEAD | shasum -a 256 | awk '{print $1}'
        printf 'status='; git -C "$ROOT_DIR" status --porcelain=v1 | shasum -a 256 | awk '{print $1}'
        printf 'diff='; git -C "$ROOT_DIR" diff --binary | shasum -a 256 | awk '{print $1}'
        git -C "$ROOT_DIR" ls-files -- '*Cargo.lock' | while IFS= read -r lock; do
            shasum -a 256 "$ROOT_DIR/$lock"
        done
    } >"$out"
}

source_snapshot "$RUN_DIR/source.before.txt"
hash_files "$FIXTURE_DIR" "$RUN_DIR/fixtures.before.sha256"
shasum -a 256 "$MAIN" "$WORKER" >"$RUN_DIR/bundle.sha256"
codesign --verify --deep --strict "$APP_BUNDLE" >"$RUN_DIR/codesign.txt" 2>&1
printf '%s\n' 'codesign --verify --deep --strict: passed' >>"$RUN_DIR/codesign.txt"
{
    printf 'main='; lipo -archs "$MAIN"
    printf 'worker='; lipo -archs "$WORKER"
} >"$RUN_DIR/architectures.txt"
python3 "$PARSER" --write-fresh-config "$CONFIG_DIR/settings.json" "$CONFIG_DIR/workspace.json" --document "$DOCUMENT"
python3 "$PARSER" --validate-fresh-config "$CONFIG_DIR/settings.json" --document "$DOCUMENT" >"$RUN_DIR/fresh-config.validation.txt"
cp "$CONFIG_DIR/settings.json" "$RUN_DIR/fresh-config.prelaunch.json"
shasum -a 256 "$CONFIG_DIR/settings.json" "$CONFIG_DIR/workspace.json" >"$RUN_DIR/fresh-config.sha256"
: >"$HEARTBEAT"

export KATANA_CONFIG_DIR="$CONFIG_DIR"
export KATANA_STARTUP_HEARTBEAT="$HEARTBEAT"
export KATANA_DOCUMENT_RESOURCE_RECEIPT_DIR="$RECEIPT_DIR"
export KATANA_DOCUMENT_RESOURCE_RUN_ID="$RUN_ID"
export DEBUG=true
"$MAIN" >"$LOG" 2>&1 &
APP_PID=$!
printf '%s\n' "$APP_PID" >"$RUN_DIR/app.pid"
printf 'launched_pid=%s\n' "$APP_PID" | tee -a "$RUN_DIR/launch.txt"
printf 'evidence_dir=%s\n' "$RUN_DIR"

if python3 "$SAMPLER" "$APP_PID" "$RUN_DIR/memory.before.json"; then :; else echo 'memory sampler unavailable or failed' >"$RUN_DIR/memory.before.status"; fi
set +e
wait "$APP_PID"
EXIT_STATUS=$?
set -e
printf '%s\n' "$EXIT_STATUS" >"$RUN_DIR/app.exit"
ps -p "$APP_PID" -o pid=,stat=,comm= >"$RUN_DIR/process-after-wait.txt" 2>&1 ||
    printf 'pid=%s status=not-running-after-wait\n' "$APP_PID" >"$RUN_DIR/process-after-wait.txt"
hash_files "$FIXTURE_DIR" "$RUN_DIR/fixtures.after.sha256"
source_snapshot "$RUN_DIR/source.after.txt"
cmp -s "$RUN_DIR/source.before.txt" "$RUN_DIR/source.after.txt" || { echo "FAIL: source provenance changed" >&2; exit 1; }
cmp -s "$RUN_DIR/fixtures.before.sha256" "$RUN_DIR/fixtures.after.sha256" || { echo "FAIL: fixture bytes changed" >&2; exit 1; }
printf 'exit_status=%s\n' "$EXIT_STATUS" >>"$RUN_DIR/launch.txt"
echo "QA evidence retained: $RUN_DIR"
exit "$EXIT_STATUS"
