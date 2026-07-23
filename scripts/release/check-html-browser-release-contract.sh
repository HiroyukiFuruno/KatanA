#!/bin/bash
set -euo pipefail

error() { printf '[ERROR] %s\n' "$*" >&2; }
success() { printf '[OK]    %s\n' "$*"; }

TARGET_VERSION=${1:-}
if [[ -z "$TARGET_VERSION" ]]; then
    error "Target version is required."
    exit 1
fi
TARGET_VERSION="${TARGET_VERSION#v}"

if [[ "$TARGET_VERSION" != "0.22.36" ]]; then
    error "The browser-equivalent HTML release contract applies only to v0.22.36; received v${TARGET_VERSION}."
    exit 1
fi

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=${KATANA_RELEASE_ROOT:-$(cd "${SCRIPT_DIR}/../.." && pwd)}
CARGO_TOML=${KATANA_CARGO_TOML:-${ROOT_DIR}/Cargo.toml}
CARGO_LOCK=${KATANA_CARGO_LOCK:-${ROOT_DIR}/Cargo.lock}
CARGO_CONFIG=${KATANA_CARGO_CONFIG:-${ROOT_DIR}/.cargo/config.toml}
HTML_SPEC=${KATANA_HTML_SPEC:-${ROOT_DIR}/openspec/specs/html-file-preview/spec.md}
RUNTIME_GUARD=${KATANA_HTML_RUNTIME_CONTRACT_GUARD:-${ROOT_DIR}/scripts/release/check-html-browser-runtime-contract.sh}
ACCEPTANCE_REQUEST=${KATANA_HTML_ACCEPTANCE_REQUEST:-${ROOT_DIR}/scripts/screenshot/examples/v0-22-36-html-headless-preview.json}
IMAGE_CONTROLS_REQUEST=${KATANA_IMAGE_CONTROLS_REQUEST:-${ROOT_DIR}/scripts/screenshot/examples/v0-22-36-light-image-controls.json}
ACCEPTANCE_INDEX=${KATANA_HTML_ACCEPTANCE_INDEX:-${ROOT_DIR}/scripts/screenshot/fixtures/v0-22-36-html-browser/index.html}
ACCEPTANCE_LINKED=${KATANA_HTML_ACCEPTANCE_LINKED:-${ROOT_DIR}/scripts/screenshot/fixtures/v0-22-36-html-browser/linked-panel.html}
ACCEPTANCE_STYLE=${KATANA_HTML_ACCEPTANCE_STYLE:-${ROOT_DIR}/scripts/screenshot/fixtures/v0-22-36-html-browser/style.css}
ACCEPTANCE_SCRIPT=${KATANA_HTML_ACCEPTANCE_SCRIPT:-${ROOT_DIR}/scripts/screenshot/fixtures/v0-22-36-html-browser/actions.js}
ACCEPTANCE_IMAGE=${KATANA_HTML_ACCEPTANCE_IMAGE:-${ROOT_DIR}/scripts/screenshot/fixtures/v0-22-36-html-browser/resource-image.svg}
ACCEPTANCE_RUNNER=${KATANA_HTML_ACCEPTANCE_RUNNER:-${ROOT_DIR}/scripts/screenshot/src/main.rs}
ACCEPTANCE_CARGO_TOML=${KATANA_HTML_ACCEPTANCE_CARGO_TOML:-${ROOT_DIR}/scripts/screenshot/Cargo.toml}
ACCEPTANCE_CARGO_LOCK=${KATANA_HTML_ACCEPTANCE_CARGO_LOCK:-${ROOT_DIR}/scripts/screenshot/Cargo.lock}

for required_file in "$CARGO_TOML" "$CARGO_LOCK" "$HTML_SPEC" "$ACCEPTANCE_REQUEST" "$IMAGE_CONTROLS_REQUEST" "$ACCEPTANCE_INDEX" "$ACCEPTANCE_LINKED" "$ACCEPTANCE_STYLE" "$ACCEPTANCE_SCRIPT" "$ACCEPTANCE_IMAGE" "$ACCEPTANCE_RUNNER" "$ACCEPTANCE_CARGO_TOML" "$ACCEPTANCE_CARGO_LOCK"; do
    if [[ ! -r "$required_file" ]]; then
        error "Required HTML browser release contract file is missing: ${required_file}"
        exit 1
    fi
