#!/bin/bash
set -euo pipefail

# 不足objectを取得せず照合するため、version文字列ではなく実際の機能を確認する。
if ! git --no-lazy-fetch --version >/dev/null 2>&1; then
    printf '%s\n' 'Git 2.45+ with --no-lazy-fetch is required for development/release evidence checks.' >&2
    printf '%s\n' 'Upgrade Git and ensure the upgraded executable is first on PATH; macOS: brew upgrade git.' >&2
    exit 1
fi
