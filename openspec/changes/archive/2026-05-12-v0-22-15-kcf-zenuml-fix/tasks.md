## Definition of Ready (DoR)

- [x] proposal.md が作成されていること
- [x] kcf 側 issue [#8](https://github.com/HiroyukiFuruno/katana-canvas-forge/issues/8) の修正が v0.1.5 として公開されていること
- [x] KatanA 側の対応範囲が「`Cargo.toml` / `Cargo.lock` の依存更新のみ」に限定されていること

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `release/v0.22.15`
- **作業ブランチ**: `feature/v0.22.15-task-x` (x はタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへ PR を作成・マージしてください。

## 1. kcf v0.1.5 取り込み

- [x] 1.1 kcf issue #8 の修正内容を確認し、ZenUML 出力契約が改善されていることを kcf 側リリースノートまたはコードで確認する
- [x] 1.2 `Cargo.toml` の `katana-canvas-forge` を `0.1.5` へ更新する
- [x] 1.3 `cargo update -p katana-canvas-forge` で `Cargo.lock` を更新する

### Definition of Done (DoD)

- [x] `Cargo.toml` / `Cargo.lock` が `katana-canvas-forge = "0.1.5"` を参照していること
- [x] `cargo check` でコンパイルエラーが無いこと
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [x] 2.1 ユーザーへ実装完了の報告および動作状況を提示する。UI の動作確認は、ユーザーに手動操作を依頼せず、`scripts/screenshot` のシナリオで生成したスクリーンショットまたは動画を提示して確認できる状態にする。スクリーンショット `scripts/screenshot/examples/v0-22-15-zenuml-verify.json` で ZenUML が SolarizedLight テーマ背景（クリーム色）で正しく描画されることを確認した（白背景解消）。
- [x] 2.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）。
  - dark テーマで ZenUML 内部配色（ライフライン・Alt/Par ブロック等）が light 配色のまま表示される問題を確認。KatanA 側は `RenderThemeMode::Dark` を正しく渡しており、**kcf 側の問題**と判断。kcf issue [#12](https://github.com/HiroyukiFuruno/katana-canvas-forge/issues/12) を起票した。kcf 側修正取り込み後に v0.22.15 としてリリースする方針のため、本日は issue 起票で終了（個別劣後ではなく次セッション継続）。

---

## KatanA CLI Entry Point

このリポジトリでは OpenSpec の実行入口として `./scripts/openspec` を使用すること。グローバルの `openspec` コマンドが見つからない場合でも未導入と判断してはならない。このラッパーは `bunx @fission-ai/openspec`、次に `npx @fission-ai/openspec` へフォールバックする。

このスキル内で `openspec ...` と書かれているコマンドは、リポジトリルートから `./scripts/openspec ...` として実行する。

## 3. Final Verification & Release Work

- [x] 3.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [x] 3.2 Format and lint-fix all updated markdown documents (e.g., tasks.md, CHANGELOG.md)
- [x] 3.3 通常の `git push` で `pre-push` hook を正式な品質ゲートとして通す。例外記録なしに、push 直前の重い `just check` / `just check-light` を二重実行しない
- [x] 3.4 Create PR from `release/v0.22.15` targeting `master`
- [x] 3.5 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
- [x] 3.6 Merge into master (`gh pr merge --merge --delete-branch`)
- [x] 3.7 Verify GitHub Release completion and archive this change using `/opsx-archive`
