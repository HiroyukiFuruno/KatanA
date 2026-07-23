#!/bin/bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
GUARD=${ROOT_DIR}/scripts/release/check-html-browser-release-contract.sh
TMP_ROOT=$(mktemp -d)
trap 'rm -rf "$TMP_ROOT"' EXIT

write_fixture() {
    local directory=$1
    local viewer_dependency=$2
    local runtime_dependency=$3
    local viewer_lock_version=$4
    local runtime_lock_version=$5
    local include_markers=$6
    local runtime_guard_result=$7
    local lock_source=${8:-registry+https://github.com/rust-lang/crates.io-index}

    mkdir -p \
        "$directory/openspec/specs/html-file-preview" \
        "$directory/scripts/release" \
        "$directory/scripts/screenshot/examples" \
        "$directory/scripts/screenshot/fixtures/v0-22-36-html-browser" \
        "$directory/scripts/screenshot/src"
    cat >"$directory/Cargo.toml" <<EOF
[workspace]

[workspace.dependencies]
katana-document-viewer = ${viewer_dependency}
katana-render-runtime = ${runtime_dependency}
EOF
    cat >"$directory/Cargo.lock" <<EOF
version = 4

[[package]]
name = "katana-document-viewer"
version = "${viewer_lock_version}"
source = "${lock_source}"
checksum = "0000000000000000000000000000000000000000000000000000000000000000"

[[package]]
name = "katana-render-runtime"
version = "${runtime_lock_version}"
source = "${lock_source}"
checksum = "1111111111111111111111111111111111111111111111111111111111111111"
EOF
    cat >"$directory/scripts/screenshot/Cargo.toml" <<'EOF'
[workspace]
EOF
    cp "$directory/Cargo.lock" "$directory/scripts/screenshot/Cargo.lock"
    if [[ "$include_markers" == "true" ]]; then
        cat >"$directory/openspec/specs/html-file-preview/spec.md" <<'EOF'
### Requirement: Browser-equivalent HTML session is the only interactive preview path
The system MUST NOT fall back to static HTML rendering when the browser session cannot start.
### Requirement: v0.22.36 release must prove the published browser chain
The minimum resolved version of KDV `0.3.3` and KRR `0.4.6` is required.
Headless evidence must prove raw KRR frame pixels independently.
EOF
    else
        printf '%s\n' '# HTML preview without the browser contract' >"$directory/openspec/specs/html-file-preview/spec.md"
    fi
    cat >"$directory/scripts/release/check-html-browser-runtime-contract.sh" <<EOF
#!/bin/bash
exit ${runtime_guard_result}
EOF
    chmod +x "$directory/scripts/release/check-html-browser-runtime-contract.sh"
    printf '%s\n' 'mod executor_harness;' >"$directory/scripts/screenshot/src/main.rs"
    cat >"$directory/scripts/screenshot/examples/v0-22-36-html-headless-preview.json" <<'EOF'
{
  "name": "v0-22-36-html-headless-preview",
  "fixture": {
    "http_server": {
      "mount_prefix": "/app/",
      "redirects": {"/start": "/app/index.html"}
    },
    "workspace_files": [
      {"name": "index.html", "source": "index.html"},
      {"name": "linked-panel.html", "source": "linked-panel.html"},
      {"name": "style.css", "source": "style.css"},
      {"name": "actions.js", "source": "actions.js"},
      {"name": "resource-image.svg", "source": "resource-image.svg"}
    ]
  },
  "steps": [
    {"type": "action", "action": {"open_fixture_url": {"path": "/start"}}},
    {"type": "assert_active_document", "path_contains": "Katana://URL"},
    {"type": "screenshot", "output_name": "01-initial-render"},
    {"type": "assert_screenshot_contains_rgb", "screenshot": "01-initial-render", "rgb": [230, 245, 239], "min_pixels": 1},
    {"type": "assert_html_browser_origin", "origin_ends_with": "/app/index.html"},
    {"type": "assert_html_browser_viewport_matches_display_rect"},
    {"type": "assert_html_browser_display_corners_rgb", "rgb": [216, 243, 220]},
    {"type": "assert_http_requests", "paths": ["/start", "/app/index.html", "/app/style.css", "/app/actions.js", "/app/resource-image.svg"]},
    {"type": "assert_url_history", "origin_suffixes": ["/app/index.html"]},
    {"type": "assert_html_browser_frame_contains_rgb", "rgb": [38, 184, 166], "min_pixels": 1},
    {"type": "assert_html_browser_frame_contains_rgb", "rgb": [245, 158, 11], "min_pixels": 1},
    {"type": "screenshot", "output_name": "01-resource-image"},
    {"type": "assert_screenshot_contains_rgb", "screenshot": "01-resource-image", "rgb": [38, 184, 166], "min_pixels": 1},
    {"type": "screenshot", "output_name": "01-embedded-svg"},
    {"type": "assert_screenshot_contains_rgb", "screenshot": "01-embedded-svg", "rgb": [245, 158, 11], "min_pixels": 1},
    {"type": "action", "action": {"click_rgb_region": {}}},
    {"type": "screenshot", "output_name": "02-accordion-open"},
    {"type": "assert_screenshot_contains_rgb", "screenshot": "02-accordion-open", "rgb": [184, 242, 208], "min_pixels": 1},
    {"type": "assert_screenshot_changed", "min_changed_pixels": 1},
    {"type": "action", "action": {"click_rgb_region": {}}},
    {"type": "screenshot", "output_name": "03-button-action"},
    {"type": "assert_screenshot_contains_rgb", "screenshot": "03-button-action", "rgb": [255, 224, 138], "min_pixels": 1},
    {"type": "assert_screenshot_changed", "min_changed_pixels": 1},
    {"type": "action", "action": {"click_rgb_region": {}}},
    {"type": "action", "action": {"type_text": {"text": "日本語 IME入力"}}},
    {"type": "screenshot", "output_name": "04-text-input"},
    {"type": "assert_screenshot_contains_rgb", "screenshot": "04-text-input", "rgb": [167, 221, 255], "min_pixels": 1},
    {"type": "assert_screenshot_changed", "min_changed_pixels": 1},
    {"type": "action", "action": {"click_rgb_region": {}}},
    {"type": "screenshot", "output_name": "05-prevented-navigation"},
    {"type": "assert_screenshot_contains_rgb", "screenshot": "05-prevented-navigation", "rgb": [255, 209, 220], "min_pixels": 1},
    {"type": "assert_screenshot_changed", "min_changed_pixels": 1},
    {"type": "assert_active_document", "path_contains": "Katana://URL"},
    {"type": "scroll", "direction": "down", "pixels": 100, "duration_seconds": 0.1},
    {"type": "screenshot", "output_name": "05-scrolled-content"},
    {"type": "assert_screenshot_contains_rgb", "screenshot": "05-scrolled-content", "rgb": [198, 246, 213], "min_pixels": 1},
    {"type": "assert_screenshot_changed", "min_changed_pixels": 1},
    {"type": "assert_active_document", "path_contains": "Katana://URL"},
    {"type": "scroll", "direction": "up", "pixels": 100, "duration_seconds": 0.1},
    {"type": "action", "action": {"click_rgb_region": {}}},
    {"type": "screenshot", "output_name": "06-fragment-navigation"},
    {"type": "assert_html_browser_origin", "origin_ends_with": "/app/index.html#fragment-target"},
    {"type": "assert_html_browser_frame_contains_rgb", "rgb": [214, 245, 227], "min_pixels": 1},
    {"type": "assert_screenshot_contains_rgb", "screenshot": "06-fragment-navigation", "rgb": [214, 245, 227], "min_pixels": 1},
    {"type": "assert_screenshot_changed", "min_changed_pixels": 1},
    {"type": "assert_active_document", "path_contains": "Katana://URL"},
    {"type": "action", "action": {"click_rgb_region": {}}},
    {"type": "screenshot", "output_name": "07-link-navigation"},
    {"type": "assert_html_browser_origin", "origin_ends_with": "linked-panel.html#linked-target"},
    {"type": "assert_http_requests", "paths": ["/app/linked-panel.html"]},
    {"type": "assert_url_history", "origin_suffixes": ["/app/index.html", "/app/linked-panel.html#linked-target"]},
    {"type": "assert_html_browser_frame_contains_rgb", "rgb": [232, 199, 255], "min_pixels": 1},
    {"type": "assert_screenshot_contains_rgb", "screenshot": "07-link-navigation", "rgb": [232, 199, 255], "min_pixels": 1},
    {"type": "assert_screenshot_changed", "min_changed_pixels": 1},
    {"type": "assert_active_document", "path_contains": "Katana://URL"},
    {"type": "action", "action": "refresh_document"},
    {"type": "screenshot", "output_name": "08-reloaded-linked-panel"},
    {"type": "assert_html_browser_origin", "origin_ends_with": "linked-panel.html#linked-target"},
    {"type": "assert_html_browser_frame_contains_rgb", "rgb": [232, 199, 255], "min_pixels": 1},
    {"type": "assert_screenshot_contains_rgb", "screenshot": "08-reloaded-linked-panel", "rgb": [232, 199, 255], "min_pixels": 1},
    {"type": "assert_active_document", "path_contains": "Katana://URL"},
    {"type": "action", "action": {"resize_window": {}}},
    {"type": "screenshot", "output_name": "09-resized-linked-panel"},
    {"type": "assert_html_browser_origin", "origin_ends_with": "linked-panel.html#linked-target"},
    {"type": "assert_html_browser_frame_contains_rgb", "rgb": [232, 199, 255], "min_pixels": 1},
    {"type": "assert_screenshot_contains_rgb", "screenshot": "09-resized-linked-panel", "rgb": [232, 199, 255], "min_pixels": 1},
    {"type": "assert_active_document", "path_contains": "Katana://URL"}
  ]
}
EOF
    cat >"$directory/scripts/screenshot/examples/v0-22-36-light-image-controls.json" <<'EOF'
{
  "name": "v0-22-36-light-image-controls",
  "fixture": {
    "settings": {
      "theme": "light",
      "preview_show_diagram_controls": true
    },
    "workspace_files": [
      {"name": "light-image-controls.png", "source": "light-image-controls.png"}
    ]
  },
  "steps": [
    {"type": "screenshot", "output_name": "01-light-image-controls"},
    {"type": "assert_screenshot_contains_rgb", "screenshot": "01-light-image-controls", "rgb": [106, 106, 106], "min_pixels": 1000},
    {"type": "action", "action": {"click_node": {"label": "Fullscreen"}}},
    {"type": "screenshot", "output_name": "02-fullscreen-controls"},
    {"type": "action", "action": {"click_node": {"label": "Zoom In"}}},
    {"type": "screenshot", "output_name": "03-fullscreen-zoomed"},
    {"type": "scroll", "direction": "right", "pixels": 100},
    {"type": "screenshot", "output_name": "04-fullscreen-scroll-right"},
    {"type": "scroll", "direction": "down", "pixels": 100},
    {"type": "screenshot", "output_name": "05-fullscreen-scroll-down"},
    {"type": "scroll", "direction": "left", "pixels": 100},
    {"type": "screenshot", "output_name": "06-fullscreen-scroll-left"},
    {"type": "scroll", "direction": "up", "pixels": 100},
    {"type": "screenshot", "output_name": "07-fullscreen-scroll-up"},
    {"type": "assert_screenshot_changed", "baseline": "03-fullscreen-zoomed", "current": "04-fullscreen-scroll-right", "min_changed_pixels": 1},
    {"type": "assert_screenshot_changed", "baseline": "04-fullscreen-scroll-right", "current": "05-fullscreen-scroll-down", "min_changed_pixels": 1},
    {"type": "assert_screenshot_changed", "baseline": "05-fullscreen-scroll-down", "current": "06-fullscreen-scroll-left", "min_changed_pixels": 1},
    {"type": "assert_screenshot_changed", "baseline": "06-fullscreen-scroll-left", "current": "07-fullscreen-scroll-up", "min_changed_pixels": 1}
  ]
}
EOF
    cat >"$directory/scripts/screenshot/fixtures/v0-22-36-html-browser/index.html" <<'EOF'
<link rel="stylesheet" href="style.css"><main id="app-root"><img id="resource-image" src="resource-image.svg">
<svg id="embedded-mermaid-svg"></svg><details></details><a id="prevented-link"></a>
<a id="fragment-link" href="#fragment-target"></a><p id="scroll-target"></p>
<section id="fragment-target"></section>
<div id="action-panel"><button id="action"></button></div><input id="text-input">
<a href="linked-panel.html#linked-target">Open</a><script src="actions.js"></script></main>
EOF
    cat >"$directory/scripts/screenshot/fixtures/v0-22-36-html-browser/style.css" <<'EOF'
:root { --page-edge: #d8f3dc; --space: 12px; }
* { box-sizing: border-box; }
main[data-ready="true"] > .status { background: #e6f5ef !important; }
.visual-assets { display: grid; grid-template-columns: 1fr 1fr; gap: var(--space); }
@media (max-width: 900px) { .visual-assets { grid-template-columns: 1fr; } }
.status { background: #e6f5ef; }
summary { background: #4f9dff; }
input { background: #fff4cc; }
a { background: #b99aff; }
#prevented-link { background: #ffb6c1; }
#scroll-target { background: #c6f6d5; }
#fragment-link { background: #f0a35e; }
#fragment-target { background: #d6f5e3; }
EOF
    cat >"$directory/scripts/screenshot/fixtures/v0-22-36-html-browser/actions.js" <<'EOF'
document.addEventListener('DOMContentLoaded', () => {});
const status = "DOMContentLoaded executed by KRR V8";
const states = ["#b8f2d0", "#ffe08a", "#a7ddff"];
const prevented = "event.preventDefault() #ffd1dc";
const stopped = "event.stopPropagation() Parent click listener must not run";
const fragment = "Same-document fragment requested by KRR V8";
const preserved = "Input state preserved: #d6f5e3";
EOF
    printf '%s\n' '<style>#linked-target { background: #e8c7ff; }</style><section id="linked-target">Linked fragment target loaded by KRR</section>' >"$directory/scripts/screenshot/fixtures/v0-22-36-html-browser/linked-panel.html"
    printf '%s\n' '<svg xmlns="http://www.w3.org/2000/svg" aria-label="External resource pipeline"><rect fill="#26b8a6"/></svg>' >"$directory/scripts/screenshot/fixtures/v0-22-36-html-browser/resource-image.svg"
}

expect_accept() {
    local name=$1
    local directory=$2
    if ! KATANA_RELEASE_ROOT="$directory" bash "$GUARD" 0.22.36 >/dev/null; then
        printf '[ERROR] Expected fixture to pass: %s\n' "$name" >&2
        exit 1
    fi
}

expect_reject() {
    local name=$1
    local directory=$2
    local version=${3:-0.22.36}
    if KATANA_RELEASE_ROOT="$directory" bash "$GUARD" "$version" >/dev/null 2>&1; then
        printf '[ERROR] Expected fixture to fail: %s\n' "$name" >&2
        exit 1
    fi
}

pass_dir="$TMP_ROOT/pass"
write_fixture "$pass_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0
expect_accept "published browser dependency lines and patch floors" "$pass_dir"

stale_kdv_lock_dir="$TMP_ROOT/stale-kdv-lock"
write_fixture "$stale_kdv_lock_dir" '"0.3.3"' '"0.4.6"' 0.3.2 0.4.6 true 0
expect_reject "stale KDV dependency patch" "$stale_kdv_lock_dir"

stale_krr_lock_dir="$TMP_ROOT/stale-krr-lock"
write_fixture "$stale_krr_lock_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.4 true 0
expect_reject "stale KRR dependency patch" "$stale_krr_lock_dir"

stale_acceptance_lock_dir="$TMP_ROOT/stale-acceptance-lock"
write_fixture "$stale_acceptance_lock_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0
python3 - "$stale_acceptance_lock_dir/scripts/screenshot/Cargo.lock" <<'PY'
import sys
from pathlib import Path

path = Path(sys.argv[1])
path.write_text(path.read_text().replace('version = "0.3.3"', 'version = "0.3.0"', 1))
PY
expect_reject "stale headless acceptance dependency patch" "$stale_acceptance_lock_dir"

missing_checksum_dir="$TMP_ROOT/missing-checksum"
write_fixture "$missing_checksum_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0
python3 - "$missing_checksum_dir/Cargo.lock" <<'PY'
import sys
from pathlib import Path

path = Path(sys.argv[1])
lines = [line for line in path.read_text().splitlines() if not line.startswith("checksum =")]
path.write_text("\n".join(lines) + "\n")
PY
expect_reject "missing crates.io checksum" "$missing_checksum_dir"

old_dependency_dir="$TMP_ROOT/old-dependency"
write_fixture "$old_dependency_dir" '"0.2.8"' '"0.3.8"' 0.2.8 0.3.8 true 0
expect_reject "old static dependency lines" "$old_dependency_dir"

stale_manifest_dir="$TMP_ROOT/stale-manifest"
write_fixture "$stale_manifest_dir" '"0.3.0"' '"0.4.0"' 0.3.3 0.4.6 true 0
expect_reject "manifest permits stale dependency patches" "$stale_manifest_dir"

path_dependency_dir="$TMP_ROOT/path-dependency"
write_fixture "$path_dependency_dir" '{ version = "0.3.3", path = "../kdv" }' '"0.4.6"' 0.3.3 0.4.6 true 0
expect_reject "local path dependency" "$path_dependency_dir"

git_dependency_dir="$TMP_ROOT/git-dependency"
write_fixture "$git_dependency_dir" '"0.3.3"' '{ version = "0.4.6", git = "https://example.invalid/krr" }' 0.3.3 0.4.6 true 0
expect_reject "git dependency" "$git_dependency_dir"

patch_override_dir="$TMP_ROOT/patch-override"
write_fixture "$patch_override_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0
cat >>"$patch_override_dir/Cargo.toml" <<'EOF'

[patch.crates-io]
katana-document-viewer = { path = "../kdv" }
EOF
expect_reject "patch override" "$patch_override_dir"

cargo_config_override_dir="$TMP_ROOT/cargo-config-override"
write_fixture "$cargo_config_override_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0
mkdir -p "$cargo_config_override_dir/.cargo"
cat >"$cargo_config_override_dir/.cargo/config.toml" <<'EOF'
[patch.crates-io]
katana-render-runtime = { path = "../krr" }
EOF
expect_reject "cargo config patch override" "$cargo_config_override_dir"

non_registry_dir="$TMP_ROOT/non-registry"
write_fixture "$non_registry_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0 'git+https://example.invalid/runtime'
expect_reject "non-registry lock source" "$non_registry_dir"

missing_spec_dir="$TMP_ROOT/missing-spec"
write_fixture "$missing_spec_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 false 0
expect_reject "missing browser OpenSpec contract" "$missing_spec_dir"

runtime_failure_dir="$TMP_ROOT/runtime-failure"
write_fixture "$runtime_failure_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 1
expect_reject "packaged runtime contract failure" "$runtime_failure_dir"

missing_interaction_dir="$TMP_ROOT/missing-interaction"
write_fixture "$missing_interaction_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0
mv \
    "$missing_interaction_dir/scripts/screenshot/fixtures/v0-22-36-html-browser/index.html" \
    "$missing_interaction_dir/scripts/screenshot/fixtures/v0-22-36-html-browser/index.html.missing"
expect_reject "missing interactive acceptance fixture" "$missing_interaction_dir"

missing_resource_dir="$TMP_ROOT/missing-resource"
write_fixture "$missing_resource_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0
mv \
    "$missing_resource_dir/scripts/screenshot/fixtures/v0-22-36-html-browser/resource-image.svg" \
    "$missing_resource_dir/scripts/screenshot/fixtures/v0-22-36-html-browser/resource-image.svg.missing"
expect_reject "missing external image acceptance fixture" "$missing_resource_dir"

missing_semantic_dir="$TMP_ROOT/missing-semantic"
write_fixture "$missing_semantic_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0
python3 - "$missing_semantic_dir/scripts/screenshot/examples/v0-22-36-html-headless-preview.json" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
request = json.loads(path.read_text())
request["steps"] = [
    step
    for step in request["steps"]
    if not (
        step.get("type") == "assert_screenshot_contains_rgb"
        and step.get("screenshot") == "03-button-action"
    )
]
path.write_text(json.dumps(request))
PY
expect_reject "missing semantic action-state assertion" "$missing_semantic_dir"

missing_loopback_open_dir="$TMP_ROOT/missing-loopback-open-url"
write_fixture "$missing_loopback_open_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0
python3 - "$missing_loopback_open_dir/scripts/screenshot/examples/v0-22-36-html-headless-preview.json" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
request = json.loads(path.read_text())
request["steps"] = [
    step
    for step in request["steps"]
    if not (
        isinstance(step.get("action"), dict)
        and "open_fixture_url" in step["action"]
    )
]
path.write_text(json.dumps(request))
PY
expect_reject "missing loopback open_fixture_url action" "$missing_loopback_open_dir"

missing_http_requests_assert_dir="$TMP_ROOT/missing-http-requests-assert"
write_fixture "$missing_http_requests_assert_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0
python3 - "$missing_http_requests_assert_dir/scripts/screenshot/examples/v0-22-36-html-headless-preview.json" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
request = json.loads(path.read_text())
request["steps"] = [
    step for step in request["steps"] if step.get("type") != "assert_http_requests"
]
path.write_text(json.dumps(request))
PY
expect_reject "missing assert_http_requests step" "$missing_http_requests_assert_dir"

missing_stop_propagation_dir="$TMP_ROOT/missing-stop-propagation"
write_fixture "$missing_stop_propagation_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0
python3 - "$missing_stop_propagation_dir/scripts/screenshot/fixtures/v0-22-36-html-browser/actions.js" <<'PY'
import sys
from pathlib import Path

path = Path(sys.argv[1])
lines = path.read_text().splitlines(keepends=True)
path.write_text("".join(line for line in lines if "event.stopPropagation()" not in line))
PY
expect_reject "missing fixture event.stopPropagation" "$missing_stop_propagation_dir"

missing_advanced_css_dir="$TMP_ROOT/missing-advanced-css-marker"
write_fixture "$missing_advanced_css_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0
python3 - "$missing_advanced_css_dir/scripts/screenshot/fixtures/v0-22-36-html-browser/style.css" <<'PY'
import sys
from pathlib import Path

path = Path(sys.argv[1])
lines = path.read_text().splitlines(keepends=True)
path.write_text("".join(line for line in lines if "grid-template-columns" not in line))
PY
expect_reject "missing advanced CSS marker" "$missing_advanced_css_dir"

missing_lifecycle_dir="$TMP_ROOT/missing-lifecycle"
write_fixture "$missing_lifecycle_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0
python3 - "$missing_lifecycle_dir/scripts/screenshot/fixtures/v0-22-36-html-browser/actions.js" <<'PY'
import sys
from pathlib import Path

path = Path(sys.argv[1])
path.write_text(
    path.read_text().replace("document.addEventListener('DOMContentLoaded', () => {});\n", "")
)
PY
expect_reject "missing DOMContentLoaded lifecycle initialization" "$missing_lifecycle_dir"

missing_horizontal_pan_dir="$TMP_ROOT/missing-horizontal-pan"
write_fixture "$missing_horizontal_pan_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0
python3 - "$missing_horizontal_pan_dir/scripts/screenshot/examples/v0-22-36-light-image-controls.json" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
request = json.loads(path.read_text())
request["steps"] = [
    step
    for step in request["steps"]
    if step.get("direction") not in {"right", "left"}
    and "scroll-right" not in step.get("output_name", "")
    and "scroll-left" not in step.get("output_name", "")
    and "scroll-right" not in step.get("baseline", "")
    and "scroll-right" not in step.get("current", "")
    and "scroll-left" not in step.get("baseline", "")
    and "scroll-left" not in step.get("current", "")
]
path.write_text(json.dumps(request))
PY
expect_reject "missing horizontal fullscreen pan evidence" "$missing_horizontal_pan_dir"

missing_fragment_dir="$TMP_ROOT/missing-fragment"
write_fixture "$missing_fragment_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0
python3 - "$missing_fragment_dir/scripts/screenshot/examples/v0-22-36-html-headless-preview.json" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
request = json.loads(path.read_text())
request["steps"] = [
    step
    for step in request["steps"]
    if step.get("output_name") != "06-fragment-navigation"
    and step.get("screenshot") != "06-fragment-navigation"
]
path.write_text(json.dumps(request))
PY
expect_reject "missing same-document fragment evidence" "$missing_fragment_dir"

missing_origin_dir="$TMP_ROOT/missing-origin"
write_fixture "$missing_origin_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0
python3 - "$missing_origin_dir/scripts/screenshot/examples/v0-22-36-html-headless-preview.json" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
request = json.loads(path.read_text())
request["steps"] = [
    step
    for step in request["steps"]
    if step.get("type") != "assert_html_browser_origin"
]
path.write_text(json.dumps(request))
PY
expect_reject "missing complete browser origin evidence" "$missing_origin_dir"

missing_raw_frame_dir="$TMP_ROOT/missing-raw-frame"
write_fixture "$missing_raw_frame_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0
python3 - "$missing_raw_frame_dir/scripts/screenshot/examples/v0-22-36-html-headless-preview.json" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
request = json.loads(path.read_text())
request["steps"] = [
    step
    for step in request["steps"]
    if step.get("type") != "assert_html_browser_frame_contains_rgb"
]
path.write_text(json.dumps(request))
PY
expect_reject "missing raw KRR frame evidence" "$missing_raw_frame_dir"

ascii_only_input_dir="$TMP_ROOT/ascii-only-input"
write_fixture "$ascii_only_input_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0
python3 - "$ascii_only_input_dir/scripts/screenshot/examples/v0-22-36-html-headless-preview.json" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
request = json.loads(path.read_text())
for step in request["steps"]:
    action = step.get("action")
    if isinstance(action, dict) and "type_text" in action:
        action["type_text"]["text"] = "ASCII only"
path.write_text(json.dumps(request))
PY
expect_reject "missing committed non-ASCII IME input" "$ascii_only_input_dir"

fixed_click_bounds_dir="$TMP_ROOT/fixed-click-bounds"
write_fixture "$fixed_click_bounds_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0
python3 - "$fixed_click_bounds_dir/scripts/screenshot/examples/v0-22-36-html-headless-preview.json" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
request = json.loads(path.read_text())
for step in request["steps"]:
    action = step.get("action")
    if isinstance(action, dict) and "click_rgb_region" in action:
        action["click_rgb_region"]["search_bounds"] = {
            "x": 500,
            "y": 250,
            "width": 2000,
            "height": 1000,
        }
        break
path.write_text(json.dumps(request))
PY
expect_reject "fixed HTML click search bounds" "$fixed_click_bounds_dir"

native_request_name_dir="$TMP_ROOT/native-request-name"
write_fixture "$native_request_name_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0
python3 - "$native_request_name_dir/scripts/screenshot/examples/v0-22-36-html-headless-preview.json" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
request = json.loads(path.read_text())
request["name"] = "v0-22-36-html-native-window-preview"
path.write_text(json.dumps(request))
PY
expect_reject "native-window acceptance request name" "$native_request_name_dir"

native_runner_dir="$TMP_ROOT/native-runner"
write_fixture "$native_runner_dir" '"0.3.3"' '"0.4.6"' 0.3.3 0.4.6 true 0
printf '%s\n' 'mod executor_native; // --native-window' >"$native_runner_dir/scripts/screenshot/src/main.rs"
expect_reject "native-window acceptance runner" "$native_runner_dir"

expect_reject "withdrawn v0.29.0 target" "$pass_dir" 0.29.0

forbidden_runtime_source_dir="$TMP_ROOT/forbidden-runtime-source"
mkdir -p "$forbidden_runtime_source_dir"
printf '%s\n' 'const RUNTIME: &str = "Chromium";' >"$forbidden_runtime_source_dir/runtime.rs"
if KATANA_HTML_UI_SOURCE_ROOT="$forbidden_runtime_source_dir" \
    bash "$ROOT_DIR/scripts/release/check-html-browser-runtime-contract.sh" 0.22.36 \
    >/dev/null 2>&1; then
    printf '[ERROR] Runtime contract accepted an external browser source marker.\n' >&2
    exit 1
fi

forbidden_runtime_manifest="$TMP_ROOT/forbidden-runtime-Cargo.toml"
printf '%s\n' '[dependencies]' 'fantoccini = "0.22"' >"$forbidden_runtime_manifest"
if KATANA_HTML_RUNTIME_MANIFESTS="$forbidden_runtime_manifest" \
    bash "$ROOT_DIR/scripts/release/check-html-browser-runtime-contract.sh" 0.22.36 \
    >/dev/null 2>&1; then
    printf '[ERROR] Runtime contract accepted an external browser dependency.\n' >&2
    exit 1
fi

if ! bash "$ROOT_DIR/scripts/release/check-html-browser-runtime-contract.sh" 0.22.36 >/dev/null; then
    printf '[ERROR] KatanA interactive HTML runtime contract failed.\n' >&2
    exit 1
fi

if ! grep -Fq 'bash scripts/release/test-html-browser-release-contract.sh' "$ROOT_DIR/scripts/release/preflight.sh"; then
    printf '[ERROR] preflight.sh does not execute the HTML browser contract regression.\n' >&2
    exit 1
fi
if ! grep -Fq 'scripts/release/check-html-browser-release-contract.sh "$VERSION"' "$ROOT_DIR/scripts/release/preflight.sh"; then
    printf '[ERROR] preflight.sh does not execute the production HTML browser contract.\n' >&2
    exit 1
fi
if ! grep -Fq './scripts/release/preflight.sh "$TARGET_VERSION"' "$ROOT_DIR/scripts/release/check-pr-ready.sh"; then
    printf '[ERROR] check-pr-ready.sh does not execute preflight.sh.\n' >&2
    exit 1
fi

printf '[OK]    HTML browser release contract tests passed.\n'
