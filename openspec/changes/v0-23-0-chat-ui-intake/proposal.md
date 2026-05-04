## Why

`katana-chat-ui` リポジトリで v0.1.0 の開発が完了した段階で、KatanA は chat 機能の実装を `katana-chat-ui` に完全委譲する。現在 `crates/katana-core/src/ai/` に残存している AI provider 実装・chat ロジックは KatanA の責務外であり、intake と同時に除去する。

## What Changes

- `Cargo.toml` に `katana-chat-ui`（neutral interface crate）を git dependency として追加する
- `katana-chat-ui-egui` を git dependency として追加し、egui chat widget を KatanA UI に組み込む
- `crates/katana-core/src/ai/` の実装本体を削除する（Ollama adapter、provider 抽象、chat service）
- KatanA UI の chat サイドパネルを `katana-chat-ui-egui` の widget に切り替える
- `katana-chat-ui` の `DocumentContext` を KatanA の現在 document 状態と接続する

## Capabilities

### New Capabilities

- `chat-ui`: katana-chat-ui 経由の chat サイドパネル（Ollama provider、ACP client）

### Modified Capabilities

- `ai-provider-abstraction`: KatanA 側実装を削除し、katana-chat-ui 側の interface に一本化する

## Impact

- 削除: `crates/katana-core/src/ai/` 一式
- 追加: `Cargo.toml` に `katana-chat-ui`、`katana-chat-ui-egui` の git dependency
- 追加: KatanA UI 側の chat panel を katana-chat-ui-egui widget に差し替える薄い adapter
- `v0.24.0`（kcf intake）・`v0.25.0`（Floem Phase 1 intake）には影響しない独立した intake
