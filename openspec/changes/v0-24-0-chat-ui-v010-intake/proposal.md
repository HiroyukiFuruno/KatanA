## Why

`katana-chat-ui` v0.1.0 で chat state 管理（neutral）と egui rendering（`katana-chat-ui-egui`）が完成した段階で KatanA に取り込む。v0.23.0 で確立した ACP plumbing の上に、ユーザーが操作できる chat サイドパネルと lint autofix diff surface を追加する。

## What Changes

- `Cargo.toml` に `katana-chat-ui`（neutral、v0.1.0）と `katana-chat-ui-egui`（egui impl、v0.1.0）を git dependency として追加する
- `katana-acp-client` の version を v0.0.1 → v0.1.0 に bump する
- KatanA UI の chat サイドパネルを `katana-chat-ui-egui` の `ChatPanelWidget::show()` 呼び出しに切り替える（薄い adapter のみ残す）
- KML diagnostics と chat の autofix diff surface を KatanA の diagnostics パネルに接続する
- `DocumentContext` を KatanA の現在 document 状態から構築して `katana-chat-ui` に渡す
- `crates/katana-core/src/ai/` の **実装本体を完全削除**する（v0.23.0 で re-export に切り替え済みのため、残るは `katana-chat-ui` の re-export のみ）

## Capabilities

### New Capabilities

- `chat-side-panel`: chat サイドパネル（Ollama provider、streaming 表示、disabled state）
- `lint-autofix-diff-surface`: KML diagnostics + LLM 提案の diff preview / confirm / apply

### Modified Capabilities

- `ai-provider-abstraction`: KatanA 内実装本体を削除し、`katana-chat-ui` widget 経由の利用に完全移行する

## Impact

- 追加: `Cargo.toml` に `katana-chat-ui` v0.1.0、`katana-chat-ui-egui` v0.1.0 の git dependency
- 変更: `katana-acp-client` の tag を v0.0.1 → v0.1.0 へ bump
- 追加: KatanA UI 側の薄い adapter（`DocumentContext` 構築 + `ChatPanelWidget` 呼び出し）
- 削除: `crates/katana-core/src/ai/` 配下の Ollama 実装本体・provider registry の独自実装・chat service の独自実装
- katana-chat-ui 側の実装は [katana-chat-ui openspec](https://github.com/HiroyukiFuruno/katana-chat-ui) を参照
