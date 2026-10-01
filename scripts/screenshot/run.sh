#!/usr/bin/env bash
# KatanA screenshot runner entry point.
#
# Builds the katana-screenshot binary (if needed) and executes a request file.
#
# Usage:
#   ./run.sh --request <request.json> --output <output_dir>
#
# Runs KatanaApp in-process with egui_kittest, not a packaged executable.
# Packaged startup is checked separately by scripts/release/smoke-launch-packaged-app.sh.
# Requires a Rust toolchain and a supported graphics backend.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BUILD_TARGET_DIR="${REPO_ROOT}/target"
SCREENSHOT_TARGET_DIR="${BUILD_TARGET_DIR}/screenshot-harness"
EXTERNAL_HYPERLINK_FIXTURE="${BUILD_TARGET_DIR}/screenshot-fixtures/representative-with-external-hyperlink.pptx"

if ! command -v cargo &>/dev/null; then
  echo "ERROR: cargo not found — install Rust via https://rustup.rs" >&2
  exit 1
fi

python3 "${SCRIPT_DIR}/generate_external_hyperlink_pptx.py" \
  --source "${SCRIPT_DIR}/fixtures/v0-22-38-multi-format/representative.pptx" \
  --output "${EXTERNAL_HYPERLINK_FIXTURE}"

echo "[katana-screenshot] building runner..."
CARGO_TARGET_DIR="${BUILD_TARGET_DIR}" cargo build --locked --release --manifest-path "${REPO_ROOT}/Cargo.toml" --package katana-ui --bin kdv-office-worker --quiet
OFFICE_WORKER="${BUILD_TARGET_DIR}/release/kdv-office-worker"
if [[ -f "${OFFICE_WORKER}.exe" ]]; then
  OFFICE_WORKER="${OFFICE_WORKER}.exe"
fi
if [[ ! -x "${OFFICE_WORKER}" && ! -f "${OFFICE_WORKER}" ]]; then
  echo "ERROR: Office worker was not built at ${OFFICE_WORKER}" >&2
  exit 1
fi
export KATANA_KDV_OFFICE_WORKER="${OFFICE_WORKER}"
# 別lockfileの生成物を本体のtargetへ混在させず、実ファイル受入と同じ配置にする。
CARGO_TARGET_DIR="${SCREENSHOT_TARGET_DIR}" cargo build --locked --release --manifest-path "${SCRIPT_DIR}/Cargo.toml" --quiet

RUNNER="${SCREENSHOT_TARGET_DIR}/release/katana-screenshot"
if [[ -f "${RUNNER}.exe" ]]; then
  RUNNER="${RUNNER}.exe"
fi

exec "$RUNNER" "$@"
