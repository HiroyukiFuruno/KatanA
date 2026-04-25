## Branch Rule

本タスクでは、以下のブランチ運用を適用します：
- **標準（Base）ブランチ**: `v0-27-0-decouple-editor` またはリリース用統合ブランチ（例: `release/v0.27.0`）
- **作業ブランチ**: 標準は `v0-27-0-decouple-editor-task-x`、リリース用は `feature/v0.27.0-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

## 1. Setup New Crate

### Definition of Done (DoD)
- [ ] Create `crates/katana-editor` with basic lib.rs
- [ ] Add crate to workspace members in root `Cargo.toml`
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

- [ ] 1.1 Create `crates/katana-editor` module structure
- [ ] 1.2 Add crate to workspace `Cargo.toml`

## 2. Implement Editor Component

### Definition of Ready (DoR)
- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### Definition of Done (DoD)
- [ ] Move editor state and widget logic to `katana-editor`
- [ ] Expose public API for `katana-ui`
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

- [ ] 2.1 Design and define `EditorConfig` (for injected settings, specifically fonts/emojis fallback configurations) and communication interfaces, while keeping egui `TextEdit` replacement out of scope for `x-x-x-native-input-surface`
- [ ] 2.2 Migrate editor state structures from `katana-ui` to `katana-editor`
- [ ] 2.3 Migrate text editing logic and keybindings
- [ ] 2.4 Implement and expose `EditorWidget` as the editor component boundary; do not make this task responsible for the future native input surface

## 3. Refactor katana-ui

### Definition of Ready (DoR)
- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### Definition of Done (DoD)
- [ ] Integrate new editor component into `katana-ui`
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

- [ ] 3.1 Refactor `katana-ui` to use `EditorWidget`
- [ ] 3.2 Inject KatanA settings via `EditorConfig`

---

## 4. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [ ] 4.1 ユーザーへ実装完了の報告および動作状況（UIの場合はスナップショット画像等）の提示を行う
- [ ] 4.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）

---

## 5. Final Verification & Release Work

- [ ] 5.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [ ] 5.2 Format and lint-fix all updated markdown documents (e.g., tasks.md, CHANGELOG.md)
- [ ] 5.3 Ensure `make check` passes with exit code 0
- [ ] 5.4 Create PR from Base Feature Branch targeting `master`
- [ ] 5.5 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
- [ ] 5.6 Merge into master (`gh pr merge --merge --delete-branch`)
- [ ] 5.7 Create `release/v0.27.0` branch from master
- [ ] 5.8 Run `make release VERSION=0.27.0` and update CHANGELOG (`changelog-writing` skill)
- [ ] 5.9 Create PR from `release/v0.27.0` targeting `master` — Ensure `Release Readiness` CI passes
- [ ] 5.10 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 5.11 Verify GitHub Release completion and archive this change using `/opsx-archive`
