#!/bin/bash
set -euo pipefail

WORKFLOW=${KATANA_CI_WORKFLOW_FILE:-.github/workflows/test-and-build.yml}

if [[ ! -f "$WORKFLOW" ]]; then
    echo "FAIL: CI workflow not found: $WORKFLOW" >&2
    exit 1
fi

if ! grep -qF "JOBS=1 just coverage" "$WORKFLOW"; then
    echo "FAIL: macOS coverage must use one linker job" >&2
    exit 1
fi

echo "PASS: macOS coverage uses one linker job"
