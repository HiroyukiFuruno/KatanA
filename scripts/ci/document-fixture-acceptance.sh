#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$ROOT_DIR"

: "${KATANA_DOCUMENT_FIXTURE_DIR:?KATANA_DOCUMENT_FIXTURE_DIR must name the supplied Office fixture directory}"
: "${KATANA_HTML_FIXTURE:?KATANA_HTML_FIXTURE must name the supplied requirements HTML fixture}"
[[ -d "$KATANA_DOCUMENT_FIXTURE_DIR" ]] || {
    echo "KATANA_DOCUMENT_FIXTURE_DIR is not a directory: $KATANA_DOCUMENT_FIXTURE_DIR" >&2
    exit 1
}
[[ -r "$KATANA_HTML_FIXTURE" ]] || {
    echo "KATANA_HTML_FIXTURE is not readable: $KATANA_HTML_FIXTURE" >&2
    exit 1
}

TARGET_DIR=${CARGO_TARGET_DIR:-"$ROOT_DIR/target"}
if [[ "$TARGET_DIR" != /* ]]; then
    TARGET_DIR="$ROOT_DIR/$TARGET_DIR"
fi
OFFICE_WORKER="$TARGET_DIR/release/kdv-office-worker"
if [[ "${OS:-}" == "Windows_NT" ]]; then
    OFFICE_WORKER+=".exe"
fi

cargo build --locked --release -p katana-ui --bin kdv-office-worker
[[ -x "$OFFICE_WORKER" ]] || {
    echo "release office worker is missing or not executable: $OFFICE_WORKER" >&2
    exit 1
}

KATANA_KDV_OFFICE_WORKER="$OFFICE_WORKER" cargo test --locked -p katana-ui --lib --features external-fixture-acceptance \
    preview_pane::document_surface::tests::external_document_fixture_dir_reports_typed_first_frame_results \
    -- --exact --nocapture
cargo test --locked -p katana-ui --lib --features external-fixture-acceptance \
    preview_pane::image_html_surface::tests::external_requirements_fragment_keeps_the_sticky_sidebar_in_the_first_frame \
    -- --exact --nocapture
