#!/usr/bin/env bash
set -euo pipefail

MAIN_EXECUTABLE="${1:-}"
WORKER_EXECUTABLE="${2:-}"
SOURCE_FILE="${3:-}"
EXPECTED_FORMAT="${4:-}"

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)

parse_office_logs() {
    local log_path="$1" source_path="$2" expected_format="$3" output="$4"
    python3 - "$log_path" "$source_path" "$expected_format" "$output" <<'PY'
import json
import pathlib
import re
import sys

log_path = pathlib.Path(sys.argv[1])
source_path = pathlib.Path(sys.argv[2])
expected_format = sys.argv[3]
output = pathlib.Path(sys.argv[4])
lines = log_path.read_text(errors="replace").splitlines() if log_path.is_file() else []
source_token = str(source_path.resolve())
uri_token = source_path.resolve().as_uri()
def field_equals(line, name, value):
    return re.search(r"(?:^|\s)" + re.escape(name) + r"=" + re.escape(value) + r"(?:\s|$)", line) is not None
source_seen = any(field_equals(line, "event", "document_file_read") and
                  field_equals(line, "path", source_token) for line in lines)
handoff_seen = any(field_equals(line, "event", "document_worker_handoff") and
                   field_equals(line, "format", expected_format) and
                   field_equals(line, "uri", uri_token) for line in lines)
frame_elapsed = None
for line in lines:
    if field_equals(line, "event", "document_frame_received") and field_equals(line, "format", expected_format):
        match = re.search(r"elapsed_ms=(\d+)", line)
        if match:
            frame_elapsed = int(match.group(1))
            break
output.write_text(json.dumps({"source_read": source_seen, "handoff": handoff_seen,
                              "frame_elapsed_ms": frame_elapsed}, indent=2) + "\n")
PY
}

parse_samples() {
    local samples_path="$1" output="$2"
    python3 - "$samples_path" "$output" <<'PY'
import json
import pathlib
import sys

samples_path, output = map(pathlib.Path, sys.argv[1:])
samples = []
poll_totals = {}
for line in samples_path.read_text().splitlines()[1:]:
    sample_id, elapsed, frame, pid, path, rss, cpu = line.split("\t", 6)
    sample = {"sample_id": int(sample_id), "elapsed_s": int(elapsed),
              "frame": int(frame) if frame.isdigit() else None, "pid": int(pid),
              "path": path, "rss_kib": int(rss or 0), "cpu_percent": float(cpu or 0)}
    samples.append(sample)
    total = poll_totals.setdefault(int(sample_id), {"rss_kib": 0, "cpu_percent": 0.0})
    total["rss_kib"] += sample["rss_kib"]
    total["cpu_percent"] += sample["cpu_percent"]
output.write_text(json.dumps({
    "samplings": samples,
    "peak_subtree_rss_kib": max((value["rss_kib"] for value in poll_totals.values()), default=0),
    "peak_subtree_cpu_percent": max((value["cpu_percent"] for value in poll_totals.values()), default=0),
}, indent=2) + "\n")
PY
}

if [[ "${1:-}" == "--contract-parse" ]]; then
    parse_office_logs "$2" "$3" "$4" "$5"
    parse_samples "$6" "$7"
    exit 0
fi

IDENTITY_HELPER="${ROOT_DIR}/scripts/release/packaged-process-identity.sh"
[[ -r "$IDENTITY_HELPER" ]] || { echo "FAIL: identity helper is missing" >&2; exit 1; }
source "$IDENTITY_HELPER"
source "${ROOT_DIR}/scripts/release/startup-process-cleanup.sh"

process_tree_pids() {
    local parent="$1" child
    echo "$parent"
    for child in $(pgrep -P "$parent" 2>/dev/null || true); do
        process_tree_pids "$child"
    done
}

case "$EXPECTED_FORMAT" in
    xlsx|pptx) ;;
    *) echo "FAIL: expected format must be xlsx or pptx: $EXPECTED_FORMAT" >&2; exit 2 ;;
esac
[[ -x "$MAIN_EXECUTABLE" ]] || { echo "FAIL: main executable is not executable: $MAIN_EXECUTABLE" >&2; exit 1; }
[[ -x "$WORKER_EXECUTABLE" ]] || { echo "FAIL: worker executable is not executable: $WORKER_EXECUTABLE" >&2; exit 1; }
[[ -f "$SOURCE_FILE" && -r "$SOURCE_FILE" ]] || { echo "FAIL: source file is not readable: $SOURCE_FILE" >&2; exit 1; }
case "${SOURCE_FILE##*.}" in
    "$EXPECTED_FORMAT") ;;
    *) echo "FAIL: source extension does not match expected format: $SOURCE_FILE" >&2; exit 2 ;;
