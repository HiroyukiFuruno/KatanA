#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
SCRIPT="$ROOT_DIR/scripts/release/smoke-office-restore.sh"
MAIN="$ROOT_DIR/tmp/KatanA-egui-0362-preserved"
WORKER="$ROOT_DIR/tmp/kdv-office-worker-egui-0362-preserved"
XLSX="$ROOT_DIR/scripts/screenshot/fixtures/v0-22-38-multi-format/representative.xlsx"
source "$ROOT_DIR/scripts/release/packaged-process-identity.sh"
source "$ROOT_DIR/scripts/release/startup-process-cleanup.sh"
source "$ROOT_DIR/scripts/release/startup-heartbeat.sh"

bash -n "$SCRIPT"
if "$SCRIPT" "$MAIN" "$WORKER" "$XLSX" docx >/dev/null 2>&1; then
    echo "FAIL: unsupported expected format was accepted" >&2
    exit 1
fi
if "$SCRIPT" "$MAIN" "$WORKER" "$XLSX" pptx >/dev/null 2>&1; then
    echo "FAIL: mismatched source extension was accepted" >&2
    exit 1
fi
if "$SCRIPT" "$MAIN" "$WORKER" "$ROOT_DIR/scripts/screenshot/fixtures/v0-22-38-multi-format/missing.xlsx" xlsx >/dev/null 2>&1; then
    echo "FAIL: missing source was accepted" >&2
    exit 1
fi
for marker in KATANA_CONFIG_DIR KATANA_STARTUP_HEARTBEAT KATANA_KDV_OFFICE_WORKER \
              DEBUG=true packaged_verify_process_identity document_frame_received \
              peak_subtree_rss_kib samplings sample_id parse_office_logs parse_samples as_uri; do
    rg -q "$marker" "$SCRIPT" || { echo "FAIL: contract marker missing: $marker" >&2; exit 1; }
done

parser_dir=$(mktemp -d "$ROOT_DIR/tmp/trash/$(date +%Y-%m-%d-%H%M%S)-office-parser.XXXXXX")
parser_log="$parser_dir/events.log"
parser_state="$parser_dir/log-state.json"
parser_samples="$parser_dir/samples.tsv"
parser_summary="$parser_dir/samples-summary.json"
parser_source="/fixture/要件 定義.xlsx"
printf '%s\n' \
    'event=document_file_read path=/fixture/要件 定義.xlsx.bak' \
    'path=/fixture/要件 定義.xlsx' \
    'event=document_worker_handoff format=pptx uri=file:///fixture/wrong.xlsx' \
    'event=document_worker_handoff format=xlsx' \
    'uri=file:///fixture/%E8%A6%81%E4%BB%B6%20%E5%AE%9A%E7%BE%A9.xlsx' \
    'event=document_frame_received format=pptx elapsed_ms=9' \
    'event=document_frame_received format=xlsx elapsed_ms=42' >"$parser_log"
printf 'sample_id\telapsed_s\tframe\tpid\tpath\trss_kib\tcpu_percent\n1\t3\t7\t101\t/one\t100\t1.25\n1\t3\t7\t102\t/two\t200\t2.25\n2\t3\t8\t101\t/one\t150\t1.50\n' >"$parser_samples"
bash "$SCRIPT" --contract-parse "$parser_log" "$parser_source" xlsx "$parser_state" "$parser_samples" "$parser_summary"
python3 - "$parser_state" "$parser_summary" <<'PY'
import json
import pathlib
import sys

state = json.loads(pathlib.Path(sys.argv[1]).read_text())
summary = json.loads(pathlib.Path(sys.argv[2]).read_text())
assert state == {"source_read": False, "handoff": False, "frame_elapsed_ms": 42}, state
assert summary["peak_subtree_rss_kib"] == 300, summary
assert summary["peak_subtree_cpu_percent"] == 3.5, summary
assert len(summary["samplings"]) == 3
assert {sample["sample_id"] for sample in summary["samplings"]} == {1, 2}
PY
valid_log="$parser_dir/valid.log"
printf '%s\n' \
    'event=document_file_read path=/fixture/要件 定義.xlsx' \
    'event=document_worker_handoff format=xlsx uri=file:///fixture/%E8%A6%81%E4%BB%B6%20%E5%AE%9A%E7%BE%A9.xlsx' \
    'event=document_frame_received format=xlsx elapsed_ms=42' >"$valid_log"
