## MODIFIED Requirements

### Requirement: AI provider abstraction の実装は katana-chat-ui に完全委譲しなければならない

システムは、AI provider abstraction の実装本体（Ollama 接続、provider registry、chat service、各種 DTO 生成 logic）を `katana-chat-ui` v0.1.0 に完全委譲し、KatanA 内 `crates/katana-core/src/ai/` 配下の実装本体を削除しなければならない（MUST）。

#### Scenario: katana-chat-ui v0.1.0 を git dependency として参照する

- **WHEN** KatanA workspace を build する
- **THEN** `Cargo.toml` の workspace dependencies に以下が含まれる:
  - `katana-acp-client = { git = "https://github.com/HiroyukiFuruno/katana-chat-ui", tag = "v0.1.0" }`
  - `katana-chat-ui = { git = "https://github.com/HiroyukiFuruno/katana-chat-ui", tag = "v0.1.0" }`
  - `katana-chat-ui-egui = { git = "https://github.com/HiroyukiFuruno/katana-chat-ui", tag = "v0.1.0" }`
- **THEN** `katana-chat-ui`（neutral state crate）の `cargo tree` に `egui` が含まれない

#### Scenario: KatanA 側 ai module の実装本体を削除する

- **WHEN** KatanA `crates/katana-core/src/ai/` を確認する
- **THEN** `OllamaProvider` 実装本体、provider registry の独自実装、chat service の独自実装は削除されている
- **THEN** 残るのは `katana-acp-client` / `katana-chat-ui` の re-export のみ
- **THEN** KatanA 内に vendor SDK の直接 import は含まれない

#### Scenario: chat サイドパネルを katana-chat-ui-egui の widget に差し替える

- **WHEN** KatanA UI が chat サイドパネルを表示する
- **THEN** `katana-chat-ui-egui` の `ChatPanelWidget::show(ui, ...)` を呼ぶ薄い adapter のみが KatanA 側に残る
- **THEN** chat 状態（message 履歴、pending request、streaming buffer）は `katana-chat-ui`（neutral state crate）が管理する

#### Scenario: DocumentContext を KatanA の document 状態と接続する

- **WHEN** KatanA UI が chat / autofix request を組み立てる
- **THEN** `DocumentContext` は KatanA の現在 active document の URI / 内容 / カーソル位置 / diagnostics から構築される
- **THEN** 構築された `DocumentContext` は `katana-chat-ui` の widget に渡される
