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

# WHY: 実アプリの既定リポジトリを使う検査でも、個人の設定や履歴を参照させない。
mkdir -p "$ROOT_DIR/tmp"
KATANA_CONFIG_DIR=$(mktemp -d "$ROOT_DIR/tmp/document-fixture-config.XXXXXX")
export KATANA_CONFIG_DIR
printf 'ISOLATED_FIXTURE_PROFILE=%s\n' "$KATANA_CONFIG_DIR"

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
KATANA_KDV_OFFICE_WORKER="$OFFICE_WORKER" cargo test --locked -p katana-ui --lib --features external-fixture-acceptance \
    views::panels::preview::side_panels::slideshow_tests::pptx_ready_tests::external_pptx_ready_slideshow_accepts_active_frame_and_rejects_stale_tab \
    -- --exact --nocapture
cargo test --locked -p katana-ui --lib --features external-fixture-acceptance \
    preview_pane::image_html_surface::tests::external_requirements_fragment_keeps_the_sticky_sidebar_in_the_first_frame \
    -- --exact --nocapture
