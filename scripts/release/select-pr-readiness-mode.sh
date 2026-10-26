#!/usr/bin/env bash
set -euo pipefail

branch=${1-}
target_version=${2-}
requested_mode=${3-}

# WHY: 公開後の自動検証へ切り替える対象を完全一致に限定し、他経路のstrict条件を保持する。
if [[ "$branch" == "release/v0.22.42-public-release-evidence" &&
      "$target_version" == "0.22.42" &&
      ( -z "$requested_mode" || "$requested_mode" == "pr-bootstrap" ) ]]; then
    printf '%s\n' 'post-release-automated'
else
    printf '%s\n' "$requested_mode"
fi

