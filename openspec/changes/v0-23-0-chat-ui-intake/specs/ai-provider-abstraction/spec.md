## MODIFIED Requirements

### Requirement: AI provider abstraction は katana-acp-client neutral interface を経由しなければならない

システムは、AI provider abstraction の型定義（`AiProvider` trait、`AiProviderRegistry`、`DocumentContext`、`AiIntent`、`AiRequest` / `AiResponse` DTO）を KatanA 内独自定義ではなく、`katana-acp-client` v0.0.1 の neutral interface crate からの re-export として提供しなければならない（MUST）。

#### Scenario: katana-acp-client を git dependency として参照する

- **WHEN** KatanA workspace を build する
- **THEN** `Cargo.toml` の workspace dependencies に `katana-acp-client = { git = "https://github.com/HiroyukiFuruno/katana-chat-ui", tag = "v0.0.1" }` が含まれる
- **THEN** `katana-acp-client` の `cargo tree` に `egui` が含まれない

#### Scenario: ai module を re-export に切り替える

- **WHEN** KatanA `crates/katana-core/src/ai/mod.rs` を読む
- **THEN** `AiProvider` / `AiProviderRegistry` / `DocumentContext` / `AiIntent` / `AiRequest` / `AiResponse` / `OllamaProvider` は `pub use katana_acp_client::{...}` で re-export されている
- **THEN** KatanA 内に上記型の独自定義は残らない（実装本体は v0.24.0 で削除する）

#### Scenario: 既存挙動が回帰しない

- **WHEN** type re-export 切り替え後に `cargo test` を実行する
- **THEN** Ollama 接続・chat UI 表示・autofix request の既存挙動は変わらず通過する
- **THEN** chat panel の widget 差し替えと `ai/` module の実装本体削除は v0.24.0 まで実施しない
