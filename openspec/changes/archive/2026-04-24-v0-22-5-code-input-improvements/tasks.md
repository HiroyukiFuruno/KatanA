## Definition of Ready (DoR)

- [ ] `proposal.md`、`design.md` がレビュー済みであること
- [ ] 対象バージョン 0.22.4 の変更 ID とスコープが確認されていること
- [ ] v0.22.3 のリリースが完了していること
- [ ] システム SVG アイコンの仕様を確認済みであること

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `v0-22-5-code-input-improvements`
- **作業ブランチ**: 標準は `v0-22-5-code-input-improvements-task-x` (x はタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへ PR を作成・マージしてください。

---

## 1. リッチテキストコントロール UI 実装

### 概要

マークダウンエディタに視覚的な書式設定ボタンを追加し、キーボードショートカットだけでなく GUI 操作も可能にする。

- [x] 1.1 システム SVG 基盤のボタンコンポーネントを `crates/katana-ui/src/views/panels/editor/toolbar.rs` に実装
- [x] 1.2 太字、斜体、見出し、リスト、コードなど一般的な Markdown 書式のボタンを追加
- [x] 1.3 エディタ上部にツールバーを表示（コンテキスト認識型、AlignCenter ウィジェットで水平配置）
- [x] 1.4 テキスト選択状態に応じてボタンの有効/無効を切り替え
- [x] 1.5 ショートカットキーとの統合（ボタンクリックとキー操作の両方で動作）

### Definition of Done (DoD)

- [x] ツールバーから Markdown 書式設定が可能であること
- [x] 選択テキストに応じてボタンが有効/無効に切り替わる
- [x] ショートカットキーと同等の動作を確認
- [x] `make check` がエラーなし (exit code 0) で通過すること
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. クリップボード画像貼り付け機能

### Definition of Ready (DoR)

- [ ] 1 つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR 作成、マージ、ブランチ削除）を完全に終えていること
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること

- [x] 2.1 クリップボードから画像データを読み取るロジックを `crates/katana-ui/src/state/clipboard/` に実装
- [x] 2.2 `command+shift+v` での貼り付けを Markdown エディタに追加
- [x] 2.3 右クリックメニューに「貼り付け」オプションを追加
- [x] 2.4 画像を `assets/img/` ディレクトリに自動的に保存する処理を実装
- [x] 2.5 画像ファイルの命名規則（タイムスタンプ付き）を実装
- [x] 2.6 挿入位置に相対パスの Markdown 画像記法を挿入

### Definition of Done (DoD)

- [x] command+shift+v でクリップボードの画像が貼り付けられること
- [x] 右クリックメニューから貼り付けが可能であること
- [x] 画像が設定した保存先に適切に保存されること
- [x] Markdown イメージ記法が正しく挿入されること
- [x] `make check` がエラーなしで通過すること
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Explorer での画像表示とドラッグ＆ドロップ

### Definition of Ready (DoR)

- [ ] 1 つ前のタスクがデリバリサイクルを完全に終えていること
- [ ] ベースブランチが最新化されており、新しいブランチが作成されていること

- [x] 3.1 Markdown ファイル内の画像参照を解析するロジックを実装
- [x] 3.2 Explorer に参照画像を表示（クリックで表示、ホバーでパス表示）
- [x] 3.3 アクティブドキュメント変更時に動的に更新（遅延読み込み不要の軽量実装）
- [x] 3.4 クリックで RevealImageAsset アクションを発火（ドラッグ&ドロップはスコープ外）
- [x] 3.5 CollapsingHeader でまとめて表示、デフォルト展開

### Definition of Done (DoD)

- [x] Explorer に参照画像が正しく表示されること
- [x] クリックで画像ファイルが表示されること
- [x] 大量の画像参照でもパフォーマンスが低下しないこと
- [x] `make check` がエラーなしで通過すること
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. UI 統合とフィードバック

### Definition of Ready (DoR)

- [ ] 1 つ前のタスクがデリバリサイクルを完全に終えていること
- [ ] ベースブランチが最新化されており、新しいブランチが作成されていること

- [x] 4.1 画像挿入操作にショートカットを設定（Cmd+Shift+V）
- [x] 4.2 設定画面 Behavior タブに画像保存先・命名形式・ディレクトリ自動作成の設定を追加
- [x] 4.3 実装内容の動作報告（ユーザーへの提示）
- [x] 4.4 フィードバックに基づく調整（builder パターン適用、linter ゲート通過）

### Definition of Done (DoD)

- [x] 画像挿入関連の操作が UI 上でシームレスに完結していること
- [x] 設定変更が即座に反映されること
- [x] `make check` がエラーなしで通過すること
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

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