done

if [[ -z "${KATANA_RELEASE_ROOT:-}" ]] &&
    git -C "$ROOT_DIR" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    acceptance_lock_relative=${ACCEPTANCE_CARGO_LOCK#"${ROOT_DIR}/"}
    if ! git -C "$ROOT_DIR" ls-files --error-unmatch "$acceptance_lock_relative" >/dev/null 2>&1; then
        error "Headless acceptance Cargo.lock must be tracked: ${acceptance_lock_relative}"
        exit 1
    fi
fi

if grep -Eq 'executor_native|native_window|--native-window' "$ACCEPTANCE_RUNNER"; then
    error "v0.22.36 acceptance runner must remain headless-only."
    exit 1
fi

if ! python3 - "$ACCEPTANCE_REQUEST" "$ACCEPTANCE_INDEX" "$ACCEPTANCE_LINKED" "$ACCEPTANCE_STYLE" "$ACCEPTANCE_SCRIPT" "$ACCEPTANCE_IMAGE" <<'PY'
import json
import sys
from pathlib import Path

request_path, index_path, linked_path, style_path, script_path, image_path = map(Path, sys.argv[1:])
request = json.loads(request_path.read_text())
steps = request.get("steps", [])
if request.get("name") != "v0-22-36-html-headless-preview":
    raise SystemExit("acceptance request must be the v0.22.36 headless preview")
workspace_files = {
    entry.get("name"): entry for entry in request.get("fixture", {}).get("workspace_files", [])
}

for name in ("index.html", "linked-panel.html", "style.css", "actions.js", "resource-image.svg"):
    if name not in workspace_files:
        raise SystemExit(f"acceptance fixture is missing workspace file {name}")

http_server = request.get("fixture", {}).get("http_server", {})
if http_server.get("mount_prefix") != "/app/":
    raise SystemExit("acceptance fixture must mount its workspace at /app/")
if http_server.get("redirects", {}).get("/start") != "/app/index.html":
    raise SystemExit("acceptance fixture must redirect /start to /app/index.html")

actions = [step.get("action") for step in steps if step.get("type") == "action"]
structured_actions = {
    next(iter(action)): action[next(iter(action))]
    for action in actions
    if isinstance(action, dict) and len(action) == 1
}
unit_actions = {action for action in actions if isinstance(action, str)}

rgb_clicks = [
    action["click_rgb_region"]
    for action in actions
    if isinstance(action, dict) and "click_rgb_region" in action
]
if len(rgb_clicks) < 6:
    raise SystemExit("acceptance scenario must click accordion, button, input, prevented link, fragment link, and navigation link")
if any(click.get("search_bounds") is not None for click in rgb_clicks):
    raise SystemExit("HTML acceptance clicks must derive bounds from the rendered browser surface")
for required_action in ("type_text", "resize_window"):
    if required_action not in structured_actions:
        raise SystemExit(f"acceptance scenario is missing {required_action}")
open_url = structured_actions.get("open_fixture_url")
if not isinstance(open_url, dict) or open_url.get("path") != "/start":
    raise SystemExit("acceptance scenario must open the redirecting loopback URL")
typed_text = structured_actions["type_text"].get("text")
if not isinstance(typed_text, str) or not any(ord(character) > 127 for character in typed_text):
    raise SystemExit("acceptance scenario must exercise committed non-ASCII IME text")
if "refresh_document" not in unit_actions:
    raise SystemExit("acceptance scenario is missing refresh_document")
scroll_directions = {
    step.get("direction")
    for step in steps
    if step.get("type") == "scroll" and step.get("pixels", 0) > 0
}
if not {"down", "up"}.issubset(scroll_directions):
    raise SystemExit("acceptance scenario must scroll down and back up through the KRR viewport")

step_types = {step.get("type") for step in steps}
for required_type in (
    "assert_html_browser_viewport_matches_display_rect",
    "assert_html_browser_display_corners_rgb",
    "assert_http_requests",
    "assert_url_history",
):
    if required_type not in step_types:
        raise SystemExit(f"acceptance scenario is missing {required_type}")
corner_assertion = next(
    step for step in steps if step.get("type") == "assert_html_browser_display_corners_rgb"
)
if corner_assertion.get("rgb") != [216, 243, 220]:
    raise SystemExit("acceptance scenario must prove the page-owned color at every HTML viewport corner")
requested_paths = {
    path
    for step in steps
    if step.get("type") == "assert_http_requests"
    for path in step.get("paths", [])
}
required_paths = {
    "/start",
    "/app/index.html",
    "/app/style.css",
    "/app/actions.js",
    "/app/resource-image.svg",
    "/app/linked-panel.html",
}
if not required_paths.issubset(requested_paths):
    raise SystemExit(f"acceptance scenario is missing HTTP request assertions: {sorted(required_paths - requested_paths)}")
history_suffixes = {
    suffix
    for step in steps
    if step.get("type") == "assert_url_history"
    for suffix in step.get("origin_suffixes", [])
}
if not {"/app/index.html", "/app/linked-panel.html#linked-target"}.issubset(history_suffixes):
    raise SystemExit("acceptance scenario must prove both URL documents in tab history")

raw_frame_markers = {
    tuple(step.get("rgb", []))
    for step in steps
    if step.get("type") == "assert_html_browser_frame_contains_rgb"
    and step.get("min_pixels", 0) > 0
}
for expected_rgb in ((38, 184, 166), (245, 158, 11)):
    if expected_rgb not in raw_frame_markers:
        raise SystemExit(
            f"acceptance scenario must prove raw KRR pixels for visual resource {expected_rgb}"
        )

screenshots = {
    step.get("output_name")
    for step in steps
    if step.get("type") == "screenshot"
}
required_screenshots = {
    "01-initial-render",
    "01-resource-image",
    "01-embedded-svg",
    "02-accordion-open",
    "03-button-action",
    "04-text-input",
    "05-prevented-navigation",
    "05-scrolled-content",
    "06-fragment-navigation",
    "07-link-navigation",
    "08-reloaded-linked-panel",
    "09-resized-linked-panel",
}
missing_screenshots = sorted(required_screenshots - screenshots)
if missing_screenshots:
    raise SystemExit(f"acceptance scenario is missing screenshots: {missing_screenshots}")

change_assertions = [
    step for step in steps if step.get("type") == "assert_screenshot_changed"
]
if len(change_assertions) < 7 or any(
    assertion.get("min_changed_pixels", 0) <= 0 for assertion in change_assertions
):
    raise SystemExit("acceptance scenario must enforce seven positive screenshot-change assertions")

expected_rgb_markers = {
    "01-initial-render": [230, 245, 239],
    "01-resource-image": [38, 184, 166],
    "01-embedded-svg": [245, 158, 11],
    "02-accordion-open": [184, 242, 208],
    "03-button-action": [255, 224, 138],
    "04-text-input": [167, 221, 255],
    "05-prevented-navigation": [255, 209, 220],
    "05-scrolled-content": [198, 246, 213],
    "06-fragment-navigation": [214, 245, 227],
    "07-link-navigation": [232, 199, 255],
    "08-reloaded-linked-panel": [232, 199, 255],
    "09-resized-linked-panel": [232, 199, 255],
}
rgb_assertions = {
    step.get("screenshot"): step
    for step in steps
    if step.get("type") == "assert_screenshot_contains_rgb"
}
for screenshot, expected_rgb in expected_rgb_markers.items():
    assertion = rgb_assertions.get(screenshot)
    if assertion is None:
        raise SystemExit(f"acceptance scenario is missing semantic RGB assertion for {screenshot}")
    if assertion.get("rgb") != expected_rgb or assertion.get("min_pixels", 0) <= 0:
        raise SystemExit(f"acceptance scenario has an invalid semantic RGB assertion for {screenshot}")
screenshot_positions = {
    step.get("output_name"): position
    for position, step in enumerate(steps)
    if step.get("type") == "screenshot"
}


def require_browser_state(
    screenshot: str, expected_origin_suffix: str, expected_rgb: list[int]
) -> None:
    position = screenshot_positions[screenshot]
    next_screenshot = min(
        (candidate for candidate in screenshot_positions.values() if candidate > position),
        default=len(steps),
    )
    evidence = steps[position + 1 : next_screenshot]
    if not any(
        step.get("type") == "assert_html_browser_origin"
        and step.get("origin_ends_with") == expected_origin_suffix
        for step in evidence
    ):
        raise SystemExit(
            f"acceptance scenario must prove the complete browser origin after {screenshot}"
        )
    if not any(
        step.get("type") == "assert_html_browser_frame_contains_rgb"
        and step.get("rgb") == expected_rgb
        and step.get("min_pixels", 0) > 0
        for step in evidence
    ):
        raise SystemExit(
            f"acceptance scenario must prove the raw KRR frame after {screenshot}"
        )


require_browser_state("01-initial-render", "/app/index.html", [38, 184, 166])
require_browser_state(
    "06-fragment-navigation", "/app/index.html#fragment-target", [214, 245, 227]
)
for screenshot in (
    "07-link-navigation",
    "08-reloaded-linked-panel",
    "09-resized-linked-panel",
):
    require_browser_state(
        screenshot, "linked-panel.html#linked-target", [232, 199, 255]
    )

active_document_positions = [
    (position, step.get("path_contains"))
    for position, step in enumerate(steps)
    if step.get("type") == "assert_active_document"
]
fragment_position = screenshot_positions["06-fragment-navigation"]
navigation_position = screenshot_positions["07-link-navigation"]
if not any(
    fragment_position < position < navigation_position and path == "Katana://URL"
    for position, path in active_document_positions
):
    raise SystemExit("acceptance scenario must keep index.html active after same-document fragment navigation")
for screenshot in ("07-link-navigation", "08-reloaded-linked-panel", "09-resized-linked-panel"):
    position = screenshot_positions[screenshot]
    next_screenshot = min(
        (candidate for candidate in screenshot_positions.values() if candidate > position),
        default=len(steps),
    )
    if not any(
        position < assertion_position < next_screenshot and path == "Katana://URL"
        for assertion_position, path in active_document_positions
    ):
        raise SystemExit(f"acceptance scenario must keep linked-panel.html active after {screenshot}")

index = index_path.read_text()
for marker in (
    'id="app-root"',
    'rel="stylesheet" href="style.css"',
    'id="resource-image" src="resource-image.svg"',
    'id="embedded-mermaid-svg"',
    "<details",
    "id=\"action\"",
    "id=\"text-input\"",
    "linked-panel.html#linked-target",
    "id=\"prevented-link\"",
    "id=\"fragment-link\"",
    "id=\"scroll-target\"",
    "id=\"fragment-target\"",
    '<script src="actions.js"></script>',
):
    if marker not in index:
        raise SystemExit(f"acceptance index fixture is missing marker: {marker}")
style = style_path.read_text()
for marker in (
    ":root",
    "--page-edge",
    "box-sizing: border-box",
    "grid-template-columns",
    "var(--space)",
    "!important",
    "@media (max-width: 900px)",
    ".status",
    "#e6f5ef",
    "#4f9dff",
    "#fff4cc",
    "#b99aff",
    "#ffb6c1",
    "#c6f6d5",
    "#f0a35e",
):
    if marker not in style:
        raise SystemExit(f"acceptance stylesheet fixture is missing marker: {marker}")
script = script_path.read_text()
for marker in (
    "document.addEventListener('DOMContentLoaded'",
    "DOMContentLoaded executed by KRR V8",
    "#b8f2d0",
    "#ffe08a",
    "#a7ddff",
    "event.preventDefault()",
    "event.stopPropagation()",
    "Parent click listener must not run",
    "#ffd1dc",
    "Same-document fragment requested by KRR V8",
    "Input state preserved:",
    "#d6f5e3",
):
    if marker not in script:
        raise SystemExit(f"acceptance script fixture is missing marker: {marker}")
linked = linked_path.read_text()
for marker in ("Linked fragment target loaded by KRR", "#e8c7ff"):
    if marker not in linked:
        raise SystemExit(f"acceptance linked fixture is missing its navigation marker: {marker}")
image = image_path.read_text()
for marker in ('xmlns="http://www.w3.org/2000/svg"', '#26b8a6', 'External resource pipeline'):
    if marker not in image:
        raise SystemExit(f"acceptance image fixture is missing marker: {marker}")
PY
then
    error "Interactive headless evidence contract is incomplete."
    exit 1
fi

if ! python3 - "$IMAGE_CONTROLS_REQUEST" <<'PY'
import json
import sys
from pathlib import Path

request = json.loads(Path(sys.argv[1]).read_text())
if request.get("name") != "v0-22-36-light-image-controls":
    raise SystemExit("image controls request must target v0.22.36")

settings = request.get("fixture", {}).get("settings", {})
if settings.get("theme") != "light" or settings.get("preview_show_diagram_controls") is not True:
    raise SystemExit("image controls evidence must use the light theme with controls enabled")
workspace_files = request.get("fixture", {}).get("workspace_files", [])
if not any(entry.get("name") == "light-image-controls.png" for entry in workspace_files):
    raise SystemExit("image controls evidence must load the white-background image fixture")

steps = request.get("steps", [])
node_clicks = {
    step.get("action", {}).get("click_node", {}).get("label")
    for step in steps
    if isinstance(step.get("action"), dict)
}
if not {"Fullscreen", "Zoom In"}.issubset(node_clicks):
    raise SystemExit("image controls evidence must enter fullscreen and zoom the image")

directions = {
    step.get("direction")
    for step in steps
    if step.get("type") == "scroll" and step.get("pixels", 0) > 0
}
if directions != {"right", "down", "left", "up"}:
    raise SystemExit("image controls evidence must scroll right, down, left, and up")

screenshots = {
    step.get("output_name")
    for step in steps
    if step.get("type") == "screenshot"
}
required_screenshots = {
    "01-light-image-controls",
    "02-fullscreen-controls",
    "03-fullscreen-zoomed",
    "04-fullscreen-scroll-right",
    "05-fullscreen-scroll-down",
    "06-fullscreen-scroll-left",
    "07-fullscreen-scroll-up",
}
if missing := sorted(required_screenshots - screenshots):
    raise SystemExit(f"image controls evidence is missing screenshots: {missing}")

expected_changes = {
    ("03-fullscreen-zoomed", "04-fullscreen-scroll-right"),
    ("04-fullscreen-scroll-right", "05-fullscreen-scroll-down"),
    ("05-fullscreen-scroll-down", "06-fullscreen-scroll-left"),
    ("06-fullscreen-scroll-left", "07-fullscreen-scroll-up"),
}
actual_changes = {
    (step.get("baseline"), step.get("current"))
    for step in steps
    if step.get("type") == "assert_screenshot_changed"
    and step.get("min_changed_pixels", 0) > 0
}
if not expected_changes.issubset(actual_changes):
    raise SystemExit("image controls evidence must assert a rendered change after every pan")

if not any(
    step.get("type") == "assert_screenshot_contains_rgb"
    and step.get("screenshot") == "01-light-image-controls"
    and step.get("rgb") == [106, 106, 106]
    and step.get("min_pixels", 0) >= 1000
    for step in steps
):
    raise SystemExit("image controls evidence must prove the fixed control background on white")
PY
then
    error "Light image controls and fullscreen pan evidence contract is incomplete."
    exit 1
fi

if ! python3 - "$CARGO_TOML" "$CARGO_LOCK" "$CARGO_CONFIG" "$ACCEPTANCE_CARGO_TOML" "$ACCEPTANCE_CARGO_LOCK" <<'PY'
import re
import sys
import tomllib
from pathlib import Path

cargo_path = Path(sys.argv[1])
lock_path = Path(sys.argv[2])
config_path = Path(sys.argv[3])
acceptance_cargo_path = Path(sys.argv[4])
acceptance_lock_path = Path(sys.argv[5])

with cargo_path.open("rb") as handle:
    cargo = tomllib.load(handle)
with lock_path.open("rb") as handle:
    lock = tomllib.load(handle)
with acceptance_cargo_path.open("rb") as handle:
    acceptance_cargo = tomllib.load(handle)
with acceptance_lock_path.open("rb") as handle:
    acceptance_lock = tomllib.load(handle)

dependencies = cargo.get("workspace", {}).get("dependencies", {})
manifest_release_lines = {
    "katana-document-viewer": (0, 3, 3),
    "katana-render-runtime": (0, 4, 6),
}
minimum_lock_versions = {
    "katana-document-viewer": (0, 3, 3),
    "katana-render-runtime": (0, 4, 6),
}


def dependency_version(name: str, expected: tuple[int, int, int]) -> None:
    value = dependencies.get(name)
    if value is None:
        raise SystemExit(f"missing workspace dependency: {name}")
    if isinstance(value, dict):
        forbidden = [key for key in ("path", "git") if key in value]
        if forbidden:
            raise SystemExit(f"{name} uses forbidden dependency source: {forbidden[0]}")
        version = value.get("version")
    else:
        version = value
    if not isinstance(version, str):
        raise SystemExit(f"{name} must declare a crates.io version")
    match = re.fullmatch(r"\^?(\d+)\.(\d+)\.(\d+)", version)
    if match is None:
        raise SystemExit(f"{name} must use a single caret-compatible x.y.z requirement: {version}")
    actual = tuple(int(part) for part in match.groups())
    if actual != expected:
        raise SystemExit(
            f"{name} must declare the {expected[0]}.{expected[1]}.{expected[2]} release line; found {version}"
        )


for dependency, expected_line in manifest_release_lines.items():
    dependency_version(dependency, expected_line)

patch_sources = [
    ("Cargo.toml", cargo.get("patch", {})),
    ("scripts/screenshot/Cargo.toml", acceptance_cargo.get("patch", {})),
]
if config_path.exists():
    with config_path.open("rb") as handle:
        config = tomllib.load(handle)
    patch_sources.append((str(config_path), config.get("patch", {})))

for source_label, patches in patch_sources:
    if not isinstance(patches, dict):
        continue
    for registry, overrides in patches.items():
        if not isinstance(overrides, dict):
            continue
        for dependency in manifest_release_lines:
            if dependency in overrides:
                raise SystemExit(
                    f"{dependency} must not be overridden in {source_label} [patch.{registry}]"
                )

for replacement in cargo.get("replace", {}):
    for dependency in manifest_release_lines:
        if replacement == dependency or replacement.startswith(f"{dependency}:"):
            raise SystemExit(f"{dependency} must not be overridden in [replace]")

registry_source = "registry+https://github.com/rust-lang/crates.io-index"


def validate_lock(label: str, lock_document: dict) -> None:
    packages = lock_document.get("package", [])
    for dependency, expected_line in manifest_release_lines.items():
        matches = [package for package in packages if package.get("name") == dependency]
        if len(matches) != 1:
            raise SystemExit(f"{label} must resolve exactly one {dependency} package")
        package = matches[0]
        version = package.get("version", "")
        version_match = re.fullmatch(r"(\d+)\.(\d+)\.(\d+)", version)
        if version_match is None:
            raise SystemExit(f"{label} has invalid {dependency} version: {version}")
        actual = tuple(int(part) for part in version_match.groups())
        minimum = minimum_lock_versions[dependency]
        if actual[:2] != expected_line[:2] or actual < minimum:
            raise SystemExit(
                f"{label} must resolve {dependency} at least "
                f"{minimum[0]}.{minimum[1]}.{minimum[2]} on the "
                f"{expected_line[0]}.{expected_line[1]}.x line; found {version}"
            )
        if package.get("source") != registry_source:
            raise SystemExit(f"{label} must resolve {dependency} from crates.io")
        checksum = package.get("checksum", "")
        if re.fullmatch(r"[0-9a-f]{64}", checksum) is None:
            raise SystemExit(f"{label} must checksum the crates.io {dependency} package")


validate_lock("Cargo.lock", lock)
validate_lock("scripts/screenshot/Cargo.lock", acceptance_lock)
PY
then
    error "KDV/KRR dependency contract is not release-ready."
    exit 1
fi

required_markers=(
    "Browser-equivalent HTML session is the only interactive preview path"
    "The system MUST NOT fall back to static HTML rendering"
    "v0.22.36 release must prove the published browser chain"
    'minimum resolved version of KDV `0.3.3` and KRR `0.4.6`'
    "raw KRR frame pixels"
)
for marker in "${required_markers[@]}"; do
    if ! grep -Fq "$marker" "$HTML_SPEC"; then
        error "Canonical HTML OpenSpec is missing required contract: ${marker}"
        exit 1
    fi
done

if [[ ! -x "$RUNTIME_GUARD" ]]; then
    error "Packaged HTML browser runtime contract guard is missing or not executable: ${RUNTIME_GUARD}"
    exit 1
fi
if ! "$RUNTIME_GUARD" "$TARGET_VERSION"; then
    error "Packaged HTML browser runtime contract failed for v${TARGET_VERSION}."
    exit 1
fi

success "Browser-equivalent HTML release contract is satisfied for v${TARGET_VERSION}."
