#!/usr/bin/env bash
# 限定承認済みの隔離profileをそのまま使い、実UI操作/通常終了は操作者へ委ねる。
set -euo pipefail
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
STAGE=${1:-}
PROFILE=${2:-}
[[ -d "$STAGE" && -d "$PROFILE" && -f "$STAGE/bundle.json" ]] || {
  echo 'usage: run-macos-document-host-qa.sh <actual-candidate-stage> <existing-isolated-qa-config-directory>' >&2
  exit 2
}
case "$STAGE:$PROFILE" in /*:/*) ;; *) echo 'absolute stage/profile required' >&2; exit 2;; esac
case "$PROFILE" in "$ROOT"/tmp/*/config) ;; *) echo 'isolated QA profile required; personal profile forbidden' >&2; exit 2;; esac
MAIN="$STAGE/unpacked/KatanA Desktop.app/Contents/MacOS/KatanA"
WORKER="$STAGE/unpacked/KatanA Desktop.app/Contents/MacOS/kdv-office-worker"
rtk proxy codesign --verify --deep --strict "$STAGE/unpacked/KatanA Desktop.app"
if rtk proxy pgrep -x KatanA >/dev/null; then echo 'another main is running' >&2; exit 3; fi
RUN=$(rtk proxy mktemp -d "$ROOT/tmp/macos-document-host.XXXXXX")
rtk proxy mkdir "$RUN/resource-receipts"
rtk proxy cp "$PROFILE/settings.json" "$RUN/settings.before.json"
export KATANA_CONFIG_DIR="$PROFILE"
export KATANA_STARTUP_HEARTBEAT="$RUN/heartbeat.txt"
export KATANA_DOCUMENT_RESOURCE_RECEIPT_DIR="$RUN/resource-receipts"
export KATANA_DOCUMENT_RESOURCE_RUN_ID="${RUN##*/}"
export DEBUG=true
unset MallocStackLogging MallocStackLoggingNoCompact
LAUNCH_ARCH=$(rtk proxy python3 -c 'import json,sys; print({"macos-arm64":"-arm64","macos-x86_64":"-x86_64"}[json.load(open(sys.argv[1]))["target"]])' "$STAGE/bundle.json")
rtk proxy python3 - "$RUN/launch.json" "$STAGE/bundle.json" "$LAUNCH_ARCH" "$MAIN" <<'PY'
import json,platform,sys
with open(sys.argv[1],"x") as output:
    json.dump({"argv":["/usr/bin/arch",sys.argv[3],sys.argv[4]],"target":json.load(open(sys.argv[2]))["target"],"observer_machine":platform.machine()},output)
PY
/usr/bin/arch "$LAUNCH_ARCH" "$MAIN" >"$RUN/app.log" 2>&1 &
QA_PID=$!
printf '%s\n' "$QA_PID" >"$RUN/app.pid"
SNAPSHOT="$ROOT/scripts/release/record-document-host-snapshot.py"
rtk proxy python3 "$SNAPSHOT" --root "$ROOT" run --run-directory "$RUN" --package-bundle-manifest "$STAGE/bundle.json" --archive "$STAGE/KatanA-macOS.zip" --profile "$PROFILE" --moment before
rtk proxy python3 "$ROOT/scripts/release/collect-document-host-memory.py" --run-directory "$RUN" --executable "$MAIN" --sidecar-executable "$WORKER" >"$RUN/observer.log" 2>&1 &
OBSERVER_PID=$!
printf 'HOST_QA_RUN=%s APP_PID=%s OBSERVER_PID=%s\n' "$RUN" "$QA_PID" "$OBSERVER_PID"
set +e
wait "$QA_PID"
QA_EXIT=$?
set -e
printf '%s\n' "$QA_EXIT" >"$RUN/app.exit"
rtk proxy cp "$PROFILE/settings.json" "$RUN/settings.after.json"
set +e
wait "$OBSERVER_PID"
OBSERVER_EXIT=$?
set -e
printf '%s\n' "$OBSERVER_EXIT" >"$RUN/observer.exit"
[[ "$QA_EXIT" == 0 ]] || exit "$QA_EXIT"
[[ "$OBSERVER_EXIT" == 0 ]] || exit "$OBSERVER_EXIT"
rtk proxy python3 "$SNAPSHOT" --root "$ROOT" run --run-directory "$RUN" --package-bundle-manifest "$STAGE/bundle.json" --archive "$STAGE/KatanA-macOS.zip" --profile "$PROFILE" --moment after
printf 'HOST_QA_NORMAL_EXIT_RECORDED=%s\n' "$RUN"
