#!/usr/bin/env bash
set -euo pipefail

patterns=(
  '*.md'
  '*.txt'
  'docs/**/*.md'
  'docs/**/*.txt'
  'docs/**/*.rst'
  'docs/**/*.adoc'
)
documents=()
excluded=()
for pattern in "${patterns[@]}"; do
  documents+=(":(top,glob)$pattern")
  excluded+=(":(exclude,top,glob)$pattern")
done

# WHY: #422 は文書と、この判定を導入する CI ファイルを同時に更新する。
if [[ "$PR_NUMBER" == 422 && "$HEAD_REPOSITORY" == "$GITHUB_REPOSITORY" ]]; then
  excluded+=(
    ':(exclude,top,literal).github/workflows/release-readiness.yml'
    ':(exclude,top,literal).github/workflows/test-and-build.yml'
    ':(exclude,top,literal).github/scripts/check-docs-change-scope.sh'
  )
fi

# WHY: リリース記録と作業ルールの変更、文書のファイル種別変更は通常の検査に回す。
if [[ -n "$(git diff --no-renames --name-only "$BASE_SHA" HEAD -- "${documents[@]}")" ]] &&
   git diff --no-renames --quiet "$BASE_SHA" HEAD -- ':(top,glob)CHANGELOG*' AGENTS.md CLAUDE.md &&
   git diff --no-renames --quiet --diff-filter=T "$BASE_SHA" HEAD &&
   git diff --no-renames --quiet "$BASE_SHA" HEAD -- . "${excluded[@]}"; then
  echo 'docs_only=true' >> "$GITHUB_OUTPUT"
  echo 'Documentation change scope verified.'
fi
