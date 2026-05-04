## 現状の interface 棚卸し

### 既存 neutral interface（整備対象）

| module | trait | 状態 |
|--------|-------|------|
| `markdown/diagram_backend/adapter.rs` | `DiagramBackendAdapter` | 既存。`id()` / `version()` / `render()` あり |
| `preview/adapter/service.rs` | `PreviewAdapter` | 既存。`render()` あり |
| `ai/mod.rs` | `AiProvider` | 既存・完成。**本 change では触らない** |

### trait が存在しない（新設対象）

| module | 追加する trait / 型 |
|--------|-------------------|
| `markdown/export/` | `ExporterTrait`、`ExportInput`、`ExportOutput`、`ExportFormat`、`ExportError` |
| `editor/`（新設） | `EditorWidget`、`SyntaxHighlighter`、`EditorConfig`、`HighlightedText` |

---

## 新設 interface 定義

### ExporterTrait（`katana-core/src/markdown/export/`）

```rust
pub enum ExportFormat { Html, Pdf, Png, Jpeg }

pub struct ExportInput {
    pub format: ExportFormat,
    pub html_source: String,      // 変換済み HTML を渡す（Markdown → HTML は呼び出し側が担う）
    pub output_path: std::path::PathBuf,
    pub config: ExportConfig,
}

pub struct ExportConfig {
    pub paper_size: PaperSize,    // A4 / Letter / 任意
    pub margin_mm: f32,
}

pub struct ExportOutput {
    pub output_path: std::path::PathBuf,
    pub format: ExportFormat,
}

pub enum ExportError { IoError(String), RenderFailed(String), UnsupportedFormat }

pub trait ExporterTrait: Send + Sync {
    fn export(&self, input: &ExportInput) -> Result<ExportOutput, ExportError>;
    fn supported_formats(&self) -> &[ExportFormat];
}
```

既存の `HtmlExporter` / `PdfExporter` / `ImageExporter` は `ExporterTrait` を impl するように変更する。呼び出し側（`katana-ui`）は `Box<dyn ExporterTrait>` を受け取るだけにする。

### EditorWidget / SyntaxHighlighter（`katana-core/src/editor/`）

```rust
pub struct HighlightedSpan { pub range: std::ops::Range<usize>, pub token_kind: TokenKind }
pub struct HighlightedText { pub spans: Vec<HighlightedSpan> }

pub trait SyntaxHighlighter: Send + Sync {
    fn highlight(&self, source: &str) -> HighlightedText;
}

pub struct EditorConfig {
    pub syntax_highlighter: Box<dyn SyntaxHighlighter>,
    pub font_size: f32,
    pub theme_is_dark: bool,
}

pub trait EditorWidget {
    fn apply_config(&mut self, config: EditorConfig);
}
```

KatanA は `struct MarkdownSyntaxHighlighter` を実装し、`EditorConfig` に注入する。`katana-language-editor-egui` intake 後は同じ `EditorConfig` をそのまま渡すだけで差し替え完了になる。

---

## 既存 interface の整備方針

### DiagramBackendAdapter

- シグネチャは変更しない
- `pub use` のスコープを確認し、`katana-ui` が `adapter` モジュールの impl 詳細に直接触れていないことを保証する
- kcf intake（v0.26.0）時は、kcf の `impl DiagramBackendAdapter` を `Box<dyn DiagramBackendAdapter>` として差し込むだけにする

### PreviewAdapter

- シグネチャは変更しない
- 同様に `pub use` スコープを確認する
- kdp intake（v0.26.0 以降）時は、kdp の `impl PreviewAdapter` を差し込むだけにする
