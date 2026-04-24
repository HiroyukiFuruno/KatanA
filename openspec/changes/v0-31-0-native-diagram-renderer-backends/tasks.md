## Branch Rule

本タスクでは、以下のブランチ運用を適用します：
- **標準（Base）ブランチ**: `v0-31-0-native-diagram-renderer-backends` またはリリース用統合ブランチ（例: `release/v0.31.0`）
- **作業ブランチ**: 標準は `v0-31-0-native-diagram-renderer-backends-task-x`、リリース用は `feature/v0.31.0-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

## 1. Backend Adapter Contract

### Definition of Done (DoD)

- [ ] Define KatanA-owned Mermaid and PlantUML backend adapter traits or equivalent service boundaries.
- [ ] Preserve the existing `DiagramResult` / preview fallback contract across all backends.
- [ ] Move direct `mmdc`, `java`, and `plantuml.jar` process calls behind backend implementations.
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

- [ ] 1.1 Define backend input types for source, theme snapshot, document context, and render options.
- [ ] 1.2 Implement external Mermaid CLI and external PlantUML jar backends through the adapter as the initial behavior-preserving migration.
- [ ] 1.3 Add tests proving preview and export code consume only adapter output.

## 2. Mermaid Rust Backend Spike

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### Definition of Done (DoD)

- [ ] Evaluate `merman`, `mermaid-rs-renderer`, and `selkie-rs` against KatanA Mermaid fixtures.
- [ ] Select the default candidate or record why Rust-native Mermaid remains opt-in.
- [ ] Implement the selected candidate behind the Mermaid backend adapter if it passes the spike gate.
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

- [ ] 2.1 Compare supported diagram kinds against KatanA's Mermaid usage and docs.
- [ ] 2.2 Render representative fixtures with each candidate and compare output dimensions, visible content, and error behavior.
- [ ] 2.3 Verify theme propagation and cache-key behavior for the selected candidate.
- [ ] 2.4 Keep `mmdc` fallback available if the Rust backend has unsupported syntax or visual regressions.

## 3. PlantUML Rust Backend Spike

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### Definition of Done (DoD)

- [ ] Evaluate `plantuml-little` against KatanA PlantUML fixtures, licensing, and cross-platform packaging constraints.
- [ ] Reject PlantUML server/client crates for default offline preview unless the product direction changes.
- [ ] Implement `plantuml-little` behind the PlantUML backend adapter if it passes the spike gate.
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

- [ ] 3.1 Verify `plantuml-little` builds on macOS, Windows, and Linux CI with KatanA's Rust toolchain.
- [ ] 3.2 Verify Graphviz-related packaging behavior and document any bundled native library requirements.
- [ ] 3.3 Compare sequence, class, state, component, and failure fixtures against the current jar backend.
- [ ] 3.4 Keep Java jar fallback available if the Rust backend has unsupported syntax or packaging gaps.

## 4. Default Backend Selection and Documentation

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### Definition of Done (DoD)

- [ ] Make Rust-native backends the default only for diagram kinds and platforms that passed parity and packaging gates.
- [ ] Update README and setup docs so Node.js, Mermaid CLI, Java, and PlantUML jar are described as fallback requirements when appropriate.
- [ ] Add diagnostics or settings visibility for the selected backend and fallback state.
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

- [ ] 4.1 Add backend selection policy and user-visible recovery guidance.
- [ ] 4.2 Update cross-platform validation docs for clean-install diagram rendering.
- [ ] 4.3 Remove or revise contradictory "no Node.js" copy where fallback requirements still remain.

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
- [ ] 6.7 Create `release/v0.31.0` branch from master
- [ ] 6.8 Run `make release VERSION=0.31.0` and update CHANGELOG (`changelog-writing` skill)
- [ ] 6.9 Create PR from `release/v0.31.0` targeting `master` — Ensure `Release Readiness` CI passes
- [ ] 6.10 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 6.11 Verify GitHub Release completion and archive this change using `/opsx-archive`
