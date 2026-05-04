## Context

KatanA は assembly host であり、実装は外部リポジトリに委譲する。しかし intake 前の段階では KatanA 内部に実装が残っている。neutral interface を先行定義することで、intake 時のコード変更を「impl 差し替え」のみに限定できる。

## Interface 一覧

### Mermaid / Draw.io renderer

```rust
// crates/katana-core/src/renderer/mod.rs
pub trait RendererTrait: Send + Sync {
    fn render(&self, input: &RenderInput) -> Result<RenderOutput, RenderDiagnostics>;
    fn runtime_version(&self) -> &RuntimeVersion;
    fn profile(&self) -> &RendererProfile;
}
```

### Document preview

```rust
// crates/katana-core/src/preview/mod.rs
pub trait PreviewRendererTrait: Send + Sync {
    fn render(&self, input: &PreviewInput) -> Result<PreviewOutput, PreviewError>;
}
```

### Language editor

```rust
// crates/katana-core/src/editor/mod.rs
pub trait EditorTrait: Send + Sync {
    fn apply_config(&mut self, config: &EditorConfig);
    fn set_highlighter(&mut self, highlighter: Box<dyn SyntaxHighlighter>);
}
```

## 既存 AI interface（変更なし）

`katana-core/src/ai/mod.rs` の `AiProvider` trait + `AiProviderRegistry` は既に neutral interface として完成している。本 change での変更対象外。

## intake 後の差し替えパターン

intake 時は KatanA の `Cargo.toml` に外部 git dependency を追加し、既存の内部 impl をその外部 impl に差し替える。KatanA 本体の呼び出しコードは trait 経由のままなので変更不要。
