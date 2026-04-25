## 0. Definition of Ready (DoR)

- [ ] 本タスクは `v0.22.7` のリリースが完全に完了したのちに着手すること。
- [ ] 関連する UI コンポーネントおよび Diagnostics データ構造について、実装方針が開発環境上で検証可能であること。

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `v0-22-8-fix-preview`
- **作業ブランチ**: 標準は `v0-22-8-fix-preview-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

---

## 1. UI Rendering Extensions

- [ ] 1.1 `crates/katana-ui` の `diagnostics_renderer.rs` を改修し、`Diagnostic` アイテムの「修正」ボタン描画ロジックに Tooltip（ホバー表示）のサポートを追加する。
- [ ] 1.2 `DiagnosticFix` から提供される `replacement` 情報と元のコード（`start_line` 等から算出）を用いて、差分テキストを組み立てるロジックを実装する。
- [ ] 1.3 組み立てた差分テキストを Tooltip 内に描画する（文字色や打ち消し線を用いて Diff を表現する）。
- [ ] 1.4 長すぎる Diff が表示された場合を考慮し、Tooltip の最大幅・最大行数制限（省略表示等）を実装し、レイアウト崩れを防ぐ。

### Definition of Done (DoD)

- [ ] Problems パネル内の「修正」ボタンにホバーした際、元のコードと新しいコードの差分が Tooltip で視覚的に表示されること。
- [ ] Tooltip が画面の端で見切れたり、レイアウトを破壊したりしないこと。
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. User Review (Pre-Final Phase)

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [ ] 2.1 ユーザーへ実装完了の報告および動作状況（UIの場合はスナップショット画像等）の提示を行う
- [ ] 2.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）

### Definition of Done (DoD)

- [ ] ユーザーの確認が完了し、フィードバックの修正が Base ブランチにマージされていること。
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Final Verification & Release Work

- [ ] 3.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [ ] 3.2 Format and lint-fix all updated markdown documents (e.g., tasks.md, CHANGELOG.md)
- [ ] 3.3 Ensure `make check` passes with exit code 0
- [ ] 3.4 Create PR from Base Feature Branch targeting `master`
- [ ] 3.5 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
- [ ] 3.6 Merge into master (`gh pr merge --merge --delete-branch`)
- [ ] 3.7 Create `release/v0-22-8` branch from master
- [ ] 3.8 Run `make release VERSION=0-22-8` and update CHANGELOG (`changelog-writing` skill)
- [ ] 3.9 Create PR from `release/v0-22-8` targeting `master` — Ensure `Release Readiness` CI passes
- [ ] 3.10 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 3.11 Verify GitHub Release completion and archive this change using `/opsx-archive`
