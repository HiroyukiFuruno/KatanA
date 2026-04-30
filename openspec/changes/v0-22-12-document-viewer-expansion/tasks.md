# Tasks: Document Viewer Integration (v0.22.12)

## DoR (Definition of Ready)
- [x] Proposal and Design are approved.
- [x] Target version v0.22.12 is set.

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：
- **標準（Base）ブランチ**: `v0-22-12-document-viewer-expansion`
- **作業ブランチ**: 標準は `v0-22-12-document-viewer-expansion-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

---

## 1. Document Viewer Framework & CSV Support

基礎となるドキュメント閲覧フレームワークと、最もシンプルな CSV レンダラーを実装する。

### Definition of Done (DoD)
- [ ] `katana-ui` に `document_viewer` モジュールを新設し、拡張子によるディスパッチロジックを実装。
- [ ] `egui_extras` を使用した CSV テーブルビューの実装。
- [ ] サイドバーからのファイルオープン連携。
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. PDF Preview Implementation

PDF ファイルのプレビュー機能を実装する。

### Definition of Ready (DoR)
- [ ] Ensure the previous task completed its full delivery cycle.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### Definition of Done (DoD)
- [ ] `pdfium-render` または WebView ベースの PDF 閲覧機能の実装。
- [ ] ページめくり、ズームイン/アウトのコントロール UI。
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine.

---

## 3. WebView Integration Spike & Web Support

WebView 統合の技術検証（スパイク）と、Office/Web URL 表示の実装を行う。

### Definition of Ready (DoR)
- [ ] Ensure the previous task completed its full delivery cycle.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### Definition of Done (DoD)
- [ ] **Spike**: `egui` と `wry` の共存、OS ごとのウィンドウハンドル取得、イベント伝播の技術検証完了。
- [ ] `wry` 等を用いた WebView コンポーネントの `egui` への統合。
- [ ] URL 入力フィールドおよびナビゲーション機能。
- [ ] オフライン時のエラーメッセージ表示の実装。
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine.

---

## 4. Local Preservation (Save Web Documents)

Web 上のドキュメントをローカルに保存する機能を実装する。

### Definition of Ready (DoR)
- [ ] Ensure the previous task completed its full delivery cycle.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

### Definition of Done (DoD)
- [ ] WebView 内のデータを取得し、ローカルファイルとして保存するロジックの実装。
- [ ] **Fallbacks**: 直接保存が制限されている場合の PDF エクスポートまたはブラウザ誘導の実装。
- [ ] 保存後のファイルがワークスペースに自動登録されること。
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine.

---

## 5. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする。

- [ ] 5.1 ユーザーへ実装完了の報告および動作状況を提示する。UI の動作確認は、`scripts/screenshot` のシナリオで生成した証跡を提示する。
- [ ] 5.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメントに追記し、すべて対応・解決する。

---

## KatanA CLI Entry Point

このリポジトリでは OpenSpec の実行入口として `./scripts/openspec` を使用すること。

## 6. Final Verification & Release Work

- [ ] 6.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [ ] 6.2 Format and lint-fix all updated markdown documents
- [ ] 6.3 通常の `git push` で `pre-push` hook を正式な品質ゲートとして通す
- [ ] 6.4 Create PR from Base Feature Branch targeting `master`
- [ ] 6.5 Confirm CI checks pass on the PR
- [ ] 6.6 Merge into master (`gh pr merge --merge --delete-branch`)
- [ ] 6.7 Create `release/v0.22.12` branch from master
- [ ] 6.8 Run `make release VERSION=0.22.12` and update CHANGELOG
- [ ] 6.9 Create PR from `release/v0.22.12` targeting `master`
- [ ] 6.10 Merge release PR into master
- [ ] 6.11 Verify GitHub Release completion and archive this change using `/opsx-archive`
