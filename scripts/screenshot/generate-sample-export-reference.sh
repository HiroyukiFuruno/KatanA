#!/usr/bin/env bash
# Generate an in-process sample.md export candidate for downstream review.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd -P)"

if [[ "$#" -ne 1 ]]; then
  echo "Usage: $0 NEW_ABSOLUTE_OUTPUT_ROOT" >&2
  exit 2
fi

OUTPUT_ROOT="$1"
case "${OUTPUT_ROOT}" in
  /*) ;;
  *) echo "ERROR: output root must be absolute" >&2; exit 2 ;;
esac
if [[ -e "${OUTPUT_ROOT}" ]]; then
  echo "ERROR: output root must be new: ${OUTPUT_ROOT}" >&2
  exit 2
fi

OUTPUT_PARENT="$(cd "$(dirname "${OUTPUT_ROOT}")" && pwd -P)"
OUTPUT_ROOT="${OUTPUT_PARENT}/$(basename "${OUTPUT_ROOT}")"
case "${OUTPUT_ROOT}" in
  "${REPO_ROOT}/assets/reference"|"${REPO_ROOT}/assets/reference/"*)
    echo "ERROR: candidates must not be written under assets/reference" >&2
    exit 2
    ;;
esac

CANDIDATE_OUTPUT="${OUTPUT_ROOT}/sample.png"

"${SCRIPT_DIR}/run.sh" \
  --request "${SCRIPT_DIR}/examples/sample-canonical-export-reference.json" \
  --output "${OUTPUT_ROOT}"

[[ -s "${CANDIDATE_OUTPUT}" ]] || {
  echo "ERROR: export candidate was not newly produced" >&2
  exit 1
}
echo "candidate_only=true execution_mode=in_process output=${CANDIDATE_OUTPUT}"
shasum -a 256 "${CANDIDATE_OUTPUT}"
