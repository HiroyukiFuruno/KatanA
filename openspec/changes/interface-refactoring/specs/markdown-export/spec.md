## MODIFIED Requirements

### Requirement: HTML出力

現在開いているMarkdownをHTMLファイルとしてエクスポートし、デフォルトブラウザで開く。 The system SHALL conform. HTML 生成は外部 library `katana-canvas-forge`（kcf）の export API を経由しなければならない（MUST）。

#### Scenario: HTMLエクスポート

- **WHEN** ユーザーがメニューから「HTMLとして出力」を選択する
- **THEN** Markdownが完全なHTMLファイルに変換され、一時ディレクトリに保存されてデフォルトブラウザで開かれる
- **THEN** HTML 生成は kcf の export API を通じて行われる

#### Scenario: HTML内のスタイル

- **WHEN** HTMLがエクスポートされる
- **THEN** 現在のテーマに基づいたCSSスタイルが埋め込まれ、プレビューと同等の表示になる

#### Scenario: HTML export goes through kcf

- **WHEN** exported HTML includes Mermaid or Draw.io output
- **THEN** the system uses the same kcf `Renderer` outputs as preview
- **THEN** the system does not depend on an unversioned Mermaid.js bundle inside KatanA or the user's OS Chrome / Chromium app

### Requirement: PDF出力

現在開いているMarkdownをPDFファイルとしてエクスポートする。 The system SHALL conform. PDF 生成は kcf の export API を経由しなければならない（MUST）。

#### Scenario: PDF出力

- **WHEN** ユーザーがメニューから「PDFとして出力」を選択する
- **THEN** 保存先ダイアログが表示され、選択した場所にPDFが保存される
- **THEN** PDF 生成は kcf の export API を通じて行われる

#### Scenario: 外部ツール未インストール時

- **WHEN** PDF生成に必要な外部ツールがインストールされていない
- **THEN** kcf からの diagnostic を基に、エラーメッセージとインストールガイドが表示される

#### Scenario: PDF export records unsupported runtime gaps

- **WHEN** kcf が PDF export を `NotImplemented` として返す
- **THEN** KatanA は明示的に未対応である旨を UI に表示する
- **THEN** KatanA は OS Chrome / Chromium app への暗黙 fallback を持たない

### Requirement: 画像出力（PNG / JPG）

現在開いているMarkdownをPNGまたはJPG画像としてエクスポートする。 The system SHALL conform. PNG / JPG 生成は kcf の export API を経由しなければならない（MUST）。

#### Scenario: 画像エクスポート

- **WHEN** ユーザーがメニューから「PNGとして出力」または「JPGとして出力」を選択する
- **THEN** 保存先ダイアログが表示され、選択した場所に画像が保存される
- **THEN** 画像生成は kcf の export API を通じて行われる

#### Scenario: 長いドキュメントの画像出力

- **WHEN** ドキュメントがビューポートより長い
- **THEN** ドキュメント全体が1枚の縦長画像として出力される

#### Scenario: Image export records unsupported runtime gaps

- **WHEN** kcf が PNG / JPG export を `NotImplemented` として返す
- **THEN** KatanA は明示的に未対応である旨を UI に表示する
- **THEN** KatanA は OS Chrome / Chromium app への暗黙 fallback を持たない
