## Why

v0.22.10 で Rust 管理 JS（Rust-managed JavaScript runtime）による Mermaid 描画が高速な採用候補になった一方、KatanA 本体が Mermaid.js の描画専門性、版（version）管理、検証画像更新まで抱える状態は保守境界として重い。

Lint を KML（katana-markdown-linter）へ分けたのと同じ考え方で、Mermaid / Draw.io / export 描画は早期に `katana-renderer` へ切り出せる接続境界（interface）へ整理する。

## What Changes

- KatanA から Mermaid 描画器（renderer）へ渡す入力、Mermaid.js 互換 config、KatanA 独自 policy、出力、診断情報の接続境界を明示する。
- 現在の無印 `mermaid.min.js` 利用をやめ、利用する Mermaid.js の版（version）を固定して KatanA 内へ一時的に埋め込む。
- 埋め込み Mermaid.js は checksum（改ざん検知用のハッシュ）と更新手順を持ち、cache key と比較証跡へ版情報を含める。
- `katana-renderer` 分離設計を文書化し、KatanA と `katana-renderer` の責務、API 境界、更新手順、検証機構を分ける。
- Draw.io 描画と HTML / PDF / PNG / JPEG export の所有境界を同じ設計上の懸念として整理する。
- 将来の preview 分離を前提に、preview は `katana-renderer` を利用できるが、`katana-renderer` は preview / egui / KatanA UI に依存しない構造にする。
- `mmdc` より軽く速い描画体験を価値として扱い、初回描画と連続描画の性能証跡を残せるようにする。
- 将来の `katana-renderer` CLI を想定し、単体 render、公式比較画像更新、性能計測を同じ core API から呼べる設計にする。

## Capabilities

### New Capabilities

- `renderer-runtime-interface`: KatanA と描画 runtime の接続境界、Mermaid.js 版固定、将来の `katana-renderer` 分離設計を扱う。

### Modified Capabilities

- `diagram-block-preview`: Mermaid preview は KatanA 内部実装詳細ではなく、版付き描画 runtime の結果として扱う。
- `markdown-export`: HTML / PDF / PNG / JPEG export は、Mermaid / Draw.io 描画 runtime の所有境界を曖昧にせず、未接続部分を明示する。

## Impact

- `crates/katana-core/src/markdown/mermaid_renderer/`
- `crates/katana-core/src/markdown/drawio_renderer/`
- `crates/katana-core/src/markdown/export/`
- `crates/katana-core/tests/markdown_mermaid.rs`
- `crates/katana-core/tests/markdown_drawio.rs`
- `crates/katana-core/tests/markdown_svg_rasterize.rs`
- `scripts/mermaid/`
- `assets/fixtures/mermaid_all/`
- 将来の `katana-renderer` repository へ移す設計文書
