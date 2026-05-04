## MODIFIED Requirements

### Requirement: 図形ブロック描画は katana-canvas-forge 経由で行わなければならない

システムは、`mermaid` および `drawio` Markdown フェンスの描画を、KatanA 内部実装ではなく外部 library `katana-canvas-forge`（kcf）v0.1.0 の `Renderer` trait 経由で行わなければならない（MUST）。

#### Scenario: kcf を git dependency として参照する

- **WHEN** KatanA workspace を build する
- **THEN** `Cargo.toml` の workspace dependencies に `katana-canvas-forge = { git = "...", tag = "v0.1.0" }` が含まれる
- **THEN** `katana-canvas-forge` の `cargo tree` に `egui` が含まれない

#### Scenario: Mermaid 描画を kcf 経由に切り替える

- **WHEN** preview が Mermaid block を描画する
- **THEN** KatanA は kcf の `Renderer` trait に `RenderInput`（source / config / policy / context）を渡し、`RenderOutput`（SVG / 診断情報 / `RuntimeVersion` / `RendererProfile`）を受け取る
- **THEN** KatanA repository 内に Mermaid 描画の実装本体（`crates/katana-core/src/markdown/mermaid_renderer/`）は残らない
- **THEN** `vendor/mermaid/`、`scripts/mermaid/`、`assets/fixtures/mermaid_all/` も KatanA から除去されている

#### Scenario: Draw.io 描画を kcf 経由に切り替える

- **WHEN** preview が Draw.io block を描画する
- **THEN** KatanA は kcf の `Renderer` trait 経由で描画する
- **THEN** KatanA repository 内に Draw.io 描画の実装本体（`crates/katana-core/src/markdown/drawio_renderer/`）は残らない

#### Scenario: cache key に kcf の RuntimeVersion を含める

- **WHEN** KatanA が描画 cache key を作る
- **THEN** cache key には source、kcf の `RuntimeVersion`、`RendererProfile`、`RenderConfig`、`RenderPolicy`、theme fingerprint が含まれる
- **THEN** kcf の release で `RuntimeVersion` または `RendererProfile` が変わった場合、古い描画 cache を再利用しない
