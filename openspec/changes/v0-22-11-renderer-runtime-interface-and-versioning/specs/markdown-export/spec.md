## MODIFIED Requirements

### Requirement: HTML出力

現在開いているMarkdownをHTMLファイルとしてエクスポートし、デフォルトブラウザで開く。 The system SHALL conform. Mermaid / Draw.io を含む diagram block は、preview と矛盾しない runtime 所有境界で処理しなければならない（MUST）。

#### Scenario: HTMLエクスポート

- **WHEN** ユーザーがメニューから「HTMLとして出力」を選択する
- **THEN** Markdownが完全なHTMLファイルに変換され、一時ディレクトリに保存されてデフォルトブラウザで開かれる

#### Scenario: HTML内のスタイル

- **WHEN** HTMLがエクスポートされる
- **THEN** 現在のテーマに基づいたCSSスタイルが埋め込まれ、プレビューと同等の表示になる

#### Scenario: HTML export keeps diagram runtime ownership explicit

- **WHEN** exported HTML includes Mermaid or Draw.io output
- **THEN** the system uses the same documented diagram runtime boundary as preview
- **THEN** the system does not depend on an unversioned Mermaid.js bundle or the user's OS Chrome / Chromium app

### Requirement: PDF出力

現在開いているMarkdownをPDFファイルとしてエクスポートする。 The system SHALL conform. HTML から PDF へ変換する runtime は、diagram runtime と接続境界が矛盾しない形で扱わなければならない（MUST）。

#### Scenario: PDF出力

- **WHEN** ユーザーがメニューから「PDFとして出力」を選択する
- **THEN** 保存先ダイアログが表示され、選択した場所にPDFが保存される

#### Scenario: 外部ツール未インストール時

- **WHEN** PDF生成に必要な外部ツールがインストールされていない
- **THEN** エラーメッセージとインストールガイドが表示される

#### Scenario: PDF export records unsupported runtime gaps

- **WHEN** PDF export cannot yet use the documented app-owned runtime boundary
- **THEN** the system records the gap as a known unsupported or deferred export path
- **THEN** the system does not silently restore OS Chrome / Chromium app dependency

### Requirement: 画像出力（PNG / JPG）

現在開いているMarkdownをPNGまたはJPG画像としてエクスポートする。 The system SHALL conform. HTML から PNG / JPG へ変換する runtime は、diagram runtime と接続境界が矛盾しない形で扱わなければならない（MUST）。

#### Scenario: 画像エクスポート

- **WHEN** ユーザーがメニューから「PNGとして出力」または「JPGとして出力」を選択する
- **THEN** 保存先ダイアログが表示され、選択した場所に画像が保存される

#### Scenario: 長いドキュメントの画像出力

- **WHEN** ドキュメントがビューポートより長い
- **THEN** ドキュメント全体が1枚の縦長画像として出力される

#### Scenario: Image export records unsupported runtime gaps

- **WHEN** PNG or JPG export cannot yet use the documented app-owned runtime boundary
- **THEN** the system records the gap as a known unsupported or deferred export path
- **THEN** the system does not silently restore OS Chrome / Chromium app dependency
