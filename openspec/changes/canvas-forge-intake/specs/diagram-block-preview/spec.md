## MODIFIED Requirements

### Requirement: 図形ブロック描画は katana-render-runtime 経由で行わなければならない

システムは、`mermaid` および `drawio` Markdown フェンスの描画を、KatanA 内部実装ではなく外部 library `katana-render-runtime`（KRR）の renderer 経由で行わなければならない（MUST）。

> Note: 本 change の当初版では `katana-canvas-forge`（KCF）が描画も担っていたが、後続で KRR が描画 runtime 専用 crate として分離された。document export は KDV へ移譲する。

#### Scenario: KRR を crates.io dependency として参照する

- **WHEN** KatanA workspace を build する
- **THEN** `Cargo.toml` の workspace dependencies に `katana-render-runtime` が含まれる
- **THEN** `katana-render-runtime` の `cargo tree` に `egui` が含まれない
- **THEN** `katana-diagram-renderer` は workspace dependency graph に含まれない

#### Scenario: Mermaid 描画を KRR 経由に切り替える

- **WHEN** preview が Mermaid block を描画する
- **THEN** KatanA は KDV adapter 経由で KRR backed renderer に source / theme / context を渡し、SVG 出力と runtime 識別情報を受け取る
- **THEN** KatanA repository 内に Mermaid 描画の実装本体（`crates/katana-core/src/markdown/mermaid_renderer/`）は残らない
- **THEN** `vendor/mermaid/`、`scripts/mermaid/`、`assets/fixtures/mermaid_all/`、`assets/fixtures/mermaid_parts/`、`assets/fixtures/drawio/official/` も KatanA から除去されている

#### Scenario: Draw.io 描画を KRR 経由に切り替える

- **WHEN** preview が Draw.io block を描画する
- **THEN** KatanA は KDV adapter 経由で KRR backed renderer に描画を委譲する
- **THEN** KatanA repository 内に Draw.io 描画の実装本体（`crates/katana-core/src/markdown/drawio_renderer/`）は残らない

#### Scenario: cache key に KRR の RuntimeVersion を含める

- **WHEN** KatanA が描画 cache key を作る
- **THEN** cache key には source、KRR の `RuntimeVersion`、`RendererProfile`、`RenderConfig`、`RenderPolicy`、theme fingerprint が含まれる
- **THEN** KRR の release で `RuntimeVersion` または `RendererProfile` が変わった場合、古い描画 cache を再利用しない
