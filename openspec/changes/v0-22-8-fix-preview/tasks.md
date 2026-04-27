## 0. Definition of Ready (DoR)

- [x] 本タスクは `v0.22.7` のリリースが完全に完了したのちに着手すること。
- [x] 関連する UI コンポーネントおよび Diagnostics データ構造について、実装方針が開発環境上で検証可能であること。

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `v0-22-8-fix-preview`
- **作業ブランチ**: 標準は `v0-22-8-fix-preview-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

---

## User Feedback

- [/] ファイル単位の修正では、手動修正・LLM自動修正のどちらでも、適用前に git diff 風の差分確認画面を表示する。
- [/] 複数ファイルやワークスペース全体の一括修正では、ページ送りのようにファイルごとの差分を順に表示し、反映または拒否を選べるようにする。
- [/] 差分表示は Split と Inline の2種類を提供し、既定値は Split、設定で永続変更、差分画面内で一時切り替えできるようにする。
- [/] KML 側へ切り出す場合は、KatanA 専用ではない汎用の Fix 適用 API に限定し、差分画面・承認・設定は KatanA 側で持つ（KML issue #43 作成済み）。
- [/] 差分画面は既存の分割表示に近い「コードとコード」のモダンなビューアにし、モーダル専用ではなくタブや LLM チャットにも再利用できる設計にする。
- [/] 差分ビューアには左右の行番号、ファイルごとの追加/削除行数、未変更行の折りたたみ表示を入れる。
- [/] UI/UX が劣化しない操作はアイコンボタン化し、KatanA らしい見た目に戻す。

---

## 1. Supporting Hover Preview

- [x] 1.1 `crates/katana-ui` の `diagnostics_renderer.rs` を改修し、`Diagnostic` アイテムの「修正」ボタン描画ロジックに Tooltip（ホバー表示）のサポートを追加する。
- [x] 1.2 `DiagnosticFix` から提供される `replacement` 情報と元のコード（`start_line` 等から算出）を用いて、差分テキストを組み立てるロジックを実装する。
- [x] 1.3 組み立てた差分テキストを Tooltip 内に描画する（文字色や打ち消し線を用いて Diff を表現する）。
- [x] 1.4 長すぎる Diff が表示された場合を考慮し、Tooltip の最大幅・最大行数制限（省略表示等）を実装し、レイアウト崩れを防ぐ。

### Definition of Done (DoD)

- [x] Problems パネル内の「修正」ボタンにホバーした際、元のコードと新しいコードの差分が Tooltip で視覚的に表示されること。
- [x] Tooltip が画面の端で見切れたり、レイアウトを破壊したりしないこと。
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. File-Level Diff Review (Main Scope)

- [x] 2.1 修正適用前の本文と適用後の本文から、git diff 風の表示モデルを構築する。
- [x] 2.2 `ApplyLintFixesForFiles` を即時適用せず、差分確認用の保留状態へ変換する。
- [x] 2.3 ファイル単位の差分確認モーダルを実装し、右下に「キャンセル」「修正を反映」を配置する。
- [x] 2.4 ユーザーが「修正を反映」を選んだ場合のみ、対象ファイルへ変更を反映する。
- [x] 2.5 ユーザーが「キャンセル」を選んだ場合、対象ファイルを変更しない。
- [x] 2.6 単体 DiagnosticFix の「修正」クリックも、正式な適用前確認としてファイル差分確認フローへ接続する。
- [x] 2.7 差分確認モデルと適用可否の回帰テストを追加する。

### Definition of Done (DoD)

- [x] ファイル単位の修正適用前に、差分確認画面が必ず表示されること。
- [x] 差分確認画面でキャンセルした場合、本文が変更されないこと。
- [x] 差分確認画面で反映した場合、本文が期待通り変更されること。

---

## 3. Multi-File Diff Review

- [x] 3.1 複数ファイルの修正候補を、ファイルごとの差分ページとして保持する。
- [x] 3.2 差分確認画面に現在のファイル番号と総数を表示する。
- [x] 3.3 「前へ」「次へ」でファイル差分を移動できるようにする。
- [x] 3.4 ファイルごとに反映または拒否を選べるようにする。
- [x] 3.5 ワークスペース全体修正と Problems ビューの複数ファイル一括修正を同じフローへ接続する。
- [x] 3.6 複数ファイルの一部反映・一部拒否の回帰テストを追加する。

### Definition of Done (DoD)

- [x] 複数ファイルの一括修正で、各ファイルの差分を順番に確認できること。
- [x] 反映したファイルだけが変更され、拒否したファイルは変更されないこと。

---

## 4. Diff Display Modes and Settings

- [x] 4.1 Split（左右分割）表示を実装し、初期表示にする。
- [x] 4.2 Inline（行内）表示を実装する。
- [x] 4.3 差分画面上に Split / Inline の一時切り替えボタンを追加する。
- [x] 4.4 差分表示方式の永続設定を追加する。
- [x] 4.5 設定画面で差分表示方式を変更できるようにする。
- [x] 4.6 既定値が Split であること、設定変更が永続化されること、一時切り替えが永続設定を書き換えないことをテストする。

### Definition of Done (DoD)

- [x] 差分確認画面の既定表示が Split であること。
- [x] 設定で既定表示を Inline に変更でき、再起動後も維持されること。
- [x] 差分画面内の一時切り替えが設定値を変更しないこと。

---

## 5. User Review (Pre-Final Phase)

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

## 6. Final Verification & Release Work

- [ ] 6.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [ ] 6.2 Format and lint-fix all updated markdown documents (e.g., tasks.md, CHANGELOG.md)
- [ ] 6.3 Ensure `make check` passes with exit code 0
- [ ] 6.4 Create PR from Base Feature Branch targeting `master`
- [ ] 6.5 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
- [ ] 6.6 Merge into master (`gh pr merge --merge --delete-branch`)
- [ ] 6.7 Create `release/v0-22-8` branch from master
- [ ] 6.8 Run `make release VERSION=0-22-8` and update CHANGELOG (`changelog-writing` skill)
- [ ] 6.9 Create PR from `release/v0-22-8` targeting `master` — Ensure `Release Readiness` CI passes
- [ ] 6.10 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 6.11 Verify GitHub Release completion and archive this change using `/opsx-archive`