esac

RUN_DIR=$(mktemp -d "${ROOT_DIR}/tmp/smoke-office-restore.XXXXXX")
CONFIG_DIR="${RUN_DIR}/config"
EVIDENCE_DIR="${RUN_DIR}/evidence"
HEARTBEAT="${RUN_DIR}/heartbeat.txt"
LOG_PATH="${RUN_DIR}/startup.log"
SAMPLES_PATH="${RUN_DIR}/samples.tsv"
LOG_STATE_PATH="${RUN_DIR}/log-state.json"
mkdir -p "$CONFIG_DIR" "$EVIDENCE_DIR"
APP_PID=""
MAIN_PID=""
WORKER_PID=""

EXPECTED_EXECUTABLE=$(packaged_canonical_path "$MAIN_EXECUTABLE")
EXPECTED_SHA256=$(packaged_sha256 "$EXPECTED_EXECUTABLE")
EXPECTED_WORKER=$(packaged_canonical_path "$WORKER_EXECUTABLE")
EXPECTED_WORKER_SHA256=$(packaged_sha256 "$EXPECTED_WORKER")
SOURCE_CANONICAL=$(packaged_canonical_path "$SOURCE_FILE")
SOURCE_SHA256=$(packaged_sha256 "$SOURCE_CANONICAL")
printf 'sample_id\telapsed_s\tframe\tpid\tpath\trss_kib\tcpu_percent\n' >"$SAMPLES_PATH"

python3 - "$CONFIG_DIR/settings.json" "$CONFIG_DIR/workspace.json" "$SOURCE_CANONICAL" <<'PY'
import json
import pathlib
import sys

settings_path, workspace_path, source = map(pathlib.Path, sys.argv[1:])
workspace = source.parent
settings = {
    "version": "0.2.3",
    "theme": {"theme": "dark", "icon_pack": "katana", "ui_contrast_offset": 0.0,
              "preset": "KatanaDark", "custom_color_overrides": None, "custom_themes": [],
              "active_custom_theme": None},
    "font": {"size": 14.0, "family": "monospace"},
    "workspace": {"last_workspace": str(workspace), "open_tabs": [str(source)],
                   "active_tab_idx": 0, "restore_session": True,
                   "open_workspace_in_tabs": True},
}
workspace_state = {"persisted": [str(workspace)], "histories": [str(workspace)],
                   "open_workspace_tabs": [str(workspace)], "active_workspace": str(workspace)}
settings_path.write_text(json.dumps(settings, indent=2) + "\n")
workspace_path.write_text(json.dumps(workspace_state, indent=2) + "\n")
PY

source "${ROOT_DIR}/scripts/release/startup-heartbeat.sh"
cleanup() {
    local status=$? candidate cleanup_failed=0
    trap - EXIT
    if [[ -z "$WORKER_PID" && -n "$APP_PID" ]]; then
        WORKER_PID=$(packaged_find_matching_pid "$APP_PID" "$EXPECTED_WORKER" || true)
    fi
    if [[ -n "$WORKER_PID" && -n "$APP_PID" ]] \
        && [[ "$(packaged_find_matching_pid "$APP_PID" "$EXPECTED_WORKER" || true)" == "$WORKER_PID" ]] \
        && packaged_verify_process_identity "$WORKER_PID" "$EXPECTED_WORKER" "$EXPECTED_WORKER_SHA256"; then
        kill "$WORKER_PID" 2>/dev/null || true
    fi
    terminate_verified_main
    if [[ -n "$APP_PID" ]] && launcher_is_owned_child "$APP_PID"; then
        kill "$APP_PID" 2>/dev/null || true
    fi
    for candidate in "$WORKER_PID" "$MAIN_PID" "$APP_PID"; do
        [[ -z "$candidate" ]] || wait "$candidate" 2>/dev/null || true
    done
    for _ in {1..20}; do
        local alive=0
        for candidate in "$WORKER_PID" "$MAIN_PID" "$APP_PID"; do
            if [[ -n "$candidate" ]] && kill -0 "$candidate" 2>/dev/null; then alive=1; fi
        done
        if (( alive == 0 )); then break; fi
        sleep 0.1
    done
    for candidate in "$WORKER_PID" "$MAIN_PID" "$APP_PID"; do
        if [[ -n "$candidate" ]] && kill -0 "$candidate" 2>/dev/null; then cleanup_failed=1; fi
    done
    [[ -f "$HEARTBEAT" ]] && cp "$HEARTBEAT" "$EVIDENCE_DIR/heartbeat.txt"
    [[ -f "$LOG_PATH" ]] && cp "$LOG_PATH" "$EVIDENCE_DIR/startup.log"
    cp "$CONFIG_DIR/settings.json" "$EVIDENCE_DIR/settings.json" 2>/dev/null || true
    cp "$CONFIG_DIR/workspace.json" "$EVIDENCE_DIR/workspace.json" 2>/dev/null || true
    cp "$SAMPLES_PATH" "$EVIDENCE_DIR/samples.tsv" 2>/dev/null || true
    if (( cleanup_failed )); then
        echo "FAIL: owned process remained after cleanup; evidence=$RUN_DIR" >&2
        status=1
    fi
    echo "Office restore evidence retained: $RUN_DIR" >&2
    exit "$status"
}
trap cleanup EXIT

