## MODIFIED Requirements

### Requirement: HTML / PDF / PNG / JPEG export は katana-canvas-forge 経由で行わなければならない

システムは、Markdown の HTML / PDF / PNG / JPEG export を、KatanA 内部実装ではなく外部 library `katana-canvas-forge`（kcf）v0.1.0 の `Exporter` trait 経由で行わなければならない（MUST）。

#### Scenario: HTML 出力を kcf 経由に切り替える

- **WHEN** ユーザーが「HTMLとして出力」を選択する
- **THEN** KatanA は kcf の `Exporter` trait（HTML format）に `ExportInput` を渡し、生成された HTML を一時ディレクトリに保存してデフォルトブラウザで開く
- **THEN** KatanA repository 内に HTML 生成の実装本体（`crates/katana-core/src/markdown/export/`）は残らない

#### Scenario: PDF 出力を kcf 経由に切り替える

- **WHEN** ユーザーが「PDFとして出力」を選択する
- **THEN** KatanA は kcf の `Exporter` trait（PDF format）に `ExportInput` を渡し、生成された PDF を保存先ダイアログで指定された場所に保存する

#### Scenario: 画像出力（PNG / JPEG）を kcf 経由に切り替える

- **WHEN** ユーザーが「PNG / JPGとして出力」を選択する
- **THEN** KatanA は kcf の `Exporter` trait（PNG / JPEG format）に `ExportInput` を渡し、生成された画像を保存先ダイアログで指定された場所に保存する

#### Scenario: 未対応の export 経路を明示する

- **WHEN** kcf が `ExportError::UnsupportedFormat` を返す
- **THEN** KatanA は明示的に未対応である旨を UI に表示する
- **THEN** KatanA は OS Chrome / Chromium app への暗黙 fallback を持たない
