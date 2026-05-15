## MODIFIED Requirements

### Requirement: 図形ブロック描画は katana-diagram-renderer 経由で行わなければならない

システムは、`mermaid` および `drawio` Markdown フェンスの描画を、KatanA 内部実装ではなく外部 library `katana-diagram-renderer`（kdr）v0.1.0 の `Renderer` trait 経由で行わなければならない（MUST）。

> Note: 本 change の当初版では `katana-canvas-forge`（kcf）が描画も担っていたが、後続で kdr が描画専用 crate として分離された。document export は引き続き kcf を利用する。

#### Scenario: kdr を crates.io dependency として参照する

- **WHEN** KatanA workspace を build する
- **THEN** `Cargo.toml` の workspace dependencies に `katana-diagram-renderer = "0.1.0"` が含まれる
- **THEN** `katana-diagram-renderer` の `cargo tree` に `egui` が含まれない

#### Scenario: Mermaid 描画を kdr 経由に切り替える

- **WHEN** preview が Mermaid block を描画する
- **THEN** KatanA は kdr の `Renderer` trait に `RenderInput`（source / config / policy / context）を渡し、`RenderOutput`（SVG / 診断情報 / `RuntimeVersion` / `RendererProfile`）を受け取る
- **THEN** KatanA repository 内に Mermaid 描画の実装本体（`crates/katana-core/src/markdown/mermaid_renderer/`）は残らない
- **THEN** `vendor/mermaid/`、`scripts/mermaid/`、`assets/fixtures/mermaid_all/`、`assets/fixtures/mermaid_parts/`、`assets/fixtures/drawio/official/` も KatanA から除去されている

#### Scenario: Draw.io 描画を kdr 経由に切り替える

- **WHEN** preview が Draw.io block を描画する
- **THEN** KatanA は kdr の `Renderer` trait 経由で描画する
- **THEN** KatanA repository 内に Draw.io 描画の実装本体（`crates/katana-core/src/markdown/drawio_renderer/`）は残らない

#### Scenario: cache key に kdr の RuntimeVersion を含める

- **WHEN** KatanA が描画 cache key を作る
- **THEN** cache key には source、kdr の `RuntimeVersion`、`RendererProfile`、`RenderConfig`、`RenderPolicy`、theme fingerprint が含まれる
- **THEN** kdr の release で `RuntimeVersion` または `RendererProfile` が変わった場合、古い描画 cache を再利用しない
