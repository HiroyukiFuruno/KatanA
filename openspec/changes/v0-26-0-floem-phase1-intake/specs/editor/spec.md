## ADDED Requirements

### Requirement: KatanA editor は Floem 実装で IME / カラー絵文字を解決しなければならない

システムは、editor 入力 surface を `katana-language-editor-floem` v0.1.0 経由で提供し、egui TextEdit に起因する IME composition 不完全とカラー絵文字非対応を解消しなければならない（MUST）。

#### Scenario: katana-language-editor-floem を git dependency として参照する

- **WHEN** KatanA workspace を build する
- **THEN** `Cargo.toml` の workspace dependencies に `katana-language-editor-floem = { git = "...", tag = "v0.1.0" }` が含まれる
- **THEN** `katana-language-editor-egui` への依存は除去されている

#### Scenario: 日本語 IME composition が壊れない

- **WHEN** ユーザーが editor で日本語を入力する
- **THEN** 確定前のインライン composition が正しく表示される
- **THEN** 確定文字列が editor buffer に反映される

#### Scenario: カラー絵文字が表示される

- **WHEN** editor 上に Apple Color Emoji（SBIX）等のカラー絵文字が含まれる
- **THEN** 該当絵文字がカラーで描画される
- **THEN** モノクロ・欠落・置換無しに正しく表示される
