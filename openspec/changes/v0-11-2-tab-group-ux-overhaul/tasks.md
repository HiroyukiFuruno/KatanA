# 実装タスク: タブグループ UX 全面改善 (v0.11.1)

## 着手条件 (DoR)

- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 現行の tab group 実装（`views/top_bar/ui.rs`、`app/action.rs`、`state/document.rs`）を確認していること

## ブランチ運用ルール

`##` ごとに grouped された task は、`/openspec-branching` workflow（`.agents/workflows/openspec-branching.md`）で定義された branching standard を無条件で守って実装すること。

---

## 1. 描画順序の刷新とグループヘッダーの視覚的差別化

### 着手条件 (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [ ] 1.1 `TabBar::show()` の描画ループを 3 フェーズ（グループブロック → ピン留めタブ → 通常タブ）に分割する
- [ ] 1.2 グループヘッダーをコンパクトチップデザインに変更する（色ドット ● + 縮小フォント名前）
- [ ] 1.3 グループ所属タブの下部に 2px カラーアンダーラインを描画する
- [ ] 1.4 折りたたみ時のグループヘッダーをドット + ▸ + メンバー数表示に変更する
- [ ] 1.5 既存テストの描画順序関連を更新し、新しい 3 フェーズ描画の regression test を追加する
- [ ] 1.6 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 1.7 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### 完了条件 (DoD)

- [ ] グループブロックがピン留めタブより左に描画されること
- [ ] グループヘッダーが通常タブと視覚的に明確に区別できること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、包括的な delivery 手順（Self-review、Commit、PR Creation、Merge）を完了する

---

## 2. グループ作成フローの刷新と色パレット

### 着手条件 (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [ ] 2.1 コンテキストメニューの「新しいグループを作成」をサブメニューパネル（名前入力欄 + 色パレット + 作成ボタン）に変更する
- [ ] 2.2 名前が空の場合は作成ボタンを無効化する
- [ ] 2.3 7 色固定パレット（Blue, Red, Green, Orange, Purple, Yellow, Teal）を横並び円形ボタンとして実装する
- [ ] 2.4 `CreateTabGroup` Action のデフォルト名・色のハードコードを除去し、ユーザー入力値を使用する
- [ ] 2.5 必要な i18n キーを追加する（`group_name_placeholder`, `create_group_button` 等）
- [ ] 2.6 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 2.7 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### 完了条件 (DoD)

- [ ] グループ作成時に名前入力と色選択が必須であること
- [ ] 名前なしではグループを作成できないこと
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、包括的な delivery 手順（Self-review、Commit、PR Creation、Merge）を完了する

---

## 3. インラインリネームとグループコンテキストメニュー拡充

### 着手条件 (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [ ] 3.1 グループヘッダーのダブルクリックでインラインリネームモードに入る実装（`layout.rs` の `inline_rename_group` state 使用）
- [ ] 3.2 Enter キーまたはフォーカス離脱でリネーム確定、空名前は元に戻す
- [ ] 3.3 シングルクリック（collapse toggle）とダブルクリック（rename）の競合回避
- [ ] 3.4 グループコンテキストメニューに「グループ解散」（Ungroup）を追加し、`UngroupTabGroup` Action を実装する
- [ ] 3.5 グループコンテキストメニューに「グループを閉じる」（Close Group）を追加し、`CloseTabGroup` Action を実装する
- [ ] 3.6 グループコンテキストメニューに色変更パレット（7色横並び）を追加する
- [ ] 3.7 コンテキストメニュー内の旧式 TextEdit リネーム UI を削除する
- [ ] 3.8 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 3.9 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### 完了条件 (DoD)

- [ ] ダブルクリックでインラインリネームが動作すること
- [ ] Ungroup / Close Group がコンテキストメニューから実行できること
- [ ] 色変更がコンテキストメニュー内パレットから即時反映されること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、包括的な delivery 手順（Self-review、Commit、PR Creation、Merge）を完了する

---

## 4. グループ単位のドラッグ＆ドロップ移動

### 着手条件 (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [ ] 4.1 グループヘッダーに `Sense::click_and_drag()` を設定し、ドラッグ操作を検出する
- [ ] 4.2 `ReorderTabGroup { from: usize, to: usize }` Action を `AppAction` に追加する
- [ ] 4.3 ドラッグ中のゴースト表示（グループヘッダーのみ）を実装する
- [ ] 4.4 ドロップ位置の判定とグループ順序の更新ロジックを実装する
- [ ] 4.5 ドラッグ＆ドロップの regression test を追加する
- [ ] 4.6 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 4.7 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### 完了条件 (DoD)

- [ ] グループヘッダーのドラッグでグループ順序を変更できること
- [ ] ドラッグ＆ドロップ後もセッション永続化が正常に動作すること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、包括的な delivery 手順（Self-review、Commit、PR Creation、Merge）を完了する

---

## 5. 最終確認とリリース作業

### 着手条件 (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [ ] 5.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` を使って self-review を実施する（各 file の version 更新漏れも確認する）
- [ ] 5.2 `make check` が exit code 0 で通過することを確認する
- [ ] 5.3 中間 base branch（もともと master から派生した branch）を `master` へ merge する
- [ ] 5.4 `master` 向け PR を作成する
- [ ] 5.5 `master` へ merge する（`--admin` 許可）
- [ ] 5.6 `.agents/skills/release_workflow/SKILL.md` を使って `0.11.1` の release tagging と release 作成を実施する
- [ ] 5.7 `/opsx-archive` など OpenSpec skill を使ってこの change を archive する

### 完了条件 (DoD)

- [ ] `master` branch に変更が統合されていること
- [ ] version 番号が正しく `v0.11.1` に更新されていること
- [ ] OpenSpec change が完全に archive されていること
