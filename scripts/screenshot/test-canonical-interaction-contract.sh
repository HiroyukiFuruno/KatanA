#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -ne 3 ]]; then
  echo "Usage: bash $0 ABSOLUTE_RUNNER ABSOLUTE_SOURCE_ROOT NEW_ABSOLUTE_OUTPUT_ROOT" >&2
  exit 2
fi
runner="$1"
source_root="$2"
output_root="$3"
for path in "$runner" "$source_root" "$output_root"; do
  case "$path" in /*) ;; *) echo "FAIL: paths must be absolute" >&2; exit 2 ;; esac
done
[[ -x "$runner" && -d "$source_root" ]] || { echo "FAIL: runner/source unavailable" >&2; exit 2; }
[[ ! -e "$output_root" ]] || { echo "FAIL: preserve existing output; choose a new directory" >&2; exit 2; }
command -v jq >/dev/null
cd "$source_root"
mkdir -p "$output_root"

# Require rejection from a rendered interaction overlay, not an unknown step or startup failure.
if "$runner" --request scripts/screenshot/examples/sample-typography-contaminated-interaction.json \
  --output "$output_root/negative" >"$output_root/negative.log" 2>&1; then
  echo "FAIL: controls-on capture was accepted" >&2
  exit 1
fi
grep -Fq 'preview geometry capture contains interaction overlay state:' "$output_root/negative.log"
grep -Eq '"code_copy_control_renders":[[:space:]]*[1-9][0-9]*' "$output_root/negative.log"

for fixture in typography diagrams; do
  "$runner" --request "scripts/screenshot/examples/sample-${fixture}-canonical-capture.json" \
    --output "$output_root/$fixture" >"$output_root/$fixture.log" 2>&1
  [[ -s "$output_root/$fixture/sample-${fixture}-full.png" ]]
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
  ' "$output_root/$fixture/sample-${fixture}-preview-geometry.json" >/dev/null
done
echo "PASS: actual controls-on rejection and both same-frame controls-off captures"
