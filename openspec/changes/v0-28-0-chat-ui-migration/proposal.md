## Why

KatanA v0.23.0 で `katana-core/src/ai/` と `katana-ui/src/app/chat.rs` に直接実装した AI / chat / autofix は技術的負債。KatanA がドキュメントレビューシェルとして薄くあるべきなのに、LLM 実装の詳細を抱え込んでいる。

これを解消するため、chat UI と ACP client を `katana-chat-ui` 外部リポジトリ（v0.1.0）として切り出し、KatanA v0.28.0 はこれを git dependency として取り込むだけにする。

## What Changes

- KatanA から以下を除去する：
  - `crates/katana-core/src/ai/`（Ollama provider、registry、types）
  - `crates/katana-ui/src/app/chat.rs`、`autofix.rs`、`autofix_request.rs`、`autofix_support.rs`
  - `crates/katana-ui/src/state/autofix.rs`、`diff_preview.rs`、`chat.rs`
  - `crates/katana-platform/src/settings/types/ai.rs`
- 代わりに `katana-chat-ui` v0.1.0 の `ChatPanel::show()` と autofix diff surface API を呼ぶ薄い adapter だけを残す。
- AI settings（Ollama endpoint、モデル選択）は `katana-chat-ui` の settings schema に委譲する。

## Capabilities

### Removed from KatanA

- `ai-provider-registry`（katana-core 内の直接実装）
- `ollama-provider`（katana-core 内の直接実装）
- `chat-ui-internal`（katana-ui 内の直接実装）
- `autofix-internal`（katana-ui 内の直接実装）

### Delegated to katana-chat-ui

- `llm-agent-protocol`：ACP client、Ollama adapter、capability negotiation
- `chat-ui-component`：chat side-panel widget、streaming 表示、disabled state
- `autofix-diff-surface`：KML diagnostics + LLM 提案の diff preview / confirm / apply

## Impact

- `crates/katana-core/src/ai/` — 除去
- `crates/katana-ui/src/app/chat*.rs`、`autofix*.rs` — 除去
- `crates/katana-ui/src/state/chat.rs`、`autofix.rs`、`diff_preview.rs` — 除去
- `crates/katana-platform/src/settings/types/ai.rs` — 除去
- root `Cargo.toml` — `katana-chat-ui` git dependency 追加
