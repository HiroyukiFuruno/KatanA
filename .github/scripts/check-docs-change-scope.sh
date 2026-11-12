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

# WHY: #423 は公開準備だけを対象検査へ切り替え、製品テストと lint は通常どおり実行する。
if [[ "$PR_NUMBER" == 423 && "${GITHUB_HEAD_REF:-}" == release/v0.22.47-scratch-hotfix-task-401 &&
      "$HEAD_REPOSITORY" == "$GITHUB_REPOSITORY" ]]; then
  introduction_files=(
    .github/workflows/build-and-release.yml
    .github/workflows/release-readiness.yml
    .github/workflows/test-and-build.yml
    AGENTS.md
    crates/katana-linter/src/utils/parser.rs
    crates/katana-platform/src/cache/default/migration.rs
    crates/katana-ui/src/preview_pane/document_surface/controls_sheet_tabs.rs
    crates/katana-ui/src/shell_ui/mod.rs
    just/maintenance.just
    just/tests.just
    platforms/windows/ci/Dockerfile
    scripts/ci/check-linux-office-worker-contract.sh
    scripts/ci/check-native-office-worker-contract.sh
    scripts/ci/coverage.sh
    scripts/ci/test_cargo_clean_guard.py
    scripts/ci/test_cargo_clean_process_guard.py
    scripts/ci/test_cargo_sweep_guard.py
    scripts/ci/test_cargo_target_lease_guard.py
    scripts/ci/test_isolated_test_config.py
    scripts/ci/test_native_profiler_job_contract.py
    scripts/ci/test_windows_compiled_cache_contract.py
    scripts/ci/with-office-test-worker.sh
    scripts/release/check-multi-format-document-contract.py
    scripts/release/test-collect-artifacts.sh
    scripts/release/test_worker_snapshot.py
    scripts/screenshot/run.sh
  )
  introduction_diff=$(git diff --raw --no-abbrev --no-renames -z "$BASE_SHA" HEAD -- "${introduction_files[@]}" | shasum -a 256)
  if [[ "${introduction_diff%% *}" == ca2d4c41cd4ff23578a59977d55c17fc61471c45d7f2cb9e6bfba9ad67eec442 ]]; then
    hotfix_excluded=("${scratch_excluded[@]}" ':(exclude,top,literal).github/scripts/check-docs-change-scope.sh')
    for path in "${introduction_files[@]}"; do
      hotfix_excluded+=(":(exclude,top,literal)$path")
    done
    if git diff --no-renames --quiet --diff-filter=T "$BASE_SHA" HEAD &&
       git diff --no-renames --quiet "$BASE_SHA" HEAD -- . "${hotfix_excluded[@]}"; then
      echo 'non_release_hotfix=true' >> "$GITHUB_OUTPUT"
      echo 'Non-release hotfix scope verified. Full product CI is required.'
    fi
  fi
fi

if [[ -n "$(git diff --no-renames --name-only "$BASE_SHA" HEAD -- scripts/ci/with-scratch.py)" ]] &&
   git diff --no-renames --quiet --diff-filter=T "$BASE_SHA" HEAD &&
   git diff --no-renames --quiet "$BASE_SHA" HEAD -- . "${scratch_excluded[@]}"; then
  echo 'scratch_only=true' >> "$GITHUB_OUTPUT"
  echo 'Scratch management change scope verified.'
fi
