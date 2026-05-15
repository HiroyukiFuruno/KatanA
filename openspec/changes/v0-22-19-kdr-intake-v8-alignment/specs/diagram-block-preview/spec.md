## MODIFIED Requirements

### Requirement: 図形ブロック描画は katana-diagram-renderer 経由で行わなければならない

システムは、`mermaid` および `drawio` Markdown フェンスの描画を、KatanA 内部実装ではなく外部 library `katana-diagram-renderer`（kdr）v0.1.0 の `Renderer` trait 経由で行わなければならない（MUST）。`katana-canvas-forge`（kcf）は本 change 時点では HTML / PDF / PNG / JPEG export 専用とし、描画は kdr へ委譲する。

#### Scenario: kdr を crates.io dependency として参照する

- **WHEN** KatanA workspace を build する
- **THEN** `Cargo.toml` の `[workspace.dependencies]` に `katana-diagram-renderer = "0.1.0"` が含まれる
- **THEN** `Cargo.toml` の `[workspace.dependencies]` に `katana-canvas-forge = "0.1.7"` が含まれる（export 用に残置）
- **THEN** `Cargo.toml` の `[workspace.dependencies]` に `v8 = "=147.4.0"` が含まれ、kdr / kcf と v8 ランタイムが一意である

#### Scenario: Mermaid 描画が kdr 経由になる

- **WHEN** preview が Mermaid block を描画する
- **THEN** `KatanaMermaidBackend` の `id().implementation` は `kdr-mermaid` を返す
- **THEN** バックエンドは `katana_diagram_renderer::MermaidRenderer::with_runtime_path(...)` を呼び、`RenderInput`（source / config / policy / context）を渡し、`RenderOutput`（SVG）を受け取る
- **THEN** バックエンド version 文字列は `crate=katana-diagram-renderer:<version>;runtime=Mermaid.js:<runtime>;profile=katana-mermaid` の形式で構成される

#### Scenario: Draw.io 描画が kdr 経由になる

- **WHEN** preview が Draw.io block を描画する
- **THEN** `KatanaDrawIoBackend` の `id().implementation` は `kdr-drawio` を返す
- **THEN** バックエンドは `katana_diagram_renderer::DrawioRenderer::with_runtime_path(...)` を呼び、`RenderInput` を渡し、`RenderOutput` を受け取る
- **THEN** バックエンド version 文字列は `crate=katana-diagram-renderer:<version>;runtime=Draw.io:<runtime>;profile=katana-drawio` の形式で構成される

### Requirement: ダイアグラムキャッシュキーは kdr の RuntimeVersion を含めなければならない

システムは、永続化されるダイアグラムキャッシュキーに kdr の `RuntimeVersion`、`RendererProfile`、テーマ fingerprint を含めなければならない（SHALL）。kdr の release で `RuntimeVersion` または `RendererProfile` が変わった場合、古いキャッシュ結果を再利用してはならない（MUST NOT）。

#### Scenario: kdr runtime と profile の差分で cache key が変化する

- **WHEN** kdr の runtime version または renderer profile が変わった時
- **THEN** KatanA の diagram cache key は変化する
- **THEN** KatanA は古い kdr 出力を再利用しない

#### Scenario: kdr の crate version を手書きで固定しない

- **WHEN** KatanA が kdr backed renderer の cache key または backend version を組み立てる
- **THEN** system は実際の `katana-diagram-renderer` dependency version（`build.rs` で Cargo.lock から取得して `KATANA_DIAGRAM_RENDERER_VERSION` として注入される値）、`RenderOutput.runtime`、`RenderOutput.profile` から識別情報を得る
- **THEN** `crate=katana-canvas-forge:0.1.0` のような古い手書き文字列を cache invalidation の根拠にしない

### Requirement: v8 ランタイムは kdr と kcf で同一バージョンに固定される

システムは、`katana-diagram-renderer` と `katana-canvas-forge` が同一プロセス内で要求する `v8` クレートのバージョンを一致させなければならない（MUST）。両者で v8 が異なると V8 ランタイムが 2 バージョン共存し、Mermaid 描画ワーカーが起動時に panic して preview 上で

```text
[Mermaid] Diagram render worker disconnected before producing a result.
```

を返す（katana #293, kcf #15）。

#### Scenario: workspace の v8 pin が kdr と kcf に揃う

- **WHEN** KatanA が kdr と kcf を同時に取り込む
- **THEN** `Cargo.toml` workspace の `v8 = "=147.4.0"` が kdr 0.1.0 / kcf 0.1.7 の v8 pin と一致している
- **THEN** `cargo tree -p v8` が単一バージョンのみ報告する

#### Scenario: 描画ワーカーが起動時に切断されない

- **WHEN** preview 上で複数の Mermaid block を含む Markdown ドキュメントを開く
- **THEN** Mermaid 全カテゴリ（基本フロー / Venn / Wardley / XY Chart / ZenUML 等）の図形が描画される
- **THEN** いずれのブロックも `Diagram render worker disconnected before producing a result.` メッセージで失敗しない
