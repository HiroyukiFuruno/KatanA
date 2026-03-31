## ブランチ運用ルール

`##` ごとにグループ化されたタスクは、実装セッション全体を通して `/openspec-branching` ワークフロー（`.agents/workflows/openspec-branching.md`）で定義されたブランチ標準へ無条件で従うこと。

---

## 1. 共通更新エントリーポイント

### 着手条件 (DoR)

- [x] 前の task が self-review、必要に応じた recovery、PR 作成、merge、branch 削除まで含む完全なデリバリーサイクルを完了していること。
- [x] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されていること。

- [x] 1.1 `RefreshDiagrams` の既存の呼び出し箇所（theme change / asset reload / preview refresh UI）を棚卸しし、内部再描画とユーザー起点更新 / 自動更新の責務境界を確定する
- [x] 1.2 共有更新 action を shell 共通 chrome に追加し、CodeOnly / PreviewOnly / Split の全 view mode で同一導線から実行できるようにする
- [x] 1.3 preview pane 専用 refresh ボタンを撤去し、preview 側には export / ToC など preview 固有操作だけを残す
- [x] 1.4 更新成功 / dirty スキップ / hash 不変 / 再読込失敗の status / i18n 契約を追加する
- [x] 1.5 自動更新の既定値の提案理由をユーザーへ提示し、`auto_refresh_interval_secs` の合意を取得する
- [x] 1.6 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [x] 1.7 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### 完了条件 (DoD)

- [x] 共有更新の正式導線が 1 つだけになり、Code / Preview / Split で同じ挙動になる
- [x] 内部再描画経路は disk 再読込を伴わないまま維持される
- [x] auto-refresh の既定値はユーザー合意済みである
- [x] `/openspec-delivery` ワークフロー（`.agents/workflows/openspec-delivery.md`）を実行し、Self-review、Commit、PR 作成、Merge を含む包括的なデリバリー手順を完了する。

## 2. Hash 管理された更新と設定

### 着手条件 (DoR)

- [x] 前の task が self-review、必要に応じた recovery、PR 作成、merge、branch 削除まで含む完全なデリバリーサイクルを完了していること。
- [x] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されていること。
- [x] 外部エディタで更新された clean 文書は共有更新または auto-refresh で取り込める
- [x] hash 差分がなければ手動 / 自動更新のどちらでも不要な再読込は起きない
- [x] dirty 文書は手動 / 自動更新でも黙って上書きされない
- [x] 同一 external hash に対する dirty warning は 1 回だけ表示される
- [x] auto-refresh の設定値は保存・復元される
- [ ] `/openspec-delivery` ワークフロー（`.agents/workflows/openspec-delivery.md`）を実行し、Self-review、Commit、PR 作成、Merge を含む包括的なデリバリー手順を完了する。

## 3. ネストされた task list の描画とUIクリック判定の統合

### 着手条件 (DoR)

- [x] 前の task が self-review、必要に応じた recovery、PR 作成、merge、branch 削除まで含む完全なデリバリーサイクルを完了していること。
- [x] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されていること。

- [x] 3.1 ユーザー要望の対応: リストマーカーと文字列のUI要素が分離解釈されている問題（HTMLバブリング相当の挙動欠如）の解消。タスク行全体の`rect`を統合し、テキストクリックでも状態をトグル可能にする
- [x] 3.2 UI改善: チェックボックスとテキスト間のマージンギャップ（8.0px）をクリッカブルエリアに統合し、ユーザーのクリック体験を向上する
- [x] 3.3 `vendor/egui_commonmark` の delayed / wrapped event 処理で元の event index を保持し、nested parsing でも task list 判定が失われないようにする
- [x] 3.4 task list 親行では checkbox だけを先頭マーカーとして表示し、余計な bullet を出さないようにする
- [x] 3.5 native task list（`[x]`, `[ ]`）と custom state（`[/]`, `[-]`, `[~]`）の両方に対する parser / preview 回帰テストを追加する
- [x] 3.6 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [x] 3.7 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### 完了条件 (DoD)

- [x] nested task list の親行から二重マーカーが消え、子リストの表現は回帰していない
- [x] タスク行全体が統一されたクリッカブル要素として挙動し、分離した判定の違和感が解消されている
- [x] parser 層と KatanA preview 層の両方で回帰が検出できる
- [ ] `/openspec-delivery` ワークフロー（`.agents/workflows/openspec-delivery.md`）を実行し、Self-review、Commit、PR 作成、Merge を含む包括的なデリバリー手順を完了する。

## 4. エンドツーエンド検証

### 着手条件 (DoR)

- [x] 前の task が self-review、必要に応じた recovery、PR 作成、merge、branch 削除まで含む完全なデリバリーサイクルを完了していること。
- [x] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されていること。

- [x] 4.1 外部エディタで Markdown を変更し、CodeOnly / PreviewOnly / Split の各モードから共有更新で反映できることを検証する
- [x] 4.2 auto-refresh interval 経過後に clean 文書は自動反映され、dirty 文書は warning のみで保護されることを確認する
- [x] 4.3 hash 不変時には手動 / 自動更新のどちらでも何もしないことを確認する
- [x] 4.4 共有更新実行時に図・画像キャッシュが適切に再描画され、theme change 等の内部再描画経路は従来どおり再描画のみで動くことを確認する
- [x] 4.5 dirty 文書で同一 external hash を維持したまま複数 polling interval が経過しても warning が重複しないことを確認する
- [x] 4.6 `katana-ui` と vendored parser と settings の対象テストを実行し、nested task list と refresh contract の回帰がないことを確認する

### 完了条件 (DoD)

- [x] ユーザー操作・自動更新・内部自動再描画・nested task list 表示の各経路が spec どおりに整合している
- [x] `/openspec-delivery` ワークフロー（`.agents/workflows/openspec-delivery.md`）を実行し、Self-review、Commit、PR 作成、Merge を含む包括的なデリバリー手順を完了する。

---

## 5. 最終検証とリリース作業

### 着手条件 (DoR)

- [x] 前の task が self-review、必要に応じた recovery、PR 作成、merge、branch 削除まで含む完全なデリバリーサイクルを完了していること。
- [x] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されていること。

- [x] 5.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` を用いて self-review を実施する（各ファイルで version 更新漏れがないか確認する）
- [x] 5.2 `make check` が exit code 0 で通ることを確認する
- [x] 5.3 中間 base branch（もともと master から派生した branch）を `master` branch に merge する
- [x] 5.4 `master` を向いた PR を作成する
- [x] 5.5 master へ merge する（※ `--admin` 使用可）
- [x] 5.6 `.agents/skills/release_workflow/SKILL.md` を用いて `0.8.6` 向けの release tag 作成と release 作成を実施する
- [x] 5.7 `/opsx-archive` などの OpenSpec skill を活用して、この change を archive する
