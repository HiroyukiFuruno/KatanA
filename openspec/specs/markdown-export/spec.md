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

システムは、HTML / PDF / PNG / JPEG export に含まれる Mermaid / Draw.io 図形ブロックを、export 開始時点の KatanA テーマスナップショットで描画しなければならない（SHALL）。export thread は KDV / KRR 内部の既定テーマや、export 実行時に変化しうるグローバル状態に依存してはならない（MUST NOT）。

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

### Requirement: V8 を使う Markdown 出力依存関係はバージョン整合している

システムは、HTML / PDF / PNG / JPEG 出力（export）で KDV / KRR が利用する V8 を使う依存関係（V8-backed dependencies）を、作業領域（workspace）内とユーザーレビュー用の `scripts/screenshot` manifest 内で単一の互換 `v8` バージョンに揃えなければならない（MUST）。同じプロセス内の数式描画（MathJax）経路は V8 を初期化してはならない（MUST NOT）。出力は、`katana-document-viewer`、`katana-render-runtime`、または数式描画依存の不整合により停止してはならない（MUST NOT）。

#### Scenario: HTML 出力は整合した依存関係で図形ブロックを描画する

- **WHEN** Mermaid または Draw.io ブロックを含む Markdown 文書を HTML へ出力する
- **THEN** KDV は crates.io dependency として解決される
- **THEN** KRR は `katana-render-runtime = "0.3.3"` として解決される
- **THEN** KCF と KDR wrapper は依存関係グラフに含まれない
- **THEN** 数式描画依存は `v8` を要求しない

#### Scenario: ネイティブ出力でも図形描画を利用できる

- **WHEN** Mermaid または Draw.io ブロックを含む Markdown 文書を PDF / PNG / JPEG へ出力する
- **THEN** 出力経路は、作業領域で整合した V8-backed dependency graph を使う
- **THEN** V8 バージョン分裂に起因するワーカー切断の失敗（failure）により、図形描画が省略または置換されない

#### Scenario: 依存関係のずれはリリース前に検出される

- **WHEN** 将来のリリースで KDV、KRR、または作業領域の `v8` を更新する
- **THEN** リリース検証は、依存関係グラフに互換性のない V8 を使う描画器（V8-backed renderer）バージョンが含まれないことを確認する
- **THEN** Mermaid / Draw.io 出力の回帰テストは、リリース完了扱いの前に実行される

### Requirement: Markdown export は KDV 経由で行う

システムは、Markdown の HTML / PDF / PNG / JPEG export を `katana-canvas-forge`（KCF）ではなく、`katana-document-viewer`（KDV）経由で実行しなければならない（MUST）。

#### Scenario: KDV export adapter を使用する

- **WHEN** ユーザーが HTML、PDF、PNG、または JPEG export を実行する
- **THEN** KatanA は KDV の export 境界へ Markdown document、theme snapshot、出力形式、保存先を渡す
- **THEN** KatanA は KCF の export API、KCF DTO、または KCF adapter を呼び出さない

#### Scenario: KCF dependency が残らない

- **WHEN** KatanA v0.22.26 の release 検証で dependency graph を確認する
- **THEN** `katana-canvas-forge` は workspace dependencies と transitive dependencies に含まれない
- **THEN** export 経路の成功は kcf の存在に依存しない
