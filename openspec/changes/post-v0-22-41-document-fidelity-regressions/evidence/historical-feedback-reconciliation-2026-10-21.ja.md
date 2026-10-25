# 旧feedback leafの現行状態照合（2026-10-21）

## 判定範囲

`tasks.md` の 388、396、405（三件）、415 を、現HEAD
`0f6120872811ebafb870089300a3be59151ca397`、元修復commitの祖先関係、既存回帰証跡、
PR346のGraphQL fresh取得で照合した。対象6 threadは `isResolved=true`、nested false、
各修復replyが元指摘へ直接replyしていることを確認した。tasks原文の失敗経緯とDoDは削除せず、
実装済み範囲と未完のrelease/受入を分離する。

## Leaf別の証跡

### 388 watcher overflow

- 原指摘: `PRRT_kwDORm09y86pVutU` / comment `4192078113`。
- 修復: `2a38304b`（現HEAD祖先）。`crates/katana-ui/src/preview_pane/local_image_loader/overflow.rs`、
  `watcher/events/queue.rs`、`watcher/{events.rs,registration.rs,tests.rs}`等で、queue overflowと
  genuine registration failureを分離し、失われたRegistered/Failed、pending、世代/resetを回復する。
- 回帰: 実PNGを含むloader 27件、AST23、strict Clippy/fmt。修復reply `4192687749`
  （`replyTo=4192078113`）は通常push `b8ff6c63` / pre-push exit0を記録する。
- 残り: native/全OS/packaged・本人受入は未完。正式coverageのnamed PASSはこのleafのnative受入を意味しない。

### 396 同一poll未消費結果

- 原指摘: `PRRT_kwDORm09y86pUrOm` / comment `4191642495`。
- 修復: `67841c51`（現HEAD祖先）。`local_image_loader.rs`、`cache.rs`、`batch_tests.rs`で、
  初consumer前の成功/失敗結果をframe限定所有として保持し、16件/64MiB reuse上限を変更せず、
  reset/path revisionと次frame同TextureIdを扱う。
- 回帰: 実pollの17件初表示、次frame同TextureId、Failed再queueなし、grace/reset/invalidationの5件、
  loader16、AST23、strict Clippy/fmt。修復reply `4192050416`（`replyTo=4191642495`）は
  通常push `ed8e35c1` exit0を記録する。
- 残り: native/GPU総量、packaged・本人受入は未完。

### 405 小画像17

- 原指摘: `PRRT_kwDORm09y86pTsE-` / comment `4191233179`。
- 修復: `3b2a0341`、`d39245dd`（現HEAD祖先）。画像loader/cache/display ownership/textureの
  回帰で、表示中所有をreuse cacheから分離し、初回consumer前の即時退避を防ぐ。
- 修復reply `4191620938`（`replyTo=4191233179`）は17枚、非表示後解放、reset、native/Linux
  loader各11、通常push成功を記録する。GraphQL freshはresolved。

### 405 MathJax physics

- 原指摘: `PRRT_kwDORm09y86pTsFA` / comment `4191233183`。
- 修復: `177a0e26`（現HEAD祖先）。`vendor/mathjax_svg/js/src/index.ts`、`js/out/index.mjs`、
  `js/test/index.test.ts`と`crates/katana-ui/src/preview_pane/math_tests.rs`でsource/generated bundle双方のphysics実式を回帰。
- 修復reply `4191621452`（`replyTo=4191233183`）はJS4/22、typecheck/lint、MathLogicOps10、
  通常push成功を記録する。GraphQL freshはresolved。AllPackages 30/30比較は別の過去確認であり、
  今回の修復根拠と混同しない。

### 405 Unicode font identity

- 原指摘: `PRRT_kwDORm09y86pTsFD` / comment `4191233188`。
- 修復: `cc8e9123`（現HEAD祖先）。font metadata/resolver/request dedupe/painterへ
  canonical `FontFamilyIdentity`を適用。
- 修復reply `4191622006`（`replyTo=4191233188`）は旧3 assertion RED、font155、AST23、
  strict lint、通常push成功を記録する。GraphQL freshはresolved。

### 415 外部画像上書き

- 原指摘: `PRRT_kwDORm09y86pR5n5` / comment `4190492654`。
- 修復: `18b1db3f`/`ba48bf8e`（現HEAD祖先）。`local_image_loader/{revision,revision_cache,watcher}`、
  `images.rs`、`fullscreen_local.rs`等で、同期stat/readへ戻さず、変更通知と世代付きinvalidateで
  cache/texture identityを更新する。
- 回帰: atomic rename、両背景、無関係画像保持、reset、実ImageLogicOps idle repaint、新TextureId、
  loader10、AST23、impacted Clippy。修復reply `4191206635`（`replyTo=4190492654`）は通常push
  c07b5028を記録する。GraphQL freshはresolved。
- 残り: native/全OS/packaged・本人受入は未完。

## 正式検査との関係

`evidence/canonical-normal-coverage-2026-10-21.ja.md` は過去のfa9a71b5でのnormal named
session89740 actual0、同fa9のcoverage session95104 actual0、strict document surface 100%・未実行行0を
記録する。これは6 leafの修復reply/resolveと整合するが、native/全OS/packaged/本人受入、trusted reuse、
公開完了を代用しない。