KATANA_CONFIG_DIR="$CONFIG_DIR" \
KATANA_STARTUP_HEARTBEAT="$HEARTBEAT" \
KATANA_KDV_OFFICE_WORKER="$EXPECTED_WORKER" \
DEBUG=true \
    "$EXPECTED_EXECUTABLE" >"$LOG_PATH" 2>&1 &
APP_PID=$!

deadline=$((SECONDS + 30))
first_frame=""
last_frame=""
frame_advances=0
post_document_advances=0
document_frame=""
source_read_seen=false
handoff_seen=false
sample_id=0
while (( SECONDS < deadline )); do
    if [[ -z "$MAIN_PID" ]]; then
        MAIN_PID=$(packaged_find_matching_pid "$APP_PID" "$EXPECTED_EXECUTABLE" || true)
    fi
    if [[ -z "$MAIN_PID" && ! -d "/proc/$APP_PID" ]] && ! kill -0 "$APP_PID" 2>/dev/null; then
        echo "FAIL: main exited before document frame; evidence=$RUN_DIR" >&2
        exit 1
    fi
    if [[ -z "$first_frame" ]] && grep -qxF "first-frame" "$HEARTBEAT" 2>/dev/null; then
        first_frame=$(heartbeat_frame "$HEARTBEAT" || true)
        last_frame="$first_frame"
    elif [[ -n "$first_frame" ]]; then
        current_frame=$(require_heartbeat_advance "$HEARTBEAT" "$last_frame" || true)
        [[ "$current_frame" =~ ^[0-9]+$ ]] || { echo "FAIL: heartbeat stalled; evidence=$RUN_DIR" >&2; exit 1; }
        frame_advances=$((frame_advances + 1))
        last_frame="$current_frame"
        [[ -n "$document_frame" ]] && post_document_advances=$((post_document_advances + 1))
    fi
    if [[ -z "$WORKER_PID" && -n "$MAIN_PID" ]]; then
        WORKER_PID=$(packaged_find_matching_pid "$MAIN_PID" "$EXPECTED_WORKER" || true)
    fi
    parse_office_logs "$LOG_PATH" "$SOURCE_CANONICAL" "$EXPECTED_FORMAT" "$LOG_STATE_PATH"
    if grep -q '"source_read": true' "$LOG_STATE_PATH"; then source_read_seen=true; fi
    if grep -q '"handoff": true' "$LOG_STATE_PATH"; then handoff_seen=true; fi
    if [[ -z "$document_frame" ]]; then
        document_frame=$(sed -n 's/.*"frame_elapsed_ms": \([0-9][0-9]*\).*/\1/p' "$LOG_STATE_PATH")
    fi
    sample_frame=$(heartbeat_frame "$HEARTBEAT" 2>/dev/null || true)
    sample_id=$((sample_id + 1))
    if [[ -n "$APP_PID" ]]; then
        process_tree_pids "$APP_PID" | while IFS= read -r pid; do
            image=$(packaged_process_image "$pid" || true)
            rss=$(ps -o rss= -p "$pid" 2>/dev/null | tr -d '[:space:]' || true)
            cpu=$(ps -o %cpu= -p "$pid" 2>/dev/null | tr -d '[:space:]' || true)
            printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$sample_id" "$SECONDS" "$sample_frame" "$pid" "$image" "${rss:-0}" "${cpu:-0}" >>"$SAMPLES_PATH"
        done
    fi
    if [[ -n "$first_frame" && "$post_document_advances" -ge 2 && -n "$document_frame" \
        && "$source_read_seen" == true && "$handoff_seen" == true && -n "$WORKER_PID" ]]; then
        break
    fi
    sleep 0.2
