## Definition of Ready (DoR)

- [ ] `proposal.md`、`design.md` がレビュー済みであること
- [ ] 対象バージョン 0.22.4 の変更 ID とスコープが確認されていること
- [ ] v0.22.3 のリリースが完了していること
- [ ] markdownlint の全ルール仕様を確認済みであること

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `v0-22-4-markdown-linter-improvements`
- **作業ブランチ**: 標準は `v0-22-4-markdown-linter-improvements-task-x` (x はタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへ PR を作成・マージしてください。

---

## 1. Markdownlint 全ルールサポート実装

### 概要

現在の MD001 のみサポートから、markdownlint の全公式ルールをサポートするように拡張する。

- [x] 1.0 `refresh_preview` のショートカットを `refresh_document` に統合
  - `os_commands.json` から `refresh_preview` キーを削除
  - `guide_ja.md` / `guide_en.md` の表記を `refresh_document` へ変更
  - `locales/*.json` (全言語) の `shortcut_refresh` 内の参照を `refresh_document` へ変更
- [x] 1.1 `crates/katana-linter/src/rules/markdown/` に markdownlint の全ルール実装を追加
  - `rules/` サブディレクトリを新設し、ルール実装ファイルを整理
  - `helpers.rs` に共有ユーティリティを `RuleHelpers` struct として集約
- [x] 1.2 各ルールの検証ロジックを実装（MD001-MD052 の全ルール）
  - MD003, MD004, MD011→MD012, MD022-MD023, MD025-MD029, MD032-MD033, MD035-MD036, MD040-MD042, MD045, MD047 を実装
  - AST linter 全項目（file-length, nesting-depth, magic-numbers, no-pub-free-fn）に完全準拠
  - `make check` が exit code 0 で通過
- [x] 1.3 ルールカテゴリ別に自動修正可能なルールと手動修正が必要なルールを分類
  - `OfficialRuleMeta.is_fixable` フィールドで各ルールの自動修正可否を管理
- [ ] 1.4 ルールの有効/無効設定をコマンドインベントリに追加
- [ ] 1.5 既存の MD001 ルーとの後方互換性を確認

### Definition of Done (DoD)

- [ ] markdownlint 公式の全ルールが動作すること
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] ルールの分類が正しく行われ、自動修正可能なルールが識別できること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. エディタ内視覚的インジケーター実装

### Definition of Ready (DoR)

- [ ] 1 つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR 作成、マージ、ブランチ削除）を完全に終えていること
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること

- [ ] 2.1 `crates/katana-ui/src/views/editor/` に lint エラーの視覚的インジケーターを実装
- [ ] 2.2 問題行に黄色のアンダーラインを描画（テーマカラー適用可能）
- [ ] 2.3 マウスホバーで問題の詳細を表示するツールチップを実装
- [ ] 2.4 lint 問題の位置情報（行番号、列番号）をエディタ状態に格納
- [ ] 2.5 複数問題が存在する場合の描画最適化（大量問題時のパフォーマンス確保）

### Definition of Done (DoD)

- [ ] lint 問題が黄色アンダーラインで視覚的に表示されること
- [ ] ホバー時のツールチップが正しく動作すること
- [ ] テーマ設定で警告色を変更できること
- [ ] 大量の lint 問題でも UI がカクつかないこと
- [ ] `make check` がエラーなしで通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. 診断ポップアップと自動修正機能

### Definition of Ready (DoR)

- [ ] 1 つ前のタスクがデリバリサイクルを完全に終えていること
- [ ] ベースブランチが最新化されており、新しいブランチが作成されていること

- [ ] 3.1 lint 問題上の右クリックメニューに「自動修正」オプションを追加
- [ ] 3.2 自動修正可能なルールに対して修正ボタンを表示
- [ ] 3.3 自動修正実行時のファイル変更管理（undo stack の管理）
- [ ] 3.4 診断ポップアップに問題の詳細情報（ルールの説明、修正例）を表示
- [ ] 3.5 一括修正機能（全文書・全ルールの自動修正）を実装

### Definition of Done (DoD)

- [ ] 右クリックメニューから自動修正が可能であること
- [ ] 自動修正が正しく適用され、undo 可能であること
- [ ] 診断ポップアップに詳細情報が表示されること
- [ ] 一括修正機能が動作すること
- [ ] `make check` がエラーなしで通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. UI 統合とフィードバック

### Definition of Ready (DoR)

- [ ] 1 つ前のタスクがデリバリサイクルを完全に終えていること
- [ ] ベースブランチが最新化されており、新しいブランチが作成されていること

- [ ] 4.1 設定画面に markdownlint ルール設定 UI を追加
- [ ] 4.2 ルール個別の有効/無効設定可能な UI を実装
- [ ] 4.3 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 4.4 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### Definition of Done (DoD)

- [ ] 設定画面からルール設定が可能であること
- [ ] 設定変更が即座に反映されること
- [ ] `make check` がエラーなしで通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Task 2

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 Task 2 description

### Definition of Done (DoD)

- [ ] (Other task-specific verifiable conditions...)
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Final Verification & Release Work

- [ ] 3.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [ ] 3.2 Ensure `make check` passes with exit code 0
- [ ] 3.3 Create PR from Base Feature Branch targeting `master`
- [ ] 3.4 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
- [ ] 3.5 Merge into master (`gh pr merge --merge --delete-branch`)
- [ ] 3.6 Create `release/v0.22.4` branch from master
- [ ] 3.7 Run `make release VERSION=0.22.4` and update CHANGELOG (`changelog-writing` skill)
- [ ] 3.8 Create PR from `release/v0.22.4` targeting `master` — Ensure `Release Readiness` CI passes
- [ ] 3.9 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 3.10 Verify GitHub Release completion and archive this change using `/opsx-archive`
