## Why

`katana-chat-ui` v0.1.0 で chat state 管理（neutral）と egui rendering（`katana-chat-ui-egui`）が完成した段階で KatanA に取り込む。v0.23.0 で確立した ACP plumbing の上に、ユーザーが操作できる chat サイドパネルと lint autofix diff surface を追加する。

## What Changes

- `Cargo.toml` に `katana-chat-ui`（neutral、v0.1.0）と `katana-chat-ui-egui`（egui impl、v0.1.0）を git dependency として追加する
- KatanA UI に chat サイドパネルを追加する（`katana-chat-ui-egui` の `ChatPanelWidget::show()` を呼ぶ薄い adapter のみ）
- KML diagnostics と chat の autofix diff surface を KatanA の diagnostics パネルに接続する
- `DocumentContext` を KatanA の現在 document 状態から構築して `katana-chat-ui` に渡す

## Capabilities

### New Capabilities

- `chat-side-panel`: chat サイドパネル（Ollama provider、streaming 表示、disabled state）
- `lint-autofix-diff-surface`: KML diagnostics + LLM 提案の diff preview / confirm / apply

## Impact

- 追加: `Cargo.toml` に `katana-chat-ui` v0.1.0、`katana-chat-ui-egui` v0.1.0 の git dependency
- 追加: KatanA UI 側の薄い adapter（`DocumentContext` 構築 + `ChatPanelWidget` 呼び出し）
- 変更なし: `katana-core/src/ai/` neutral interface は維持する
- katana-chat-ui 側の実装は [katana-chat-ui openspec](https://github.com/HiroyukiFuruno/katana-chat-ui) を参照
