#!/usr/bin/env bash
# Generate an in-process controls-off sample_diagrams candidate for downstream review.

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

RAW_OUTPUT_DIR="${OUTPUT_ROOT}/raw"
CANDIDATE_OUTPUT="${OUTPUT_ROOT}/sample-diagrams-top-candidate.png"
GEOMETRY_OUTPUT="${RAW_OUTPUT_DIR}/sample-diagrams-preview-geometry.json"
FULL_SCREENSHOT="${RAW_OUTPUT_DIR}/sample-diagrams-full.png"

if ! command -v magick >/dev/null 2>&1; then
  echo "ERROR: ImageMagick magick is required" >&2
  exit 1
fi

"${SCRIPT_DIR}/run.sh" \
  --request "${SCRIPT_DIR}/examples/sample-diagrams-canonical-capture.json" \
  --output "${RAW_OUTPUT_DIR}"

[[ -s "${FULL_SCREENSHOT}" && -s "${GEOMETRY_OUTPUT}" ]] || {
  echo "ERROR: clean capture did not produce the full screenshot and geometry record" >&2
  exit 1
}
jq -e '
  . as $capture | .interaction_state as $state |
  $capture.require_clean_interaction_state == true and
  ($capture.ui_pass_frame_nr | type) == "number" and
  $state.completed_ui_frame_nr == $capture.ui_pass_frame_nr and
  $state.markdown_section_renders > 0 and
  ($state | has("pointer_position") and has("active_editor_line")) and
  $state.pointer_inside_content == false and
  $state.active_editor_line == null and
  ([ $state.hovered_preview_line_count, $state.diagram_control_renders,
     $state.image_control_renders, $state.code_copy_control_renders,
     $state.active_markdown_ranges, $state.hovered_markdown_spans,
     $state.image_hover_background_renders, $state.local_image_hover_background_renders,
     $state.code_selection_renders ] | all(. == 0)) and
  $capture.preview_geometry.scroll_y == 0 and
  $capture.preview_geometry.physical_crop == {x:88,y:268,width:2374,height:4450}
' "${GEOMETRY_OUTPUT}" >/dev/null

# 検証済みcaptureを候補へ正規化するだけで、参照画像として採用しない。
magick "${FULL_SCREENSHOT}" \
  -crop 2374x4450+88+268 \
  +repage \
  -filter Box \
  -resize '1280x2400!' \
  -strip \
  "${CANDIDATE_OUTPUT}"

[[ "$(magick identify -format '%wx%h' "${CANDIDATE_OUTPUT}")" == "1280x2400" ]] || {
  echo "ERROR: candidate normalization did not produce 1280x2400" >&2
  exit 1
}
echo "candidate_only=true execution_mode=in_process output=${CANDIDATE_OUTPUT}"
shasum -a 256 "${CANDIDATE_OUTPUT}"
