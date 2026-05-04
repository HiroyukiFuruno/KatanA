# Tasks: Interface Refactoring（master 直接）

> KatanA 側に neutral interface（trait + DTO）を先行定義・整備する。  
> 外部 crate の実装移管・intake は各 vX.X.0 の責務であり本 tasks では扱わない。  
> 既存の挙動は変えない。version branch 不要。master で直接 commit する。

---

## 1. DiagramBackendAdapter の整備（`katana-core/src/markdown/diagram_backend/`）

### 目的

kcf intake（v0.26.0）時に `Box<dyn DiagramBackendAdapter>` を差し込むだけで動く境界にする。

- [ ] 1.1 `DiagramBackendAdapter` trait のシグネチャを確認する
  - `id() -> &DiagramBackendId`
  - `version() -> &DiagramBackendVersion`
  - `render(&self, input: &DiagramBackendInput) -> DiagramBackendRenderResult`
  - 不足があれば補完する（挙動は変えない）
- [ ] 1.2 `katana-ui` が `diagram_backend::adapter` の impl 詳細（具体的な struct）を直接参照している箇所を `git grep` で洗い出す
- [ ] 1.3 直接参照がある場合、`Box<dyn DiagramBackendAdapter>` 経由に切り替える
- [ ] 1.4 `crates/katana-core/src/markdown/diagram_backend/mod.rs` の `pub use` スコープが適切か確認する（impl 詳細が漏れていないこと）
- [ ] 1.5 `cargo test --package katana-core` がエラーなしで通ること

---

## 2. ExporterTrait の新設（`katana-core/src/markdown/export/`）

### 目的

現在 trait がない HTML / PDF / PNG / JPEG export に統一 trait を定義し、kcf 委譲の境界にする。

- [ ] 2.1 `crates/katana-core/src/markdown/export/` に以下を追加する
  - `ExportFormat` enum（`Html` / `Pdf` / `Png` / `Jpeg`）
  - `ExportInput`（`format`, `html_source: String`, `output_path: PathBuf`, `config: ExportConfig`）
  - `ExportConfig`（`paper_size: PaperSize`, `margin_mm: f32`）
  - `ExportOutput`（`output_path: PathBuf`, `format: ExportFormat`）
  - `ExportError` enum（`IoError(String)` / `RenderFailed(String)` / `UnsupportedFormat`）
  - `ExporterTrait: Send + Sync`（`fn export(&self, input: &ExportInput) -> Result<ExportOutput, ExportError>` / `fn supported_formats(&self) -> &[ExportFormat]`）
- [ ] 2.2 既存の `HtmlExporter` が `ExporterTrait` を impl するよう変更する（既存の `export()` fn は内部に保持、trait 経由の呼び出しに対応する）
- [ ] 2.3 既存の `PdfExporter` が `ExporterTrait` を impl するよう変更する
- [ ] 2.4 既存の `ImageExporter`（PNG / JPEG）が `ExporterTrait` を impl するよう変更する
- [ ] 2.5 `katana-ui` の export 呼び出し箇所を `git grep` で洗い出す（`HtmlExporter::export` / `PdfExporter::export` 等の直接呼び出し）
- [ ] 2.6 `katana-ui` の呼び出しを `Box<dyn ExporterTrait>` 経由に切り替える
- [ ] 2.7 `cargo test --package katana-core` がエラーなしで通ること

---

## 3. PreviewAdapter の整備（`katana-core/src/preview/adapter/`）

### 目的

kdp intake 時に `Box<dyn PreviewAdapter>` を差し込むだけで動く境界にする。

- [ ] 3.1 `PreviewAdapter` trait のシグネチャを確認する
  - `render(&self, input: &PreviewInput) -> PreviewAdapterResult`
  - 不足があれば補完する（挙動は変えない）
- [ ] 3.2 `katana-ui` が `preview::adapter` の impl 詳細を直接参照している箇所を `git grep` で洗い出す
- [ ] 3.3 直接参照がある場合、`Box<dyn PreviewAdapter>` 経由に切り替える
- [ ] 3.4 `crates/katana-core/src/preview/adapter/mod.rs` の `pub use` スコープを確認する
- [ ] 3.5 `cargo test --package katana-core` がエラーなしで通ること

---

## 4. EditorWidget / SyntaxHighlighter の新設（`katana-core/src/editor/`）

### 目的

kle intake 時に `EditorConfig` に kle の実装を差し込むだけで動く境界にする。

- [ ] 4.1 `crates/katana-core/src/editor/mod.rs` を新設し以下を定義する
  - `TokenKind` enum（`Keyword` / `String` / `Comment` / `Number` / `Operator` / `Default` 等、最低限の種別）
  - `HighlightedSpan { range: Range<usize>, token_kind: TokenKind }`
  - `HighlightedText { spans: Vec<HighlightedSpan> }`
  - `SyntaxHighlighter: Send + Sync`（`fn highlight(&self, source: &str) -> HighlightedText`）
  - `EditorConfig { syntax_highlighter: Box<dyn SyntaxHighlighter>, font_size: f32, theme_is_dark: bool }`
  - `EditorWidget`（`fn apply_config(&mut self, config: EditorConfig)`）
- [ ] 4.2 KatanA の Markdown editor 部分が `MarkdownSyntaxHighlighter` を実装し `EditorConfig` に注入する形に切り替える（既存の syntect / tree-sitter 呼び出しを `SyntaxHighlighter` impl 内に閉じ込める）
- [ ] 4.3 `katana-ui` の editor 呼び出しが `EditorWidget::apply_config()` 経由になっていることを確認する
- [ ] 4.4 `cargo test --package katana-core` がエラーなしで通ること

---

## 5. 最終確認と commit

- [ ] 5.1 `just check-local` がエラーなし（exit code 0）で通過すること
- [ ] 5.2 `git diff crates/katana-core/src/ai/` で ai module が変更されていないことを確認する
- [ ] 5.3 各 task で洗い出した `katana-ui` の直接参照がすべて trait 経由に切り替わっていることを `git grep` で確認する
  - `git grep "HtmlExporter::" crates/katana-ui/` → ゼロ件
  - `git grep "PdfExporter::" crates/katana-ui/` → ゼロ件
  - `git grep "ImageExporter::" crates/katana-ui/` → ゼロ件
- [ ] 5.4 commit & push（master 直接）
