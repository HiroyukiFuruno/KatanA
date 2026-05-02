# Tasks: PDF出力ページングUX (v0.22.13)

## DoR (Definition of Ready)

- [x] Proposal and Design are created.
- [x] Target version v0.22.13 is set.
- [x] Scope is limited to Markdown PDF export pagination, page break selection, and export preview.

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：
- **標準（Base）ブランチ**: `v0-22-13-pdf-export-pagination-preview`
- **作業ブランチ**: 標準は `v0-22-13-pdf-export-pagination-preview-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

---

## 1. ページの選択系

PDF出力設定とExport導線を作り、PDF選択時に保存へ直行せず出力前プレビューへ進む状態にする。

- [ ] 1.1 `katana-core` に `PdfExportOptions` と `PdfPageMode` を追加し、既定値を `Paginated` にする
- [ ] 1.2 `SinglePage` は文書全体を1ページPDFとして保存し、`Paginated` は既存の複数ページ出力を使う
- [ ] 1.3 `katana-ui` のExportパネルでPDFを選んだとき、保存ダイアログではなく `Katana://PDFExportPreview/...` の仮想タブを開く
- [ ] 1.4 PDF出力前プレビューに `1ページ` / `複数ページ` の切り替え操作を追加する
- [ ] 1.5 `katana-core` の回帰テストで、`SinglePage` が1ページ、`Paginated` が長文で複数ページになることを確認する
- [ ] 1.6 `katana-ui` のテストで、ExportパネルのPDF選択が専用プレビュータブを開くことを確認する

### Definition of Done (DoD)

- [ ] PDF出力のページモードが保存処理へ渡る
- [ ] PDF選択時に保存ダイアログへ直行しない
- [ ] `1ページ` / `複数ページ` の切り替えでプレビュー状態が変わる
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. 複数ページ時のページ区切り候補選択

Markdown exportの中間表現にページ区切り候補を持たせ、ユーザー選択をPDFページ計算へ反映する。

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 Markdown export中間表現に、描画ブロックの高さ、候補ID、候補種別、表示ラベル、有効状態を追加する
- [ ] 2.2 見出し前、図の前、段落境界からページ区切り候補を生成する
- [ ] 2.3 ページ計算サービスを `katana-core` 側に集約し、候補の有効・無効を反映したページ境界を返す
- [ ] 2.4 ページ高さを超える図または段落を、候補とは別の強制分割として扱う
- [ ] 2.5 候補線のON/OFFがページ数と区切り位置に反映される回帰テストを追加する
- [ ] 2.6 強制分割が候補選択に依存しないことを回帰テストで確認する

### Definition of Done (DoD)

- [ ] ページ区切り候補が見出し前、図の前、段落境界から生成される
- [ ] 無効化された候補ではページを切らない
- [ ] 強制分割は候補とは別に扱われる
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. PDF出力前プレビュー

専用仮想タブ上で、実際のPDF出力と同じページ計算結果を確認・調整・保存できるようにする。

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 `Katana://PDFExportPreview/...` の仮想タブ表示を追加する
- [ ] 3.2 左側にページ一覧、中央にPDFページ見た目、設定領域にページモードと候補線リストを表示する
- [ ] 3.3 候補線のON/OFF操作で、ページ一覧と中央プレビューを同じページ計算結果から更新する
- [ ] 3.4 保存ボタンからファイル保存ダイアログを開き、選択中のページモードと候補設定でPDFを保存する
- [ ] 3.5 `katana-ui` のテストで、ページモード切り替えと候補線ON/OFFがプレビュー状態を更新することを確認する
- [ ] 3.6 `scripts/screenshot/examples/v0-22-13-pdf-export-preview.json` を追加し、ユーザーレビュー用のスクリーンショットまたは動画を生成できるようにする

### Definition of Done (DoD)

- [ ] PDF出力前プレビューと実際のPDF出力が同じページ計算結果を使う
- [ ] ページ一覧、中央プレビュー、候補線リスト、保存操作が同じ専用タブ上で完結する
- [ ] UI証跡を `scripts/screenshot` から生成できる
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [ ] 4.1 ユーザーへ実装完了の報告および動作状況を提示する。UI の動作確認は、ユーザーに手動操作を依頼せず、`scripts/screenshot` のシナリオで生成したスクリーンショットまたは動画を提示して確認できる状態にする
- [ ] 4.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）

---

## KatanA CLI Entry Point

このリポジトリでは OpenSpec の実行入口として `./scripts/openspec` を使用すること。グローバルの `openspec` コマンドが見つからない場合でも未導入と判断してはならない。このラッパーは `bunx @fission-ai/openspec`、次に `npx @fission-ai/openspec` へフォールバックする。

このスキル内で `openspec ...` と書かれているコマンドは、リポジトリルートから `./scripts/openspec ...` として実行する。

## 5. Final Verification & Release Work

- [ ] 5.1 Execute self-review using `docs/coding-rules.ja.md` and `$self-review` skill
- [ ] 5.2 Format and lint-fix all updated markdown documents (e.g., tasks.md, CHANGELOG.md)
- [ ] 5.3 通常の `git push` で `pre-push` hook を正式な品質ゲートとして通す。例外記録なしに、push 直前の重い `just check` / `just check-light` を二重実行しない
- [ ] 5.4 Create PR from Base Feature Branch targeting `master`
- [ ] 5.5 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
- [ ] 5.6 Merge into master (`gh pr merge --merge --delete-branch`)
- [ ] 5.7 Create `release/v0.22.13` branch from master
- [ ] 5.8 Run `just VERSION=0.22.13 release` and update CHANGELOG (`changelog-writing` skill)
- [ ] 5.9 Create PR from `release/v0.22.13` targeting `master` — Ensure `Release Readiness` CI passes
- [ ] 5.10 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 5.11 Verify GitHub Release completion and archive this change using `/opsx-archive`
