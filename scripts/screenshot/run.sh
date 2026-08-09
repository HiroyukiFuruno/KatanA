#!/usr/bin/env bash
# KatanA screenshot runner entry point.
#
# Builds the katana-screenshot binary (if needed) and executes a request file.
#
# Usage:
#   ./run.sh --request <request.json> --output <output_dir> --binary <path/to/KatanA>
#
# Requirements (macOS):
#   - Rust toolchain (cargo)
#   - Accessibility permission granted to Terminal / the shell process
#     (System Settings > Privacy & Security > Accessibility)
#
# Requirements (Linux):
#   - Rust toolchain (cargo)
#   - scrot or ImageMagick (import) for screenshot capture

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BUILD_TARGET_DIR="${REPO_ROOT}/target"

if ! command -v cargo &>/dev/null; then
  echo "ERROR: cargo not found — install Rust via https://rustup.rs" >&2
  exit 1
fi

echo "[katana-screenshot] building runner..."
CARGO_TARGET_DIR="${BUILD_TARGET_DIR}" cargo build --release --manifest-path "${REPO_ROOT}/Cargo.toml" --package katana-ui --bin kdv-office-worker --quiet
OFFICE_WORKER="${BUILD_TARGET_DIR}/release/kdv-office-worker"
if [[ -f "${OFFICE_WORKER}.exe" ]]; then
  OFFICE_WORKER="${OFFICE_WORKER}.exe"
fi
if [[ ! -x "${OFFICE_WORKER}" && ! -f "${OFFICE_WORKER}" ]]; then
  echo "ERROR: Office worker was not built at ${OFFICE_WORKER}" >&2
  exit 1
fi
export KATANA_KDV_OFFICE_WORKER="${OFFICE_WORKER}"
CARGO_TARGET_DIR="${BUILD_TARGET_DIR}" cargo build --release --manifest-path "${SCRIPT_DIR}/Cargo.toml" --quiet

RUNNER="${BUILD_TARGET_DIR}/release/katana-screenshot"
if [[ -f "${RUNNER}.exe" ]]; then
  RUNNER="${RUNNER}.exe"
fi

exec "$RUNNER" "$@"
