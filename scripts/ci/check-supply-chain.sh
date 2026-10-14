#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."
command -v cargo-deny >/dev/null || {
    echo "cargo-deny is required. Install cargo-deny 0.20.2 or use the pinned CI installer." >&2
    exit 127
}
cargo deny --locked check --hide-inclusion-graph
cargo deny --locked --manifest-path scripts/screenshot/Cargo.toml check --hide-inclusion-graph
cargo deny --locked --manifest-path scripts/paint-metrics/Cargo.toml check --hide-inclusion-graph
