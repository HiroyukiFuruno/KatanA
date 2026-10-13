## Why

`katana-canvas-forge`（kcf）v0.1.0 で Mermaid 描画・Draw.io 描画・HTML/PDF/PNG/JPEG export の実装が完成した段階で KatanA に取り込む。現在 `katana-core` に残存する描画実装を除去し、KatanA を純粋な assembly host に近づける。

これは利用者に見える新機能ではなく、描画 backend の責務整理である。そのため version 付き release 計画には含めず、`master` 上の内部リファクタリングとして扱う。

## What Changes

- `Cargo.toml` に crates.io の `katana-canvas-forge` v0.1.0 を dependency として追加する
- `crates/katana-core/src/markdown/mermaid_renderer/`、`drawio_renderer/`、`export/` の実装本体を削除する
- `vendor/mermaid/`、`scripts/mermaid/`、`assets/fixtures/mermaid_all/`、`assets/fixtures/mermaid_parts/`、`assets/fixtures/drawio/official/` を除去する（kcf 側移管済み）
- KatanA 側の Mermaid / Draw.io preview・export を kcf の `Renderer` / `Exporter` trait 経由に切り替える薄い adapter のみ残す
- cache key に kcf の `RuntimeVersion` と `RendererProfile` を含め、kcf 更新で自動無効化する

## Capabilities

### Modified Capabilities

- `diagram-block-preview`: kcf `Renderer` 経由に一本化する
- `markdown-export`: kcf `Exporter` 経由に一本化する

## Impact

- 削除: `katana-core/src/markdown/mermaid_renderer/`、`drawio_renderer/`、`export/` 一式
- 削除: `vendor/mermaid/`、`scripts/mermaid/`、`assets/fixtures/mermaid_all/`、`assets/fixtures/mermaid_parts/`、`assets/fixtures/drawio/official/`、関連 just recipe / screenshot request
- 追加: `Cargo.toml` に crates.io の `katana-canvas-forge` v0.1.0 dependency
- 追加: KatanA 側の薄い adapter（kcf DTO ↔ KatanA preview/export 状態の変換）
- kcf 側の実装は [katana-canvas-forge openspec](https://github.com/HiroyukiFuruno/katana-canvas-forge) を参照
