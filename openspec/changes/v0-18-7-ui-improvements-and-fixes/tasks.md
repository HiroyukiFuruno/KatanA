## Branch Rule

本タスクでは、ユーザーの指定に基づき以下のブランチ運用を厳格に適用します：

- **統合（Base）ブランチ**: `release/v0.18.7`
- **各タスクの作業ブランチ**: `release/v0.18.7-task-x` (xはタスク番号)

各タスクの実装開始前に、`release/v0.18.7` から `release/v0.18.7-task-x` を作成して作業してください。
実装完了後は `/openspec-delivery` を使用して統合ブランチ（`release/v0.18.7`）へPRを作成・マージしてください。

## 1. Search Noise Reduction & Auto-link Fix

- [ ] 1.1 `katana-core/src/search/mod.rs` を修正し、`#[allow(...)]` 行をフィルタリングするロジックを実装
- [ ] 1.2 `katana-core/src/markdown/link_resolver.rs` (または該当箇所) を修正し、平文URLの自動リンク検出を改善
- [ ] 1.3 `katana-core` の関連テストを実行し、意図せぬデグレードがないか確認

### Definition of Done (DoD)

- [ ] 検索結果から `#[allow]` が適切に除外されることを確認
- [ ] 平文URLが正しくリンク化されることを確認
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine.

## 2. Meta Information UI Renewal

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 `katana-ui/src/views/modals/meta_info.rs` を刷新し、Finder風の整理されたレイアウトを実装
- [ ] 2.2 メタ情報の各項目（パス、サイズ、作成日時等）をセクション分けして表示
- [ ] 2.3 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 2.4 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] メタ情報ダイアログがFinder風の見た目になっていることを確認
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine.

## 3. Diagram Fullscreen & UI Polish

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 ダイアグラム全画面表示時のオーバーレイ背景を不透明化（アルファ値 1.0）
- [ ] 3.2 その他軽微な表示の乱れや透過設定の不整合を修正

### Definition of Done (DoD)

- [ ] 全画面表示で背景が透けず、図に集中できることを確認
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine.

## 4. Sidebar Continuity & Popup UI

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 4.1 `katana-ui/src/app/action/dispatch.rs` を修正し、他パネル展開時もエクスプローラーを表示維持する
- [ ] 4.2 サイドバーアイコンクリック時のアニメーション付きポップアップUIの実装（`Area` を使用）
- [ ] 4.3 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 4.4 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] エクスプローラーが常に表示または必要に応じてドロワーとして残ることを確認
- [ ] ポップアップがアイコンから生えてくるようなアニメーションで表示されることを確認
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine.

## 5. Tab Group Operations & Explorer Integration

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 5.1 タブ名入力時の `Return` キー/`Blur` での確定・クローズ処理を実装
- [ ] 5.2 エクスプローラーのコンテキストメニューに「タブグループを作成」「既存グループに追加」アクションを追加
- [ ] 5.3 5.2のアクションから該当ファイルをタブグループとして開くロジックを実装

### Definition of Done (DoD)

- [ ] タブのリネーム等がスムーズに確定されることを確認
- [ ] エクスプローラーからディレクトリ単位等でタブグループが作成できることを確認
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine.

## 6. Help Enrichment (Welcome & Guide)

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 6.1 「ようこそ」画面をタブ形式で開くように変更（初回起動時含む）
- [ ] 6.2 「操作ガイド」メニューを追加し、Markdownタブとして表示
- [ ] 6.3 `assets/docs/user_guide.md` 等の操作ガイドコンテンツを作成

### Definition of Done (DoD)

- [ ] ようこそ画面と操作ガイドがタブとして正しく表示されることを確認
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine.

---

## 7. Final Verification & Release Work

- [ ] 7.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [ ] 7.2 Ensure `make check` passes with exit code 0
- [ ] 7.3 Create PR from Base Feature Branch targeting `master`
- [ ] 7.4 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
- [ ] 7.5 Merge into master (`gh pr merge --merge --delete-branch`)
- [ ] 7.6 Create `release/v0.18.7` branch from master and update CHANGELOG (`changelog-writing` skill)
- [ ] 7.7 Create PR from `release/v0.18.7` targeting `master` — CI must pass
- [ ] 7.8 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 7.9 Execute `make release VERSION=0.18.7 FORCE=1` from `master`
- [ ] 7.10 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
