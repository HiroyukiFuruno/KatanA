#!/usr/bin/env bash
# Contract-test generator argument guards without claiming an in-process candidate is acceptance.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
DIAGRAM_GENERATOR="${ROOT_DIR}/scripts/screenshot/generate-sample-diagrams-reference.sh"
EXPORT_GENERATOR="${ROOT_DIR}/scripts/screenshot/generate-sample-export-reference.sh"
EXPORT_REQUEST="${ROOT_DIR}/scripts/screenshot/examples/sample-canonical-export-reference.json"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/katana-candidate-generator.XXXXXX")"

cleanup() {
  rmdir "${TMP_ROOT}/existing" "${TMP_ROOT}" 2>/dev/null || true
}
trap cleanup EXIT

expect_reject() {
  local generator="$1"
  local label="$2"
  shift 2
  if bash "${generator}" "$@" >/dev/null 2>&1; then
    echo "FAIL: ${label} was accepted by $(basename "${generator}")" >&2
    exit 1
  fi
}

mkdir "${TMP_ROOT}/existing"
for generator in "${DIAGRAM_GENERATOR}" "${EXPORT_GENERATOR}"; do
  expect_reject "${generator}" "missing output root"
  expect_reject "${generator}" "relative output root" candidate-output
  expect_reject "${generator}" "existing output root" "${TMP_ROOT}/existing"
  expect_reject "${generator}" "reference output root" "${ROOT_DIR}/assets/reference/candidate-contract-reject"
  grep -Fq 'candidate_only=true execution_mode=in_process' "${generator}"
done

grep -Fq 'sample-diagrams-canonical-capture.json' "${DIAGRAM_GENERATOR}"
if grep -Fq 'sample-diagrams-canonical-reference.json' "${DIAGRAM_GENERATOR}"; then
  echo "FAIL: diagram generator uses an unchecked reference request" >&2
  exit 1
fi
grep -Fq 'require_clean_interaction_state == true' "${DIAGRAM_GENERATOR}"
grep -Fq 'physical_crop == {x:88,y:268,width:2374,height:4450}' "${DIAGRAM_GENERATOR}"

jq -e '
  any(.steps[]; .type == "assert_active_document" and .path_contains == "sample.md")
' "${EXPORT_REQUEST}" >/dev/null

echo "PASS: candidate generators reject unsafe output roots; no candidate capture was run"
