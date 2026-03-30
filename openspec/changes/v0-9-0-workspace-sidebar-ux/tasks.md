## 着手条件 (DoR)

- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.9.0 の変更 ID とスコープが確認されていること
- [ ] 現行のワークスペースヘッダー配置と左レール化対象（表示切替・検索・履歴）を `views/app_frame.rs` / `views/panels/workspace.rs` で再確認していること

## ブランチ運用ルール

`##` ごとに grouped された task は、`/openspec-branching` workflow（`.agents/workflows/openspec-branching.md`）で定義された branching standard を無条件で守って実装すること。

---

## 1. 左アクティビティレールの実装

### 着手条件 (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [ ] 1.1 `views/app_frame.rs` の `WorkspaceSidebar` を改修し、既存 `workspace_collapsed` 専用サイドパネルを常設レールへ置き換える
- [ ] 1.2 レールに workspace toggle / search / history を縦配置し、workspace pane 非表示時もレールが残るようにする
- [ ] 1.3 履歴ボタンを既存の `settings.workspace.paths` と `OpenWorkspace` / `RemoveWorkspace` に接続し、最近のワークスペース履歴メニューをレール側へ移す
- [ ] 1.4 履歴 0 件時は history ボタンを非活性表示で残し、レイアウトを崩さない挙動を実装する
- [ ] 1.5 左レールのアイコンを既存資産のまま一段大きく描画し、ホバー説明とアクティブ状態を整理する
- [ ] 1.5.1 `show_workspace` / `show_search_modal` 以外の新 layout state が必要と判明した場合は、実装継続前に `design.md` / `specs` / `tasks.md` を更新する
- [ ] 1.6 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 1.7 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### 完了条件 (DoD)

- [ ] 左レールからワークスペース表示切り替え・検索・履歴が利用できること
- [ ] ワークスペースペインを閉じても左レールが残り、同じ導線で再表示できること
- [ ] collapsed 専用サイドパネルが不要になり、導線が `WorkspaceSidebar` 側へ一元化されていること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、包括的な delivery 手順（Self-review、Commit、PR Creation、Merge）を完了する

---

## 2. ワークスペースヘッダーの再配置

### 着手条件 (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [ ] 2.1 `views/panels/workspace.rs` のヘッダー 1 行目から `Workspace` / `ワークスペース` 見出しと collapse ボタンを除去する
- [ ] 2.2 pane ヘッダーから search / history ボタンを除去し、refresh + filter を左グループ、expand all + collapse all を右グループへ再配置する
- [ ] 2.3 フィルタートグルと正規表現入力 UI をヘッダー内に維持し、既存の `filter_enabled` / `filter_query` / `filter_cache` の挙動が変わらないことを確認する
- [ ] 2.4 workspace 表示切り替え後も expanded directories・filter 状態・loading 表示が破綻しないよう整合を取る
- [ ] 2.5 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 2.6 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### 完了条件 (DoD)

- [ ] 見出し文言が消え、更新と全展開・全閉の配置が要求どおりになっていること
- [ ] フィルターの表示・入力・絞り込み結果が既存どおり動作すること
- [ ] pane ヘッダーの責務が「現 workspace 操作」に限定され、主要導線がレールへ集約されていること
- [ ] 実装対象 file (`app_frame.rs`, `workspace.rs`) と state (`show_workspace`, `show_search_modal`, `settings.workspace.paths`) が design と一致していること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、包括的な delivery 手順（Self-review、Commit、PR Creation、Merge）を完了する

---

## 3. 検索導線と回帰確認

### 着手条件 (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [ ] 3.1 左レールの検索ボタンから既存の検索モーダルを開けるようにし、ショートカット導線を維持する
- [ ] 3.2 レール化に伴う tooltip・i18n 文言・アクセシビリティラベルを整理する
- [ ] 3.3 history 0 件・workspace 未選択・workspace collapsed の 3 状態でレール挙動を確認する
- [ ] 3.4 UI テストまたはハーネスで、左レール起点の表示切り替え・検索・履歴操作の回帰を追加確認する
- [ ] 3.4.1 試作結果が spec の UX detail と合わない場合は、UI 微調整前に artifact を更新する
- [ ] 3.5 ユーザーへの UI スナップショット（画像等）の提示および動作報告
- [ ] 3.6 ユーザーからのフィードバックに基づく UI の微調整および改善実装

### 完了条件 (DoD)

- [ ] 検索はショートカットと左レールの両方から開けること
- [ ] 履歴メニューの開く・削除・ワークスペース再オープンが回帰していないこと
- [ ] history 0 件と no workspace 状態でもレールの配置と tooltip が破綻しないこと
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) を実行し、包括的な delivery 手順（Self-review、Commit、PR Creation、Merge）を完了する

---

## 4. 最終確認とリリース作業

### 着手条件 (DoR)

- [ ] 直前の task が self-review、recovery（必要時）、PR 作成、merge、branch 削除まで含めて完了している
- [ ] base branch が同期済みであり、この task 用の新しい branch が明示的に作成されている

- [ ] 4.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` を使って self-review を実施する（各 file の version 更新漏れも確認する）
- [ ] 4.2 `make check` が exit code 0 で通過することを確認する
- [ ] 4.3 中間 base branch（もともと master から派生した branch）を `master` へ merge する
- [ ] 4.4 `master` 向け PR を作成する
- [ ] 4.5 `master` へ merge する（`--admin` 許可）
- [ ] 4.6 `.agents/skills/release_workflow/SKILL.md` を使って `0.9.0` の release tagging と release 作成を実施する
- [ ] 4.7 `/opsx-archive` など OpenSpec skill を使ってこの change を archive する
