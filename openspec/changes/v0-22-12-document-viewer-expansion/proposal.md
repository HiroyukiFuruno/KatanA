## Why

KatanA は現在、Markdown と画像（PNG, JPG, GIF, SVG）のプレビューには対応しているが、実務で頻繁に利用される PDF や Office ドキュメント（Word, Excel, PowerPoint）、およびデータ形式である CSV を直接表示する機能が不足している。

ユーザーがこれらのファイルを開く際、外部アプリ（Acrobat, Excel 等）へ切り替える必要があるため、KatanA 内で一貫したドキュメント閲覧体験を提供することを目的とする。また、Google Workspace や Office 365 などの Web ベースのドキュメントを URL から直接表示し、必要に応じてローカルへ保存する機能を追加することで、Web とローカルの境界をシームレスにする。

## What Changes

- **Document Viewer Integration**:
    - **Local Document Preview**:
        - PDF: ページめくり、ズーム機能を備えたプレビュー。
        - CSV: `egui` のテーブルとして表示、簡易的なソート機能。
        - Office (DOCX, XLSX, PPTX): WebView を利用した閲覧専用ビュー。
    - **Web Document Integration**:
        - URL 入力により、WebView を介してスプレッドシートやプレゼンテーション、文書を表示。
        - 埋め込み URL（Google Docs / Sheets / Slides 等）の自動認識と最適化表示。
    - **Local Preservation**:
        - Web で表示しているドキュメントをローカルファイル（.xlsx, .pdf 等）として保存する機能。
        - ダウンロード管理インターフェースの追加。

## Capabilities

### New Capabilities

- `document-viewer`: PDF, CSV, Office ドキュメントを閲覧する機能。
- `web-document-bridge`: URL から Web ドキュメントを埋め込み表示し、ローカル保存へ橋渡しする機能。

### Modified Capabilities

- `workspace-shell`: サイドバーやエクスプローラーから、Markdown 以外のドキュメントを「閲覧モード」で開くアクションの追加。

## Impact

- `crates/katana-ui/src/document_viewer/`: 新規作成。PDF/CSV/Office の各ビューアを管理。
- `crates/katana-ui/src/preview_pane/`: リンクからドキュメントビューアへ遷移するロジックの追加。
- `crates/katana-platform/src/filesystem/`: ダウンロード/保存ロジックの追加。
- `crates/katana-ui/src/webview/`: WebView コンポーネントの導入（`wry` 等の採用検討）。
