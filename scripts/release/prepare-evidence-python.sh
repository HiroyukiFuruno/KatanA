#!/usr/bin/env bash
set -euo pipefail

# WHY: 公開記録と既存受入検査を同じ固定依存環境で実行する。
environment="tmp/release-evidence-python"
if [[ ! -x "$environment/bin/python" ]]; then
    python3 -m venv "$environment"
fi
"$environment/bin/python" -m pip install --disable-pip-version-check --only-binary=:all: -r scripts/release/evidence-requirements.txt
