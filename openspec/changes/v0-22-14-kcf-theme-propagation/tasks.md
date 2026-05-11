## Definition of Ready (DoR)

- [x] proposal.md、design.md、specs が揃っていること
- [x] kcf 側 issue [#4](https://github.com/HiroyukiFuruno/katana-canvas-forge/issues/4) が KatanA 側の前提として記録されていること
- [x] KatanA 側の対応範囲が「kcf 修正版の取り込み」「adapter と回帰テスト」「不要な renderer asset 取得経路の削除」「証跡生成」に限定されていること

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `v0-22-14-kcf-theme-propagation` またはリリース用統合ブランチ（例: `release/vX.Y.Z`）
- **作業ブランチ**: 標準は `v0-22-14-kcf-theme-propagation-task-x`、リリース用は `feature/v0.22.14-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

## 1. kcf 修正版の取り込み準備

- [ ] 1.1 kcf 側 issue #4 の修正内容を確認し、`RenderInput` 由来のテーマが Mermaid / Draw.io の実描画へ使われることを kcf 側テストで確認する
- [ ] 1.2 kcf の修正版 release version を確認し、KatanA の `katana-canvas-forge` dependency 更新対象を決める
- [ ] 1.3 `Cargo.toml` / `Cargo.lock` を kcf 修正版へ更新する
- [ ] 1.4 `cargo tree -p katana-canvas-forge --no-dedupe` で `egui` が含まれないことを確認する
- [ ] 1.5 kcf issue #4 の修正版が未公開の場合は、KatanA 側で一時的なグローバル同期回避策を入れず、作業を止めて release 待ちまたは kcf 側対応へ切り替える
- [ ] 1.6 kcf が Draw.io / Mermaid の min.js を組み込み済みであることを前提に、KatanA 側の `renderer_assets`、起動時取得、再取得用コマンドパレット（Command Palette）項目、多言語文言、関連 action / state を削除する

### Definition of Done (DoD)

- [ ] KatanA が kcf のテーマ伝播対応版を参照していること
- [ ] kcf 側で `RenderInput` の light / dark 差分が実描画へ反映されることを確認していること
- [ ] KatanA 側に Draw.io / Mermaid の min.js ダウンロード URL、起動時取得、手動修復・再取得導線が残っていないこと
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 2. KatanA adapter のテーマ伝播

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 `DiagramThemeSnapshot` から kcf のテーマ入力へ変換する専用 adapter を追加する
- [ ] 2.2 Mermaid / Draw.io preview が adapter 経由で light / dark のテーマ名、背景、文字色、塗り、線、矢印、Mermaid theme / Draw.io label color を渡すようにする
- [ ] 2.3 `DiagramBlock::render()` と preview dispatch のテーマ入力経路を揃え、片方だけが古い `DiagramColorPreset::current()` に依存しないようにする
- [ ] 2.4 export 開始時点の theme snapshot を background thread へ渡し、thread 内でグローバル状態だけを読み直さないようにする
- [ ] 2.5 kcf 内部 `DARK_MODE` が true でも、KatanA が light を渡した場合に light 入力が維持される回帰テストを追加する

### Definition of Done (DoD)

- [ ] preview / export の両方が同じテーマ変換 adapter を通ること
- [ ] light テーマ入力が kcf へ渡されることを unit test で確認できること
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 3. Cache key と kcf metadata の整理

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 `KCF_MERMAID_BACKEND_VERSION` / `KCF_DRAWIO_BACKEND_VERSION` の古い `0.1.0` 手書き文字列をやめ、実際の kcf version / runtime / renderer profile へ追従する形にする
- [ ] 3.2 KatanA の diagram cache key が、実描画で使われる theme fingerprint、kcf runtime、renderer profile の差分で変化することを保証する
- [ ] 3.3 kcf の `cache_fingerprint` と KatanA の persistent cache key が、light / dark の差分を同じ意味で扱うことを確認する
- [ ] 3.4 既存 cache が dark 配色を再利用する可能性が残る場合は、diagram cache version または key material を更新する
- [ ] 3.5 cache key 回帰テストを追加し、同一 source でも light / dark / kcf runtime/profile の差分で key が変わることを確認する

### Definition of Done (DoD)

- [ ] kcf dependency version と backend metadata の不一致が残っていないこと
- [ ] 古い dark diagram cache が light テーマで再利用されないこと
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 4. Preview / Export の回帰テストと証跡

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 4.1 Mermaid light theme preview の unit / integration test を追加する
- [ ] 4.2 Draw.io light theme preview の unit / integration test を追加する
- [ ] 4.3 HTML export の Mermaid / Draw.io が current theme と一致する回帰テストを追加する
- [ ] 4.4 PDF / PNG / JPEG export の入力 HTML が light テーマの SVG を含むことを確認する対象テストを追加する
- [ ] 4.5 `scripts/screenshot` に light テーマで Mermaid / Draw.io を表示する review 用 request を追加する
- [ ] 4.6 `./scripts/openspec validate v0-22-14-kcf-theme-propagation --strict` を通す
- [ ] 4.7 対象テストと `just check-local` を通す

### Definition of Done (DoD)

- [ ] light テーマで Mermaid / Draw.io が dark 的な配色へ戻らないことをテストと screenshot で確認できること
- [ ] OpenSpec validate と対象品質ゲートが成功していること
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 5. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [ ] 5.1 ユーザーへ実装完了の報告および動作状況を提示する。UI の動作確認は、ユーザーに手動操作を依頼せず、`scripts/screenshot` のシナリオで生成したスクリーンショットまたは動画を提示して確認できる状態にする
- [ ] 5.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）

---

## KatanA CLI Entry Point

このリポジトリでは OpenSpec の実行入口として `./scripts/openspec` を使用すること。グローバルの `openspec` コマンドが見つからない場合でも未導入と判断してはならない。このラッパーは `bunx @fission-ai/openspec`、次に `npx @fission-ai/openspec` へフォールバックする。

このスキル内で `openspec ...` と書かれているコマンドは、リポジトリルートから `./scripts/openspec ...` として実行する。

## 6. Final Verification & Release Work

- [ ] 6.1 Execute self-review using `docs/coding-rules.ja.md` and `$self-review` skill
- [ ] 6.2 Format and lint-fix all updated markdown documents (e.g., tasks.md, CHANGELOG.md)
- [ ] 6.3 通常の `git push` で `pre-push` hook を正式な品質ゲートとして通す。例外記録なしに、push 直前の重い `just check` / `just check-light` を二重実行しない
- [ ] 6.4 Create PR from Base Feature Branch targeting `master`
- [ ] 6.5 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
- [ ] 6.6 Merge into master (`gh pr merge --merge --delete-branch`)
- [ ] 6.7 Create `release/v0.22.14` branch from master
- [ ] 6.8 Run `just VERSION=0.22.14 release` and update CHANGELOG (`changelog-writing` skill)
- [ ] 6.9 Create PR from `release/v0.22.14` targeting `master` — Ensure `Release Readiness` CI passes
- [ ] 6.10 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 6.11 Verify GitHub Release completion and archive this change using `/opsx-archive`
