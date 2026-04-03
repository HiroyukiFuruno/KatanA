## 着手条件 (DoR)

- [x] `proposal.md`、`design.md`、`specs` が揃っていること
- [x] 対象バージョン 0.16.0 の変更 ID とスコープが確認されていること
- [x] 現行の Markdown preview 制御群、fullscreen viewer、theme 解決経路を `views/panels/preview.rs` / `preview_pane/*` / `markdown/*` で再確認していること

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. スライドショー起動導線

### Definition of Ready (DoR)

- [x] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [x] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [x] 1.1 `views/panels/preview.rs` の Markdown 制御群にスライドショー起動ボタンを追加する
- [x] 1.2 スライドショー起動時に active Markdown 文書が対象になることを確認する
- [x] 1.3 既存の preview 操作や fullscreen image 導線を壊さないことを確認する
- [x] 1.4 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [x] 1.5 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### Definition of Done (DoD)

- [x] Markdown 系の制御群からスライドショーを起動できること
- [x] 起動対象が active Markdown 文書であること
- [x] 既存 preview の基本操作が回帰していないこと
- [x] `make check` が exit code 0 で通過すること
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. 全画面 viewer とページング

### Definition of Ready (DoR)

- [x] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [x] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [x] 2.1 Markdown スライドショー用の全画面 viewer を実装する
- [x] 2.2 左右のページング操作で前後のページへ移動できるようにする
- [x] 2.3 先頭/末尾ページでの挙動が安定し、境界で破綻しないことを確認する
- [x] 2.4 全画面表示中のキーボード操作とフォーカス移動を整理する
- [x] 2.5 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [x] 2.6 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### Definition of Done (DoD)

- [x] スライドショーは全画面で表示されること
- [x] 左右のページングで前後に移動できること
- [x] 先頭/末尾で境界挙動が安定していること
- [x] `make check` が exit code 0 で通過すること
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. 終了導線・theme・ページ分割

### Definition of Ready (DoR)

- [x] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [x] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [x] 3.1 `Esc` と右上 `[x]` でスライドショーを終了できるようにする
- [x] 3.2 現在の theme を継承し、preview と同じ色解決経路を使うようにする
- [x] 3.3 diagram を含む Markdown 文書でも印刷時と同様の自動ページ分割を使うようにする
- [x] 3.4 theme 切り替え後の再表示でも配色が維持されることを確認する
- [x] 3.5 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [x] 3.6 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### Definition of Done (DoD)

- [x] `Esc` と `[x]` の両方で終了できること
- [x] theme が preview から引き継がれていること
- [x] diagram の切れ目が自動ページ分割に従うこと
- [x] `make check` が exit code 0 で通過すること
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. 最終確認とリリース作業

### Definition of Ready (DoR)

- [x] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [x] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [x] 4.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` を使って self-review を実施する（各 file の version 更新漏れも確認する）
- [x] 4.2 `make check` が exit code 0 で通過することを確認する
- [x] 4.3 中間 base branch（もともと master から派生した branch）を `master` へ merge する
- [x] 4.4 `master` 向け PR を作成する
- [x] 4.5 `master` へ merge する（`--admin` 許可）
- [x] 4.6 `.agents/skills/release_workflow/SKILL.md` を使って `0.17.0` の release tagging と release 作成を実施する
- [x] 4.7 `/opsx-archive` など OpenSpec skill を使ってこの change を archive する
