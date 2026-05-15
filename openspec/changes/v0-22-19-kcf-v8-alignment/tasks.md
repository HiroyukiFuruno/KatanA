# Tasks: v0.22.19 kcf V8 alignment - KatanA

## 要件(Verbatim Requirements)

> kcf側対応済みでv0.1.7でリリース済みです。
>
> あたたにはkatana側でissueを次期patch versionとしてopenspecのchange作成をお願いしたいです！
>
> 現在katanaはv0.22.18リリース済みなので、v0.22.19が対象になるかなって思いますね。

> documentオンリーなのでmasterで作業でOKです。

### この要件から導出される制約(MUST)

- kcf 側は v0.1.7 で対応済みとして扱う。
- KatanA 側は次期パッチ版（patch version）の v0.22.19 対象として OpenSpec change を作成する。
- 本 OpenSpec 文書作成は文書のみ（document-only）であり、`master` 上で作業してよい。

### 設計判断時の参照義務

- 設計方針を提案・変更する際は、本セクションを直接参照し、各 MUST 制約を満たすことを設計説明の冒頭で確認する。
- 設計変更が verbatim と矛盾する場合、設計を変更するのではなくユーザーへ要件の更新を確認する。

## Definition of Ready (DoR)

- [x] KatanA issue [#293](https://github.com/HiroyukiFuruno/KatanA/issues/293) が作成されていること
- [x] kcf v0.1.7 が公開され、`v8 = "=147.4.0"` へ追従済みであること
- [x] KatanA v0.22.18 が既存リリース済みで、次期パッチ対象（patch target）が v0.22.19 であること
- [x] KatanA 側の OpenSpec 作成作業は文書のみ（document-only）として `master` 上で行うこと

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **OpenSpec 文書作成ブランチ**: `master`
- **実装時の標準（Base）ブランチ**: `release/v0.22.19`
- **実装時の作業ブランチ**: `feature/v0.22.19-task-x` (x はタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへ PR を作成・マージしてください。

## 1. 依存バージョンを v0.22.19 向けに揃える

- [ ] 1.1 `make help` または Makefile を確認し、依存更新後に使う既存の検証対象（target）を決める
- [ ] 1.2 `Cargo.toml` / `Cargo.lock` の現状が課題 #293 の前提（`katana-canvas-forge = "0.1.6"`、`v8 = "=139.0.0"`、kdr 側 `v8 = "=147.4.0"`）と一致することを確認する
- [ ] 1.3 作業領域の依存関係（workspace dependency）の `katana-canvas-forge` を `0.1.7` へ更新する
- [ ] 1.4 作業領域の依存関係の `v8` を `=147.4.0` へ更新する
- [ ] 1.5 `cargo update -p v8 -p katana-canvas-forge` で `Cargo.lock` を更新する
- [ ] 1.6 `cargo tree -i v8` で kcf / kdr が `v8` の互換グラフ（graph）に揃っていることを確認する

### Definition of Done (DoD)

- [ ] `Cargo.toml` が `katana-canvas-forge = "0.1.7"` と `v8 = "=147.4.0"` を参照していること
- [ ] `Cargo.lock` の更新範囲が kcf / v8 整合に必要な範囲へ限定されていること
- [ ] `cargo tree -i v8` で競合する V8 依存関係（conflicting V8-backed dependency）が残っていないこと
- [ ] `/openspec-delivery` ワークフロー（workflow）（`.codex/workflows/openspec-delivery.md`）を実行し、自己レビュー（Self-review）、コミット（Commit）、PR 作成（PR Creation）、マージ（Merge）までの包括的な配送手順（delivery routine）を通す。

## 2. Mermaid / Draw.io プレビューの回帰を確認する

### Definition of Ready (DoR)

- [ ] 前タスクの配送サイクル（delivery cycle: self-review、必要に応じた recovery、PR 作成、merge、branch deletion）が完了していることを確認する。
- [ ] 基準ブランチ（Base branch）が同期済みで、このタスク用の新しいブランチ（branch）が明示的に作成されていること。

- [ ] 2.1 既存のプレビュー統合テスト（preview integration test）で Mermaid ワーカー切断（worker disconnect）が検出できるか確認する
- [ ] 2.2 既存カバレッジ（coverage）が不足している場合、対応済み Mermaid ブロック（block）が `[Mermaid] Diagram render worker disconnected before producing a result.` へ置換されない回帰テストを追加する
- [ ] 2.3 Draw.io ブロック（block）が V8 バージョン分裂（version split）により描画前の失敗（failure）にならないことを、既存または追加した回帰テストで確認する
- [ ] 2.4 Makefile の既存 target または対象 test target でプレビュー回帰確認（preview regression）を実行し、結果を tasks.md または PR 本文へ記録する

### Definition of Done (DoD)

- [ ] Mermaid プレビューがワーカー切断メッセージ（message）で全面失敗しないこと
- [ ] Draw.io プレビューが kcf / kdr の V8 不整合で失敗しないこと
- [ ] 失敗時の退避表示（fallback message）を隠すだけのテスト（test）になっていないこと
- [ ] `/openspec-delivery` ワークフロー（workflow）（`.codex/workflows/openspec-delivery.md`）を実行し、自己レビュー（Self-review）、コミット（Commit）、PR 作成（PR Creation）、マージ（Merge）までの包括的な配送手順（delivery routine）を通す。

## 3. HTML / PDF / PNG / JPEG 出力の回帰を確認する

### Definition of Ready (DoR)

- [ ] 前タスクの配送サイクル（delivery cycle: self-review、必要に応じた recovery、PR 作成、merge、branch deletion）が完了していることを確認する。
- [ ] 基準ブランチ（Base branch）が同期済みで、このタスク用の新しいブランチ（branch）が明示的に作成されていること。

- [ ] 3.1 Mermaid / Draw.io を含む検証データ（fixture）を HTML 出力し、kcf 0.1.7 経由で図形出力が欠落しないことを確認する
- [ ] 3.2 同じ検証データを PDF / PNG / JPEG 出力し、V8 バージョン分裂由来のワーカー失敗（failure）が発生しないことを確認する
- [ ] 3.3 出力経路に OS ブラウザ依存や古い実行環境退避（runtime fallback）を戻していないことを確認する
- [ ] 3.4 実行した出力回帰確認のコマンド（command）と成果物の確認観点を PR 本文へ記録する

### Definition of Done (DoD)

- [ ] HTML 出力で Mermaid / Draw.io 出力が欠落しないこと
- [ ] PDF / PNG / JPEG 出力で Mermaid / Draw.io 出力が V8 不整合により失敗しないこと
- [ ] kcf 0.1.7 取り込み以外の描画所有境界変更を混ぜていないこと
- [ ] `/openspec-delivery` ワークフロー（workflow）（`.codex/workflows/openspec-delivery.md`）を実行し、自己レビュー（Self-review）、コミット（Commit）、PR 作成（PR Creation）、マージ（Merge）までの包括的な配送手順（delivery routine）を通す。

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

- [ ] 5.1 `docs/coding-rules.ja.md` と `$self-review` skill を使って自己レビューを実行する
- [ ] 5.2 更新した Markdown 文書（tasks.md、CHANGELOG.md など）を整形し、lint 修正（lint-fix）を行う
- [ ] 5.3 通常の `git push` で `pre-push` hook を正式な品質ゲートとして通す。例外記録なしに、push 直前の重い `just check` / `just check-light` を二重実行しない
- [ ] 5.4 Base Feature Branch から `master` 向けに PR（pull request）を作成する
- [ ] 5.5 PR 上の CI checks（Lint / Coverage / CodeQL）が通ることを確認する。失敗があれば merge を止める
- [ ] 5.6 master へ merge する（`gh pr merge --merge --delete-branch`）
- [ ] 5.7 master から `release/v0.22.19` branch を作成する
- [ ] 5.8 `just VERSION=0.22.19 release` を実行し、CHANGELOG を更新する（`changelog-writing` skill）
- [ ] 5.9 `release/v0.22.19` から `master` 向けに PR を作成し、Release Readiness CI が通ることを確認する
- [ ] 5.10 release PR を master へ merge する（`gh pr merge --merge --delete-branch`）
- [ ] 5.11 GitHub Release の完了を確認し、`/opsx-archive` でこの change を archive する