done

[[ -n "$MAIN_PID" ]] || { echo "FAIL: launched main identity was not observed; evidence=$RUN_DIR" >&2; exit 1; }
packaged_verify_process_identity "$MAIN_PID" "$EXPECTED_EXECUTABLE" "$EXPECTED_SHA256" || {
    echo "FAIL: main identity mismatch; evidence=$RUN_DIR" >&2; exit 1;
}
[[ -n "$WORKER_PID" ]] || { echo "FAIL: office worker identity was not observed; evidence=$RUN_DIR" >&2; exit 1; }
packaged_verify_process_identity "$WORKER_PID" "$EXPECTED_WORKER" "$EXPECTED_WORKER_SHA256" || {
    echo "FAIL: worker identity mismatch; evidence=$RUN_DIR" >&2; exit 1;
}
[[ -n "$first_frame" && "$post_document_advances" -ge 2 && -n "$document_frame" \
    && "$source_read_seen" == true && "$handoff_seen" == true ]] || {
    echo "FAIL: office restore did not prove bounded frame progress and document frame; evidence=$RUN_DIR" >&2
    exit 1
}

parse_samples "$SAMPLES_PATH" "$EVIDENCE_DIR/samples-summary.json"
python3 - "$EVIDENCE_DIR/office-restore.json" "$SAMPLES_PATH" "$EXPECTED_FORMAT" "$SOURCE_CANONICAL" "$SOURCE_SHA256" "$EXPECTED_EXECUTABLE" "$EXPECTED_SHA256" "$MAIN_PID" "$EXPECTED_WORKER" "$EXPECTED_WORKER_SHA256" "$WORKER_PID" "$first_frame" "$last_frame" "$document_frame" "$frame_advances" <<'PY'
import json
import pathlib
import sys

(_, output, samples_path, fmt, source, source_sha, main, main_sha, main_pid, worker,
 worker_sha, worker_pid, first, last, document_elapsed, advances) = sys.argv
samples = []
poll_totals = {}
for line in pathlib.Path(samples_path).read_text().splitlines()[1:]:
    sample_id, elapsed, frame, pid, path, rss, cpu = line.split("\t", 6)
    sample = {"sample_id": int(sample_id), "elapsed_s": int(elapsed),
                    "frame": int(frame) if frame.isdigit() else None,
                    "pid": int(pid), "path": path, "rss_kib": int(rss or 0),
                    "cpu_percent": float(cpu or 0)}
    samples.append(sample)
    totals = poll_totals.setdefault(int(sample_id), {"rss_kib": 0, "cpu_percent": 0.0})
    totals["rss_kib"] += sample["rss_kib"]
    totals["cpu_percent"] += sample["cpu_percent"]
payload = {
    "mode": "office_restore_native_diagnostic",
    "expected_format": fmt,
    "source_path": source,
    "source_sha256": source_sha,
    "main_path": main,
    "main_sha256": main_sha,
    "main_pid": int(main_pid),
    "worker_path": worker,
    "worker_sha256": worker_sha,
    "worker_pid": int(worker_pid),
    "first_frame": int(first),
    "last_frame": int(last),
    "heartbeat_advances": int(advances),
    "document_frame_elapsed_ms": int(document_elapsed),
    "samplings": samples,
    "peak_subtree_rss_kib": max((total["rss_kib"] for total in poll_totals.values()), default=0),
    "peak_subtree_cpu_percent": max((total["cpu_percent"] for total in poll_totals.values()), default=0),
    "clean_machine": False,
    "normal_close": False,
    "acceptance_scope": "native diagnostic only; no UI quality or distribution acceptance",
}
pathlib.Path(output).write_text(json.dumps(payload, indent=2) + "\n")
PY
echo "PASS: native Office restore diagnostic format=$EXPECTED_FORMAT source=$SOURCE_CANONICAL main_pid=$MAIN_PID worker_pid=$WORKER_PID first_frame=$first_frame document_frame_elapsed_ms=$document_frame frame_advances=$frame_advances evidence=$RUN_DIR"
