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
if [[ "$TARGET_VERSION" != "0.22.33" ]]; then
    error "The HTML runtime contract applies only to v0.22.33; received v${TARGET_VERSION}."
    exit 1
fi

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=${KATANA_RELEASE_ROOT:-$(cd "${SCRIPT_DIR}/../.." && pwd)}
SURFACE=${KATANA_HTML_SURFACE:-${ROOT_DIR}/crates/katana-ui/src/preview_pane/image_html_surface.rs}
SURFACE_INPUT="${SURFACE%.rs}_input.rs"
SURFACE_PANE="${SURFACE%.rs}_pane.rs"
SURFACE_VIEW="${SURFACE%.rs}_view.rs"
HTML_RENDER=${KATANA_HTML_RENDER:-${ROOT_DIR}/crates/katana-ui/src/preview_pane/core_render_html_document.rs}
URL_SOURCE=${KATANA_HTML_URL_SOURCE:-${ROOT_DIR}/crates/katana-ui/src/app/action/url_source.rs}
UI_SOURCE_ROOT=${KATANA_HTML_UI_SOURCE_ROOT:-${ROOT_DIR}/crates/katana-ui/src}
RUNTIME_MANIFESTS_RAW=${KATANA_HTML_RUNTIME_MANIFESTS:-${ROOT_DIR}/Cargo.toml:${ROOT_DIR}/crates/katana-ui/Cargo.toml}
IFS=: read -r -a RUNTIME_MANIFESTS <<<"$RUNTIME_MANIFESTS_RAW"
SURFACE_FILES=("$SURFACE" "$SURFACE_INPUT" "$SURFACE_PANE" "$SURFACE_VIEW")

for required_file in "${SURFACE_FILES[@]}" "$HTML_RENDER" "$URL_SOURCE" "${RUNTIME_MANIFESTS[@]}"; do
    if [[ ! -r "$required_file" ]]; then
        error "Required HTML runtime integration file is missing: ${required_file}"
        exit 1
    fi
done

if [[ ! -d "$UI_SOURCE_ROOT" ]]; then
    error "KatanA UI source root is missing: ${UI_SOURCE_ROOT}"
    exit 1
fi

for required_marker in \
    "BrowserSessionAdapter" \
    "HtmlBrowserSource" \
    "take_html_browser_navigation"; do
    if ! grep -Fq "$required_marker" "${SURFACE_FILES[@]}"; then
        error "HTML surface is missing KDV/KRR runtime integration: ${required_marker}"
        exit 1
    fi
done

if ! grep -Fq "HtmlBrowserInput" "$SURFACE_INPUT"; then
    error "HTML input surface is missing KDV/KRR runtime integration: HtmlBrowserInput"
    exit 1
fi

for required_marker in \
    "HtmlBrowserSurface::start" \
    "full_render_html_document" \
    "full_render_html_source"; do
    if ! grep -Fq "$required_marker" "$HTML_RENDER"; then
        error "HTML document rendering does not select the interactive surface: ${required_marker}"
        exit 1
    fi
done

if ! grep -Fq "apply_fetched_html_source(source" "$URL_SOURCE"; then
    error "Fetched HTML URLs are not delivered to the KatanA document state."
    exit 1
fi

for forbidden_runtime_marker in \
    "Chromium" \
    "WebView" \
    "headless_chrome" \
    "chromiumoxide" \
    "KRR_CHROME" \
    "HtmlBrowserProcess" \
    "HtmlBrowserProcessConfig" \
    "wry::" \
    "webkit2gtk" \
    "cef::"; do
    if grep -R -Fq --include='*.rs' "$forbidden_runtime_marker" "$UI_SOURCE_ROOT" || \
        grep -Fq "$forbidden_runtime_marker" "${RUNTIME_MANIFESTS[@]}"; then
        error "KatanA UI must not depend on external browser runtime marker ${forbidden_runtime_marker}."
        exit 1
    fi
done

for forbidden_dependency in \
    "headless_chrome" \
    "chromiumoxide" \
    "wry" \
    "webkit2gtk" \
    "cef" \
    "fantoccini" \
    "thirtyfour" \
    "playwright"; do
    if grep -Eiq "^[[:space:]]*\"?${forbidden_dependency}\"?[[:space:]]*=" "${RUNTIME_MANIFESTS[@]}"; then
        error "KatanA UI must not declare external browser dependency ${forbidden_dependency}."
        exit 1
    fi
done

for forbidden_interactive_marker in "HtmlRenderer" "HtmlParser"; do
    if grep -Fq "$forbidden_interactive_marker" "${SURFACE_FILES[@]}" "$HTML_RENDER" "$URL_SOURCE"; then
        error "Interactive HTML integration must not depend on ${forbidden_interactive_marker}."
        exit 1
    fi
done

for forbidden_path in \
    "${ROOT_DIR}/vendor/chromium" \
    "${ROOT_DIR}/crates/katana-ui/vendor/chromium" \
    "${ROOT_DIR}/resources/chromium"; do
    if [[ -e "$forbidden_path" ]]; then
        error "KatanA must not package an external browser runtime: ${forbidden_path}"
        exit 1
    fi
done

success "KatanA HTML integration uses the KDV/KRR in-process Rust/V8 runtime only."
