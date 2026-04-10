#!/bin/bash
set -euo pipefail

# Usage: ./scripts/release/collect-artifacts.sh <output_dir>
OUTPUT_DIR=${1:-"release-files"}

echo "📦 Organizing artifacts into ${OUTPUT_DIR}..."
mkdir -p "${OUTPUT_DIR}"

# Move artifacts from all platforms
# Using '|| true' to avoid failure if some artifacts are missing in partial runs
mv artifacts-macos/* "${OUTPUT_DIR}/" 2>/dev/null || true
mv artifacts-linux/* "${OUTPUT_DIR}/" 2>/dev/null || true
mv artifacts-windows/* "${OUTPUT_DIR}/" 2>/dev/null || true

cd "${OUTPUT_DIR}"

echo "🔐 Generating checksums.txt..."
# Generate checksums for all files except the checksums file itself
find . -maxdepth 1 -type f ! -name "checksums.txt" -exec shasum -a 256 {} + > checksums.txt

echo "✅ Artifacts collected:"
ls -lh
