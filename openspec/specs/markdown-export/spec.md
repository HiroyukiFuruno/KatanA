## Purpose

This is a legacy capability specification that was automatically migrated to comply with the new OpenSpec schema validation rules. Please update this document manually if more context is required.
## Requirements
### Requirement: HTML出力

現在開いているMarkdownをHTMLファイルとしてエクスポートし、デフォルトブラウザで開く。 The system SHALL conform.

#### Scenario: HTMLエクスポート

- **WHEN** ユーザーがメニューから「HTMLとして出力」を選択する
- **THEN** Markdownが完全なHTMLファイルに変換され、一時ディレクトリに保存されてデフォルトブラウザで開かれる

#### Scenario: HTML内のスタイル

- **WHEN** HTMLがエクスポートされる
- **THEN** 現在のテーマに基づいたCSSスタイルが埋め込まれ、プレビューと同等の表示になる

### Requirement: PDF出力

現在開いているMarkdownをPDFファイルとしてエクスポートする。 The system SHALL conform.

#### Scenario: PDF出力

- **WHEN** ユーザーがメニューから「PDFとして出力」を選択する
- **THEN** 保存先ダイアログが表示され、選択した場所にPDFが保存される

#### Scenario: 外部ツール未インストール時

- **WHEN** PDF生成に必要な外部ツールがインストールされていない
- **THEN** エラーメッセージとインストールガイドが表示される

### Requirement: 画像出力（PNG / JPG）

現在開いているMarkdownをPNGまたはJPG画像としてエクスポートする。 The system SHALL conform.

#### Scenario: 画像エクスポート

- **WHEN** ユーザーがメニューから「PNGとして出力」または「JPGとして出力」を選択する
- **THEN** 保存先ダイアログが表示され、選択した場所に画像が保存される

#### Scenario: 長いドキュメントの画像出力

- **WHEN** ドキュメントがビューポートより長い
- **THEN** ドキュメント全体が1枚の縦長画像として出力される

### Requirement: Export 時の図形ブロックは現在テーマで描画される

システムは、HTML / PDF / PNG / JPEG export に含まれる Mermaid / Draw.io 図形ブロックを、export 開始時点の KatanA テーマスナップショットで描画しなければならない（SHALL）。export thread は kcf 内部の既定テーマや、export 実行時に変化しうるグローバル状態に依存してはならない（MUST NOT）。

#### Scenario: HTML export の Mermaid が light テーマで描画される

- **WHEN** KatanA の active theme が light mode の状態で Mermaid block を含む Markdown を HTML export する
- **THEN** export された HTML 内の Mermaid SVG は KatanA が渡した light テーマに基づく
- **THEN** HTML 全体の CSS と Mermaid 図形の配色が矛盾しない

#### Scenario: PDF / PNG / JPEG export の Mermaid が light テーマで描画される

- **WHEN** KatanA の active theme が light mode の状態で Mermaid block を含む Markdown を PDF / PNG / JPEG export する
- **THEN** native export の入力 HTML に含まれる Mermaid SVG は light テーマに基づく
- **THEN** 出力画像または PDF 内で Mermaid 図形だけが dark 的な配色へ戻らない

#### Scenario: Export thread はテーマスナップショットを受け取る

- **WHEN** export 処理が background thread で実行される
- **THEN** export 開始時点で取得した theme snapshot が thread へ渡される
- **THEN** thread 内で `DiagramColorPreset::current()` のようなグローバル状態だけを読み直してテーマを決めない

