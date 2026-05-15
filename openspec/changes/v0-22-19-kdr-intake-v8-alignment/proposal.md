## Why

ダイアグラム描画責務を `katana-canvas-forge`（kcf）から `katana-diagram-renderer`（kdr）へ切り出すべく、kdr v0.1.0 が crates.io に公開された。
kcf は引き続き HTML / PDF / PNG / JPEG export を担う棲み分けである。

ただし kdr 0.1.0 は `v8 = "=147.4.0"`、当時の kcf 0.1.6 は `v8 = "=139.0.0"` で publish されており、両 crate を同一 workspace に取り込むと v8 ランタイムが 2 バージョン共存して preview の Mermaid 描画ワーカーが起動時に落ち、

```text
[Mermaid] Diagram render worker disconnected before producing a result.
```

ですべての Mermaid 図形が失敗する状態となった（katana [#293](https://github.com/HiroyukiFuruno/KatanA/issues/293), kcf [#15](https://github.com/HiroyukiFuruno/katana-canvas-forge/issues/15)）。

kcf 0.1.7 で v8 を `=147.4.0` に揃えて再リリース済みのため、本 change で KatanA を一斉に整合させる。

## What Changes

- `Cargo.toml` workspace dependency を以下に更新:
  - `katana-diagram-renderer = "0.1.0"`（新規追加。描画責務）
  - `katana-canvas-forge = "0.1.7"`（v8 整合済み。export 責務のみ残置）
  - `v8 = "=147.4.0"`
- 描画系の `use katana_canvas_forge::...` を `use katana_diagram_renderer::...` へ移行:
  - `crates/katana-core/src/markdown/diagram_runtime_assets.rs`
  - `crates/katana-core/src/markdown/diagram_backend/katana_backend.rs`
  - `crates/katana-core/src/markdown/diagram_backend/kdr_theme_adapter.rs`（旧 `kcf_theme_adapter.rs` をリネーム）
  - `crates/katana-core/src/markdown/diagram_backend/impls.rs`（`from_kcf` → `from_kdr`）
- `crates/katana-core/build.rs` の `KATANA_CANVAS_FORGE_VERSION` 環境変数を `KATANA_DIAGRAM_RENDERER_VERSION` に切替
- `markdown/export/mod.rs` は kcf 残置（HtmlExporter / ImageExporter / PdfExporter / `markdown::color_preset::DiagramColorPreset` は kcf 0.1.7 を使い続ける）
- 周辺リポジトリの docs / openspec を新責務分離（kdr 描画 / kcf export）に追従

## Capabilities

### Modified Capabilities

- `diagram-block-preview`: Mermaid / Draw.io 描画は `katana-diagram-renderer` 経由になる。`RuntimeVersion` / `RendererProfile` / `crate=katana-diagram-renderer:<version>` の識別が backend version 文字列に反映される。
- `markdown-export`: HTML / PDF / PNG / JPEG export は `katana-canvas-forge` 0.1.7 経由のまま維持する。

## Impact

- `Cargo.toml` / `Cargo.lock`: kdr 0.1.0 を追加、kcf を 0.1.7 に更新、workspace の v8 を =147.4.0 に bump。
- `crates/katana-core` の描画 import を kdr へ全面切替（export 系は kcf のまま）。
- 影響を受けるテスト: `markdown::diagram_backend::*` / `markdown::diagram_runtime_assets::*` / `katana-ui` の preview_pane 統合テスト。
- BUG 修正: `Diagram render worker disconnected before producing a result.` の解消。
- ユーザー影響: preview 上で Mermaid 図形（全カテゴリ）が再び描画されるようになる。

## Out of Scope

- export 機能の kdr 移管。kdr には現状 exporter が無く、KDV 移譲計画もまだ進行中のため本 change では触らない。
- kcf の責務縮小（export 専用化）の openspec 上の正式取り扱いは `establish-kme-markdown-platform` 配下で別途扱う。
