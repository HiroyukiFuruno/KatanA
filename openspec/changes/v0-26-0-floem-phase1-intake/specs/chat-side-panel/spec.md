## MODIFIED Requirements

### Requirement: chat 入力サーフェスは Floem 実装で IME / カラー絵文字を解決しなければならない

システムは、chat サイドパネルの入力サーフェスを `katana-chat-ui-floem` v0.1.0 経由で提供し、chat 入力時の IME composition 不完全とカラー絵文字非対応を解消しなければならない（MUST）。

#### Scenario: katana-chat-ui-floem を git dependency として参照する

- **WHEN** KatanA workspace を build する
- **THEN** `Cargo.toml` の workspace dependencies に `katana-chat-ui-floem = { git = "https://github.com/HiroyukiFuruno/katana-chat-ui", tag = "v0.1.0" }` が含まれる
- **THEN** `katana-chat-ui-egui` への依存は除去されている

#### Scenario: chat 入力で日本語 IME が壊れない

- **WHEN** ユーザーが chat 入力で日本語を入力する
- **THEN** 確定前のインライン composition が正しく表示される
- **THEN** 確定文字列が chat composer に反映される

#### Scenario: chat 入力でカラー絵文字が表示される

- **WHEN** chat composer または chat history にカラー絵文字が含まれる
- **THEN** 該当絵文字がカラーで描画される
