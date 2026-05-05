## ADDED Requirements

### Requirement: KatanA preview は Floem 実装で vello retained 描画を行わなければならない

システムは、Markdown / 画像 / 図表 preview を `katana-document-preview-floem` v0.1.0 経由で提供し、egui_commonmark の vendor パッチ依存と immediate mode 再描画コストを解消しなければならない（MUST）。

#### Scenario: katana-document-preview-floem を git dependency として参照する

- **WHEN** KatanA workspace を build する
- **THEN** `Cargo.toml` の workspace dependencies に `katana-document-preview-floem = { git = "...", tag = "v0.1.0" }` が含まれる
- **THEN** `katana-document-preview-egui` への依存は除去されている
- **THEN** `vendor/egui_commonmark/` ディレクトリと `[patch.crates-io]` の関連エントリは除去されている

#### Scenario: preview が vello で描画される

- **WHEN** ユーザーが Markdown ドキュメントを開く
- **THEN** preview は `katana-document-preview-floem` の vello scene 経由で描画される
- **THEN** 行間・マージンの調整は vendor パッチなしで実現できる

#### Scenario: 図表は kcf 経由で preview に表示される

- **WHEN** preview が Mermaid / Draw.io block を含む
- **THEN** 図表描画は kcf の `Renderer` trait 経由で取得した SVG を vello scene に組み込む
- **THEN** preview crate 内に独自 Mermaid / Draw.io 描画は含まれない

#### Scenario: カラー絵文字が preview で表示される

- **WHEN** preview に Apple Color Emoji 等のカラー絵文字が含まれる
- **THEN** 該当絵文字がカラーで描画される（cosmic-text の SBIX/CBTF 対応経由）
