#!/bin/bash
set -euo pipefail

branch=$(git symbolic-ref --quiet --short HEAD)
if [[ "$branch" == "master" ]]; then
    exit 0
fi

# WHY: Issue契約を既存commit-msg経路へ追加し、masterの公開履歴生成は変更しない。
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
exec python3 "$script_dir/check-commit-issue-reference.py" \
    --repository HiroyukiFuruno/KatanA --verify-live "${1:?commit message file is required}"
