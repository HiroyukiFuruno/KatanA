# Tasks: UIアーキテクチャの真のコンポーネント化とテスト完全網羅

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. UIモジュールの設計見直しと「真の」コンポーネント分離

- [ ] 1.1 `views/panels/workspace/` の再実装（`ui.rs` と `logic.rs` への機能分解と再結合）
- [ ] 1.2 `views/panels/editor.rs` (354行) の分解（`editor_ui.rs` と `editor_logic.rs` 等）
- [ ] 1.3 `views/top_bar.rs` (689行) の分解（UIとルーター/イベント処理ロジックの分離）
- [ ] 1.4 `shell_ui.rs` から切り出したその他の表層分割モジュール群に対する関心事の分離（UI/Logic分割）の徹底
- [ ] 1.5 新たな UI コンポーネントツリーに対する Integration Test 導線の張り直しと検証
- [ ] 1.6 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 1.7 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)
- [ ] UIコンポーネント化とテスト導線の再構築が完了していること。
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. AST LinterのUIレイヤー完全適用

### Definition of Ready (DoR)
- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 `crates/katana-linter/tests/ast_linter.rs` の target に `katana-ui/src` を正式に追加する
- [ ] 2.2 対象違反（`file_length`, `function_length`, `nesting_depth`, `error_first`, `pub_free_fn`）を完全にゼロ・または正当な例外指定として解消する
- [ ] 2.3 `pub_free_fn` の統合テストにかけられていた `#[ignore]` を外し、UIを含めた全域のコーディングルールとして有効化する

### Definition of Done (DoD)
- [ ] Linterの違反が解消されテスト対象に組み込まれていること。
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. COVERAGE_IGNORE 解除と「未検証ロジックゼロ」の達成

### Definition of Ready (DoR)
- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 `views/`, `app/`, `state/` 等のディレクトリに配置された「純粋なコアロジックファイル（egui非依存）」の `COVERAGE_IGNORE` を解除する
- [ ] 3.2 除外が解除されたロジックファイルに対し、完全な単体・結合テスト（UT/IT）を実装し分岐網羅率100%を達成する
- [ ] 3.3 `make check` にて実行行カバレッジ100%（Uncovered Lines = 0）のゲートを再突破し、恒久化する

### Definition of Done (DoD)
- [ ] 全コアロジックファイルのカバレッジが100%に達していること。
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. コーディングルール適用・ドキュメント更新

### Definition of Ready (DoR)
- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 4.1 本フェーズでの発見と対応方針から得られた UI アーキテクチャガイドラインをドキュメントに追加する
- [ ] 4.2 `emoji.rs` の絵文字マッピングデータを外部データファイル（JSON等）に移行（v0.8.4からの移行）
- [ ] 4.3 ast_linterの除外リスト管理方法の確立（必要に応じて）（v0.8.4からの移行）

### Definition of Done (DoD)
- [ ] ドキュメント更新と移行タスクが完了していること。
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 5. Final Verification & Release Work

- [ ] 5.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 5.2 Ensure `make check` passes with exit code 0
- [ ] 5.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 5.4 Create a PR targeting `master`
- [ ] 5.5 Merge into master (※ `--admin` is permitted)
- [ ] 5.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.8.5`
- [ ] 5.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