bash "$SCRIPT" --contract-parse "$valid_log" "$parser_source" xlsx "$parser_state" "$parser_samples" "$parser_summary"
python3 - "$parser_state" <<'PY'
import json
import pathlib
import sys

assert json.loads(pathlib.Path(sys.argv[1]).read_text()) == {
    "source_read": True, "handoff": True, "frame_elapsed_ms": 42
}
PY
empty_log="$parser_dir/empty.log"
: >"$empty_log"
bash "$SCRIPT" --contract-parse "$empty_log" "$parser_source" xlsx "$parser_state" "$parser_samples" "$parser_summary"
python3 - "$parser_state" <<'PY'
import json
import pathlib
import sys

assert json.loads(pathlib.Path(sys.argv[1]).read_text()) == {
    "source_read": False, "handoff": False, "frame_elapsed_ms": None
}
PY

heartbeat=$(mktemp "${TMPDIR:-/tmp}/katana-heartbeat-contract.XXXXXX")
printf 'first-frame\nframe=1\n' >"$heartbeat"
if require_heartbeat_advance "$heartbeat" 1 >/dev/null 2>&1; then
    echo "FAIL: stopped heartbeat was accepted" >&2
    exit 1
fi
trash_dir="$ROOT_DIR/tmp/trash/$(date +%Y-%m-%d-%H%M%S)-office-contract"
mkdir -p "$trash_dir"
mv "$heartbeat" "$trash_dir/heartbeat.txt"

child=""
owned=""
unrelated=""
cleanup_child() {
    [[ -z "${owned:-}" ]] || kill "$owned" 2>/dev/null || true
    [[ -z "$child" ]] || kill "$child" 2>/dev/null || true
    [[ -z "$child" ]] || wait "$child" 2>/dev/null || true
    [[ -z "$unrelated" ]] || kill "$unrelated" 2>/dev/null || true
    [[ -z "$unrelated" ]] || wait "$unrelated" 2>/dev/null || true
}
trap cleanup_child EXIT
bash -c 'sleep 5 & wait' &
child=$!
sleep_path=$(packaged_canonical_path "$(command -v sleep)")
for _ in {1..20}; do
    owned=$(packaged_find_matching_pid "$child" "$sleep_path" || true)
    [[ -n "$owned" ]] && break
    sleep 0.05
done
[[ -n "$owned" ]] || { echo "FAIL: real child process was not observed" >&2; exit 1; }
launcher_is_owned_child "$child" || { echo "FAIL: real wrapper ownership was not observed" >&2; exit 1; }
packaged_verify_process_identity "$owned" "$sleep_path" "$(packaged_sha256 "$sleep_path")" || {
    echo "FAIL: real child identity verification failed" >&2
    exit 1
}
sleep_sha=$(packaged_sha256 "$sleep_path")
EXPECTED_EXECUTABLE="$sleep_path"
EXPECTED_SHA256="$sleep_sha"
APP_PID="$child"
MAIN_PID=""
terminate_verified_main
unrelated_sleep_path=$(command -v sleep)
"$unrelated_sleep_path" 5 &
unrelated=$!
kill "$child" 2>/dev/null || true
wait "$child" 2>/dev/null || true
child=""
for _ in {1..20}; do
    if ! kill -0 "$owned" 2>/dev/null; then break; fi
    sleep 0.05
done
if kill -0 "$owned" 2>/dev/null; then
    echo "FAIL: owned child survived identity cleanup" >&2
    exit 1
fi
kill -0 "$unrelated" 2>/dev/null || { echo "FAIL: unrelated process was killed" >&2; exit 1; }
kill "$unrelated" 2>/dev/null || true
wait "$unrelated" 2>/dev/null || true
unrelated=""
echo "PASS: smoke-office-restore contract tests"
