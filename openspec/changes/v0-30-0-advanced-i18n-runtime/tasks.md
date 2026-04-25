## Branch Rule

本タスクでは、以下のブランチ運用を適用します：
- **標準（Base）ブランチ**: `v0-30-0-advanced-i18n-runtime` またはリリース用統合ブランチ（例: `release/v0.30.0`）
- **作業ブランチ**: 標準は `v0-30-0-advanced-i18n-runtime-task-x`、リリース用は `feature/v0.30.0-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

## 1. I18n Runtime Safety

実装状態: `i18n-runtime-safety` Task 1 として `master` に実装済み。この change では同じ runtime fallback を再実装せず、Task 2 以降へ進む。

### Definition of Done (DoD)

- [x] Replace unsupported runtime language panic paths with deterministic fallback to the configured default language.
- [x] Preserve test or startup failure for embedded locale files that are structurally invalid.
- [x] Add regression tests for unknown language code, supported language lookup, and fallback message lookup.
- [x] Runtime fallback は `185d2913 fix: 未対応言語を安全にフォールバック` で `master` へ反映済み。

- [x] 1.1 Add a fallback-aware language resolver for `I18nOps`.
- [x] 1.2 Replace dictionary lookup `expect` / `panic!` paths that can be reached from runtime language values.
- [x] 1.3 Add tests that demonstrate unsupported user settings do not crash application startup.

## 2. Formatter Adapter

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### Definition of Done (DoD)

- [ ] Introduce a KatanA-owned formatter adapter for message identifiers and typed named arguments.
- [ ] Route existing `I18nOps::tf` behavior through the adapter as a compatibility layer.
- [ ] Document which formatter types may cross UI boundaries.
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

- [ ] 2.1 Define formatter input/output types and fallback error behavior.
- [ ] 2.2 Update representative UI call sites to use the adapter through existing i18n wrappers.
- [ ] 2.3 Add unit tests for missing parameter, extra parameter, and escaped brace behavior.

## 3. Fluent / ICU Candidate Selection

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### Definition of Done (DoD)

- [ ] Evaluate Fluent and ICU-compatible Rust formatting options behind the formatter adapter.
- [ ] Record the selected engine, rejected alternatives, binary-size impact, startup impact, and migration constraints.
- [ ] Migrate a small set of plural-sensitive messages through the selected path.
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

- [ ] 3.1 Inventory messages that include counts, problem totals, file counts, or other plural-sensitive values.
- [ ] 3.2 Spike Fluent and ICU-compatible formatting behind the adapter with embedded resources.
- [ ] 3.3 Select one engine or explicitly defer if neither meets binary-size, maintenance, or quality constraints.
- [ ] 3.4 Add tests for singular and plural output in at least English and one non-English locale.

## 4. Locale Quality Guardrails

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### Definition of Done (DoD)

- [ ] Extend locale checks to cover current-file completeness, pseudo-translations, and selected formatter message keys.
- [ ] Confirm the stale `task_todo` finding from PR #236 is not reintroduced as a failing task unless current locale data regresses.
- [ ] Add CI or linter coverage for the locale checks.
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

- [ ] 4.1 Add current-locale checks for missing keys and pseudo-translation markers.
- [ ] 4.2 Add checks that selected formatted messages exist in every supported locale.
- [ ] 4.3 Document the superseded relationship to `x-x-x-advanced-i18n-framework`.

---

## 5. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [ ] 5.1 ユーザーへ実装完了の報告および動作状況（UIの場合はスナップショット画像等）の提示を行う
- [ ] 5.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）

---

## 6. Final Verification & Release Work

- [ ] 6.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [ ] 6.2 Format and lint-fix all updated markdown documents (e.g., tasks.md, CHANGELOG.md)
- [ ] 6.3 Ensure `make check` passes with exit code 0
- [ ] 6.4 Create PR from Base Feature Branch targeting `master`
- [ ] 6.5 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
- [ ] 6.6 Merge into master (`gh pr merge --merge --delete-branch`)
- [ ] 6.7 Create `release/v0.30.0` branch from master
- [ ] 6.8 Run `make release VERSION=0.30.0` and update CHANGELOG (`changelog-writing` skill)
- [ ] 6.9 Create PR from `release/v0.30.0` targeting `master` — Ensure `Release Readiness` CI passes
- [ ] 6.10 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 6.11 Verify GitHub Release completion and archive this change using `/opsx-archive`
