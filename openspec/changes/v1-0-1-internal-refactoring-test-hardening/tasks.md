策定日: 2026-04-25

## 着手条件（DoR）

- [x] `proposal.md`、`design.md`、`specs`、`tasks.md` が揃っていること
- [x] 対象バージョンが `v1.0.1` であり、change ディレクトリが `v1-0-1-internal-refactoring-test-hardening` であること
- [x] 策定日が 2026-04-25 であることが `proposal.md`、`design.md`、`tasks.md` に明記されていること
- [x] この計画は 2026-04-25 時点の解析結果であり、実装着手時に task0 で差分反映する前提が明記されていること
- [x] この change は v1.0.1 の最初の規格として、ユーザー向け新機能ではなく内部リファクタリングと回帰検知強化を扱うこと

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `v1-0-1-internal-refactoring-test-hardening` またはリリース用統合ブランチ（例: `release/v1.0.1`）
- **作業ブランチ**: 標準は `v1-0-1-internal-refactoring-test-hardening-task-x`、リリース用は `feature/v1.0.1-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

## 0. 着手前差分反映と計画最適化

### 着手条件（DoR）

- [x] Base ブランチが最新の `master` または `release/v1.0.1` に追従していること
- [x] 2026-04-25 から着手日までの差分を確認するための比較対象 commit が特定できていること

実施記録: 2026-04-25 14:53 JST に、比較基準 `512f6864` から `9ffeb570` までの `master` 差分、active OpenSpec、`.agents` / `.codex` workflow、Makefile、`scripts/runner`、`katana-ui` 構造、主要 test file 行数を確認した。

- [x] 0.1 2026-04-25 から着手日までの `master`、active OpenSpec、`.agents` ワークフロー、Makefile、テスト実行基盤の差分を確認する
- [x] 0.2 `katana-ui` のモジュール構造、大きいファイル、テスト配置を再計測し、2026-04-25 時点の解析との差分を整理する
- [x] 0.3 既に別 change で整理済みの領域、または新しく追加された領域を `design.md` の現状分析へ反映する
- [x] 0.4 task1 以降の対象順序、DoR、DoD、検証対象を最新状態に合わせて更新する

### 完了条件（DoD）

- [x] 2026-04-25 から着手日までの差分が `design.md` または `tasks.md` に記録されていること
- [x] task1 以降の作業順序が、着手日時点のコード構造と active change 状態に合っていること
- [x] この task では製品コードの移動や挙動変更を行っていないこと
- [x] `openspec validate v1-0-1-internal-refactoring-test-hardening` が exit code 0 で通過すること
- [x] 本セッションのユーザー指示に従い、この task0 の docs-only 差分を `master` へ commit / push すること

## 1. 構造棚卸しと分類

### 着手条件（DoR）

- [x] 前の task は本セッションのユーザー指示により、docs-only 差分として `master` へ commit / push 済みであること
- [x] Base ブランチが `master` と同期済みで、この task は製品コード変更を伴わない docs-only 先行作業として実施すること

- [x] 1.1 `katana-ui`、`katana-core`、`katana-platform` の大きいモジュール、責務が混在するモジュール、巨大なテストファイルを一覧化する
- [x] 1.2 単純なファイル移動で済む候補と、サービス境界 / 状態不変条件の再設計が必要な候補を分ける
- [x] 1.3 `AppAction`、`AppState`、shell dispatch、preview rendering、diagnostics、workspace、settings の現状責務を表にする
- [x] 1.4 master に入った i18n fallback、diagram backend contract、local LLM UI 前提整理を再実装対象から除外し、v1.0.1 で扱う対象と後続バージョンへ送る対象を明記する

### 完了条件（DoD）

- [x] 各対象モジュールについて「機械的な移動」「境界再設計」「後続送り」の分類が記録されていること
- [x] 分類ごとに、変更前に必要な契約テストまたは確認手順が定義されていること
- [x] v1.0.1 の範囲が、UI新機能を含まない内部整理として閉じていること
- [x] 本セッションのユーザー指示に従い、この task1 の docs-only 差分を `master` へ commit / push すること

## 2. ディレクトリとモジュール境界の再設計

### 着手条件（DoR）

- [ ] 前の task で、自己レビュー、必要時の復旧、PR作成、merge、ブランチ削除まで完了していること
- [ ] Base ブランチが同期済みで、この task 用の新しいブランチが明示的に作成されていること

- [ ] 2.1 `features/document`、`features/workspace`、`features/preview`、`features/diagnostics`、`features/settings` の目標構造を確定する
- [ ] 2.2 `shell`、`views`、`widgets`、`features` の import rule と ownership rule を定義する
- [ ] 2.3 ファイル移動だけで可能なモジュールを、挙動変更なしの小さい PR 単位に分割する
- [ ] 2.4 既存 `app/action/*` と `state/*` の所有境界を明文化してから移動し、移動後も public API、feature state、ユーザーから見える挙動が変わらないことを確認する

### 完了条件（DoD）

- [ ] 目標ディレクトリ構造と import rule が architecture note または `design.md` に記録されていること
- [ ] ファイル移動の commit と挙動変更の commit が混ざっていないこと
- [ ] 移動対象ごとに `make check` または対象単体テスト / 統合テストが通過していること
- [ ] `/openspec-delivery` ワークフロー（`.agents/workflows/openspec-delivery.md`）を実行し、自己レビュー、commit、PR作成、merge を含む delivery routine を完了すること

## 3. 内部実装境界の再設計

### 着手条件（DoR）

- [ ] 前の task で、自己レビュー、必要時の復旧、PR作成、merge、ブランチ削除まで完了していること
- [ ] Base ブランチが同期済みで、この task 用の新しいブランチが明示的に作成されていること

- [ ] 3.1 `AppAction` を document、workspace、layout、settings、preview、diagnostics、tabs などの領域 action へ分割する設計を作る
- [ ] 3.2 root dispatcher を領域 handler への routing に寄せ、巨大 match に無関係な action が集まらない構造にする
- [ ] 3.3 `AppState` の direct mutable access を減らし、feature state の query / command API を追加する
- [ ] 3.4 document mutation、workspace mutation、preview refresh、diagnostics refresh の不変条件を契約テストで固定する
- [ ] 3.5 view module が filesystem mutation や domain mutation を直接所有しないことを確認する

### 完了条件（DoD）

- [ ] root action と領域 action の責務が文書化され、追加先が判断できること
- [ ] `AppState` の主要 mutation が feature state API 経由になり、複数 field の手動同期が減っていること
- [ ] document / workspace / preview / diagnostics の不変条件を壊す変更がテストで検知できること
- [ ] view module は action 発行または表示に集中し、filesystem mutation を直接持たないこと
- [ ] `/openspec-delivery` ワークフロー（`.agents/workflows/openspec-delivery.md`）を実行し、自己レビュー、commit、PR作成、merge を含む delivery routine を完了すること

## 4. 単体テスト / 統合テストの再編成

### 着手条件（DoR）

- [ ] 前の task で、自己レビュー、必要時の復旧、PR作成、merge、ブランチ削除まで完了していること
- [ ] Base ブランチが同期済みで、この task 用の新しいブランチが明示的に作成されていること

- [ ] 4.1 `shell_tests.rs`、`shell_ui_tests.rs`、`preview_pane/tests.rs` を契約単位へ分割する計画を作る
- [ ] 4.2 pure logic / state transition は単体テストに寄せ、UI harness 依存を減らす
- [ ] 4.3 user workflow は統合テストに残し、state、semantic text、layout rect、action dispatch の assertion を強化する
- [ ] 4.4 既存 integration の `preview_pane/tables.rs`、`preview_pane/diagrams.rs` を release regression gate へ接続するか、通常 integration contract として残すかを分類する
- [ ] 4.5 過去 bug の regression test を fixture と test name で追跡できるようにする
- [ ] 4.6 fixed wait に依存している harness helper を特定し、条件待機へ移す

### 完了条件（DoD）

- [ ] 巨大 test file の分割後に、各 file が守る contract が file 名または module 名から判断できること
- [ ] unit test と integration test の境界が明確になり、UI harness が不要な test は unit test に移っていること
- [ ] 過去 bug の regression test が削除されず、再発検知の意図が残っていること
- [ ] fixed wait の新規追加がなく、既存 fixed wait は条件待機へ置き換えられていること
- [ ] `/openspec-delivery` ワークフロー（`.agents/workflows/openspec-delivery.md`）を実行し、自己レビュー、commit、PR作成、merge を含む delivery routine を完了すること

## 5. リリース回帰ゲート

### 着手条件（DoR）

- [ ] 前の task で、自己レビュー、必要時の復旧、PR作成、merge、ブランチ削除まで完了していること
- [ ] Base ブランチが同期済みで、この task 用の新しいブランチが明示的に作成されていること

- [ ] 5.1 v1.0.0 後に壊してはいけない core workflow を test manifest として定義する
- [ ] 5.2 document open / edit / save、workspace navigation、preview render、diagnostics、settings persistence、export の gate を整える
- [ ] 5.3 `make check` に含める範囲と、必要なら `make release-check` に分ける範囲を決める
- [ ] 5.4 リリース回帰ゲートの実行時間、失敗時の切り分け方法、CI での扱いを記録する

### 完了条件（DoD）

- [ ] リリース回帰ゲートの対象 workflow が manifest または tasks に明記されていること
- [ ] 正式リリース後に壊してはいけない core workflow が自動テストで検知できること
- [ ] ゲートの実行方法が Makefile または runner から再現できること
- [ ] CI で gate が失敗した場合に blocking とする条件が明記されていること
- [ ] `/openspec-delivery` ワークフロー（`.agents/workflows/openspec-delivery.md`）を実行し、自己レビュー、commit、PR作成、merge を含む delivery routine を完了すること

---

## 6. ユーザーレビュー（最終検証前）

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [ ] 6.1 ユーザーへ実装完了の報告および構造変更・test gate の差分を提示する
- [ ] 6.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）

---

## 7. 最終検証とリリース作業（Final Verification & Release Work）

- [ ] 7.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` に沿って自己レビューを実行する
- [ ] 7.2 更新した Markdown 文書（例: `tasks.md`、`CHANGELOG.md`）を format / lint-fix する
- [ ] 7.3 `make check` が exit code 0 で通過することを確認する
- [ ] 7.4 Base Feature Branch から `master` 向け PR を作成する
- [ ] 7.5 PR の CI（Lint / Coverage / CodeQL）が通過していることを確認し、失敗している場合は merge しない
- [ ] 7.6 `master` へ merge する（`gh pr merge --merge --delete-branch`）
- [ ] 7.7 `master` から `release/v1.0.1` ブランチを作成する
- [ ] 7.8 `make release VERSION=1.0.1` を実行し、`changelog-writing` skill で CHANGELOG を更新する
- [ ] 7.9 `release/v1.0.1` から `master` 向け PR を作成し、`Release Readiness` CI の通過を確認する
- [ ] 7.10 release PR を `master` へ merge する（`gh pr merge --merge --delete-branch`）
- [ ] 7.11 GitHub Release の完了を確認し、`/opsx-archive` でこの change を archive する
