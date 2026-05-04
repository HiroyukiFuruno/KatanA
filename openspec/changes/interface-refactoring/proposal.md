## Why

各外部リポジトリ（katana-canvas-forge / katana-document-preview / katana-language-editor）を順次 intake する前提として、KatanA 側の呼び出し境界を neutral interface（trait + DTO）として明確に定義する。

現状：
- `DiagramBackendAdapter` trait は `katana-core/src/markdown/diagram_backend/adapter.rs` に存在するが、export（HTML / PDF / PNG / JPEG）には trait がなく `HtmlExporter` / `PdfExporter` / `ImageExporter` の struct 直呼びになっている
- `PreviewAdapter` trait は `katana-core/src/preview/adapter/service.rs` に存在する
- Language editor には trait が存在しない

intake 時は KatanA がこれらの trait 越しに外部 crate を呼ぶだけになり、実装の移管先が変わっても KatanA 本体の呼び出しコードは変更不要になる。

master で直接作業し、version bump・release branch は作成しない。

## What Changes

### 1. Diagram renderer interface の整備（`katana-core/src/markdown/diagram_backend/`）

既存の `DiagramBackendAdapter` trait を kcf との境界として整備する。

- `DiagramBackendAdapter` trait のシグネチャを確認・必要に応じて補完する（`id()` / `version()` / `render()`）
- DTO（`DiagramBackendInput` / `DiagramBackendOutput` / `DiagramBackendCacheKey` / `DiagramThemeSnapshot` 等）が kcf と共有できる形になっているか確認する
- `pub use` の公開スコープを整理し、KatanA UI から直接 impl 詳細を参照していないことを確認する

### 2. Export interface の新設（`katana-core/src/markdown/export/`）

現在 trait がない export 系に `ExporterTrait` を定義し、kcf 委譲の境界にする。

- `ExportInput`：export 対象（HTML 文字列 / Markdown source）、出力先 path、用紙サイズ・余白等の設定
- `ExportOutput`：生成されたファイル path または bytes
- `ExportFormat` enum：`Html` / `Pdf` / `Png` / `Jpeg`
- `ExporterTrait: Send + Sync`：`fn export(&self, input: &ExportInput) -> Result<ExportOutput, ExportError>`
- 既存の `HtmlExporter` / `PdfExporter` / `ImageExporter` が `ExporterTrait` を impl するように切り替える（挙動は変えない）

### 3. Document preview interface の整備（`katana-core/src/preview/adapter/`）

既存の `PreviewAdapter` trait を kdp との境界として整備する。

- `PreviewAdapter` trait のシグネチャを確認・必要に応じて補完する（`render()`）
- DTO（`PreviewInput` / `PreviewAdapterResult` / `PreviewThemeSnapshot` 等）が kdp と共有できる形になっているか確認する
- `pub use` の公開スコープを整理する

### 4. Language editor interface の新設（`katana-core/src/editor/`）

trait が存在しないため新設し、kle 委譲の境界にする。

- `SyntaxHighlighter` trait：`fn highlight(&self, source: &str) -> HighlightedText`
- `EditorConfig`：`syntax_highlighter: Box<dyn SyntaxHighlighter>`、`font_size`、`theme` 等
- `EditorWidget` trait：`fn apply_config(&mut self, config: &EditorConfig)`
- KatanA 側は `MarkdownSyntaxHighlighter` を実装して `EditorConfig` に注入するだけにする

## Non-Goals

- chat-ui interface の定義は katana-chat-ui 側で行う（KatanA は `katana-chat-ui-egui` を呼ぶだけ）
- 外部 crate の実装移管・intake は各 vX.X.0 の責務（本 change では既存実装を trait に沿わせるだけ）
- UI コンポーネントの分離・Floem 移行は本 change では行わない

## Impact

- `crates/katana-core/src/markdown/export/` — `ExporterTrait` + DTO 追加、既存 struct が impl するよう変更
- `crates/katana-core/src/editor/` — 新設（`SyntaxHighlighter` / `EditorConfig` / `EditorWidget`）
- `crates/katana-core/src/markdown/diagram_backend/` — 公開スコープ整理のみ（trait シグネチャは既存）
- `crates/katana-core/src/preview/adapter/` — 公開スコープ整理のみ（trait シグネチャは既存）
- 既存の挙動変更なし
