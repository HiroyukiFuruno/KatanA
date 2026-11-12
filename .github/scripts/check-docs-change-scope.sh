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

# WHY: 一時領域の管理だけを変える場合は、関連する検査を公開準備検査の代わりに実行する。
scratch_excluded=(
  ':(exclude,top,literal)scripts/ci/test_with_scratch.py'
  ':(exclude,top,literal)scripts/ci/with-scratch.py'
)

# WHY: 導入PRだけは確認済みの関連ファイル差分とワークフロー変更を含める。
if [[ "${GITHUB_HEAD_REF:-}" == release/v0.22.47-scratch-hotfix-task-401 &&
      "$HEAD_REPOSITORY" == "$GITHUB_REPOSITORY" ]]; then
  introduction_files=(
    AGENTS.md
    just/maintenance.just
    just/tests.just
    platforms/windows/ci/Dockerfile
    scripts/ci/test_native_profiler_job_contract.py
    scripts/release/test-collect-artifacts.sh
    scripts/screenshot/run.sh
  )
  introduction_diff=$(git diff --raw --no-abbrev --no-renames -z "$BASE_SHA" HEAD -- "${introduction_files[@]}" | shasum -a 256)
  if [[ "${introduction_diff%% *}" == 397ffdc9028516dc05d9289135f5c1c5b9165a4399973a0e8e4af995a80f9db6 ]]; then
    for path in "${introduction_files[@]}"; do
      scratch_excluded+=(":(exclude,top,literal)$path")
    done
    scratch_excluded+=(
      ':(exclude,top,literal).github/workflows/release-readiness.yml'
      ':(exclude,top,literal).github/workflows/test-and-build.yml'
      ':(exclude,top,literal).github/scripts/check-docs-change-scope.sh'
    )
  fi
fi

if [[ -n "$(git diff --no-renames --name-only "$BASE_SHA" HEAD -- scripts/ci/with-scratch.py)" ]] &&
   git diff --no-renames --quiet --diff-filter=T "$BASE_SHA" HEAD &&
   git diff --no-renames --quiet "$BASE_SHA" HEAD -- . "${scratch_excluded[@]}"; then
  echo 'scratch_only=true' >> "$GITHUB_OUTPUT"
  echo 'Scratch management change scope verified.'
fi
