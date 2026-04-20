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

if ! command -v cargo &>/dev/null; then
  echo "ERROR: cargo not found — install Rust via https://rustup.rs" >&2
  exit 1
fi

echo "[katana-screenshot] building runner..."
cargo build --release --manifest-path "${SCRIPT_DIR}/Cargo.toml" --quiet

RUNNER="${SCRIPT_DIR}/target/release/katana-screenshot"

exec "$RUNNER" "$@"
