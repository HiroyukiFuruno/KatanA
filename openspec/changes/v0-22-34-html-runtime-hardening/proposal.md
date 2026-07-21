## Why

KatanA v0.22.33 の browser-equivalent HTML viewer は基本操作を満たしたが、許可済み remote subresource と embedded SVG、worker 起動失敗後の復旧、原因を追跡できるエラー表示が不足していた。画像プレビューにも fullscreen 中の scroll で寸法が戻る不具合と、light theme または白い画像上で操作アイコンが見えない問題があるため、v0.22.34 で runtime chain と表示契約を一体で強化する。

## What Changes

- KRR の Rust/V8 HTML runtime が policy で許可した HTTP/HTTPS stylesheet、script、image と embedded SVG を描画し、個別 subresource 失敗時も主文書を維持する
- KDV worker が session 起動失敗後も停止せず、後続 resize/navigation で復旧できるようにする
- KatanA の HTML error UI とログに layer、operation、document URL、一次 cause を表示し、後発の worker stop が一次原因を隠さないようにする
- fullscreen 画像で上下左右 scroll を pan として扱い、拡大後の寸法を保持する
- 最大化、右下 controls、fullscreen close を theme と画像色に依存しない固定背景と明示 border で描画する
- 公開済み KRR `0.4.4`、KDV `0.3.2` の registry-only chain を KatanA v0.22.34 に取り込み、Chromium、WebView、external browser/helper を拒否する
- 既存の厳格 coverage、release、SemVer gate を緩和せず、headless 操作証跡とスクリーンショットを release 判定に含める

## Capabilities

### New Capabilities

- `image-preview-interaction`: fullscreen scroll の寸法保持と、画像内容・theme に依存しない操作 control の視認性を規定する

### Modified Capabilities

- `html-file-preview`: browser worker recovery、追跡可能な error、remote subresource、embedded SVG、および v0.22.34 registry-only release evidence の要件を追加する

## Impact

- KRR: HTML subresource policy、DOM projection、SVG layout、runtime diagnostics
- KDV: browser-session worker lifecycle と typed operation error
- KatanA: HTML surface error state/log、image/diagram controls、fullscreen input、release dependency と headless harness
- Release: KRR v0.4.4 -> KDV v0.3.2 -> KatanA v0.22.34 の公開順序、最新 v0.22.33 にだけ隣接する SemVer guard
