#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."
command -v cargo-deny >/dev/null || {
    echo "cargo-deny is required. Install cargo-deny 0.20.2 or use the pinned CI installer." >&2
    exit 127
}
python3 -B -m unittest discover -s scripts/ci -p test_zip_advisory_overlay.py
python3 scripts/ci/zip_advisory_overlay.py --deny-file deny.toml \
    Cargo.toml scripts/screenshot/Cargo.toml scripts/paint-metrics/Cargo.toml
