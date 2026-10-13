#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -ne 2 ]]; then
  echo "Usage: $0 ABSOLUTE_RUNNER NEW_ABSOLUTE_OUTPUT_ROOT" >&2
  exit 2
fi
RUNNER="$1"
OUTPUT_ROOT="$2"
case "$RUNNER:$OUTPUT_ROOT" in
  /*:/*) ;;
  *) echo "FAIL: runner and output paths must be absolute" >&2; exit 2 ;;
esac
[[ -x "$RUNNER" && ! -e "$OUTPUT_ROOT" ]] || {
  echo "FAIL: runner unavailable or output already exists" >&2
  exit 2
}
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
mkdir -p "$OUTPUT_ROOT"
if "$RUNNER" --request "$SCRIPT_DIR/examples/export-without-active-document-regression.json" \
  --output "$OUTPUT_ROOT/capture" >"$OUTPUT_ROOT/runner.log" 2>&1; then
  echo "FAIL: export without an active document was accepted" >&2
  exit 1
fi
grep -Fq 'export_png requires an active document' "$OUTPUT_ROOT/runner.log"
[[ ! -e "$OUTPUT_ROOT/capture/must-not-exist.png" ]]
echo "PASS: actual in-process runner rejects export without an active document"
