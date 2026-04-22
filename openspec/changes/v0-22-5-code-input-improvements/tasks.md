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

- [ ] 1.1 システム SVG 基盤のボタンコンポーネントを `crates/katana-ui/src/components/buttons/` に実装
- [ ] 1.2 太字、斜体、見出し、リスト、コードなど一般的な Markdown 書式のボタンを追加
- [ ] 1.3 エディタ上部にツールバーを表示（コンテキスト認識型）
- [ ] 1.4 テキスト選択状態に応じてボタンの有効/無効を切り替え
- [ ] 1.5 ショートカットキーとの統合（ボタンクリックとキー操作の両方で動作）

### Definition of Done (DoD)

- [ ] トールバーから Markdown 書式設定が可能であること
- [ ] 選択テキストに応じてボタンが有効/無効に切り替わる 것
- [ ] ショートカットキーと同等の動作を確認
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. クリップボード画像貼り付け機能

### Definition of Ready (DoR)

- [ ] 1 つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR 作成、マージ、ブランチ削除）を完全に終えていること
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること

- [ ] 2.1 クリップボードから画像データを読み取るロジックを `crates/katana-ui/src/state/clipboard/` に実装
- [ ] 2.2 `command+v` での貼り付けを Markdown エディタに追加
- [ ] 2.3 右クリックメニューに「貼り付け」オプションを追加
- [ ] 2.4 画像を `assets/img/` ディレクトリに自動的に保存する処理を実装
- [ ] 2.5 画像ファイルの命名規則（タイムスタンプ付き）を実装
- [ ] 2.6 挿入位置に相対パスの Markdown 画像記法を挿入

### Definition of Done (DoD)

- [ ] command+v でクリップボードの画像が貼り付けられること
- [ ] 右クリックメニューから貼り付けが可能であること
- [ ] 画像が `assets/img/` に適切に保存されること
- [ ] Markdown イメージ記法が正しく挿入されること
- [ ] `make check` がエラーなしで通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Explorer での画像表示とドラッグ＆ドロップ

### Definition of Ready (DoR)

- [ ] 1 つ前のタスクがデリバリサイクルを完全に終えていること
- [ ] ベースブランチが最新化されており、新しいブランチが作成されていること

- [ ] 3.1 Markdown ファイル内の画像参照を解析するロジックを実装
- [ ] 3.2 Explorer に参照画像を表示（アイコン付き）
- [ ] 3.3 パフォーマンス維持のため遅延読み込みを実装
- [ ] 3.4 エディタへの画像ドラッグ＆ドロップをサポート
- [ ] 3.5 ドロップ位置に画像記法を挿入（選択なしの場合は末尾）

### Definition of Done (DoD)

- [ ] Explorer に参照画像が正しく表示されること
- [ ] ドラッグ＆ドロップで画像が挿入されること
- [ ] 大量の画像参照でもパフォーマンスが低下しないこと
- [ ] `make check` がエラーなしで通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. UI 統合とフィードバック

### Definition of Ready (DoR)

- [ ] 1 つ前のタスクがデリバリサイクルを完全に終えていること
- [ ] ベースブランチが最新化されており、新しいブランチが作成されていること

- [ ] 4.1 画像挿入操作にショートカットを設定
- [ ] 4.2 設定画面に画像保存先の設定を追加

### Definition of Done (DoD)

- [ ] 画像挿入関連の操作が UI 上でシームレスに完結していること
- [ ] 設定変更が即座に反映されること
- [ ] `make check` がエラーなしで通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 5. User Review (Pre-Final Phase)

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
- [ ] 6.7 Create `release/v0.22.4` branch from master
- [ ] 6.8 Run `make release VERSION=0.22.4` and update CHANGELOG (`changelog-writing` skill)
- [ ] 6.9 Create PR from `release/v0.22.4` targeting `master` — Ensure `Release Readiness` CI passes
- [ ] 6.10 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 6.11 Verify GitHub Release completion and archive this change using `/opsx-archive`
