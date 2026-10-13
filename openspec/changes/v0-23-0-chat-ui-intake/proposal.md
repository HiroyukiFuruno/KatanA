## Why

`katana-chat-ui` リポジトリの v0.0.1（`katana-acp-client` neutral interface crate、egui ゼロ）を KatanA に最初に取り込む段階。AI provider abstraction の実体を KatanA 内に閉じたまま漸進的に責務委譲を進めるため、本 change では neutral interface のみを intake し、widget 接続と `katana-core/src/ai/` の完全削除は v0.24.0（katana-chat-ui v0.1.0 intake）の責務として明確に分離する。

## What Changes

- `Cargo.toml` に `katana-acp-client = { git = "...", tag = "v0.0.1" }` を git dependency として追加する
- `crates/katana-core/src/ai/` の独自定義を `katana-acp-client` の型（`AiProvider` trait、`AiProviderRegistry`、`DocumentContext`、`AiIntent`、`AiRequest`、`AiResponse`、`OllamaProvider`）の re-export に置き換える
- KatanA 側の `ai/` module の実装本体は **削除しない**。型定義だけを `katana-acp-client` に肩代わりさせる
- chat panel の widget 差し替え、`ai/` module の完全削除、`DocumentContext` の document 状態接続は v0.24.0 の責務とする

## Capabilities

### Modified Capabilities

- `ai-provider-abstraction`: KatanA 内の独自定義を `katana-acp-client` の neutral interface に切り替える（実装は KatanA 内のまま）

## Impact

- 追加: `Cargo.toml` に `katana-acp-client` v0.0.1 の git dependency
- 変更: `crates/katana-core/src/ai/mod.rs` を `katana-acp-client` の re-export に書き換える
- 変更なし: KatanA 内の Ollama 実装本体（v0.24.0 で widget 接続と同時に削除する）
- 変更なし: KatanA UI の chat サイドパネル（v0.24.0 で widget 差し替え）
- katana-chat-ui 側の実装は [katana-chat-ui openspec](https://github.com/HiroyukiFuruno/katana-chat-ui) を参照
