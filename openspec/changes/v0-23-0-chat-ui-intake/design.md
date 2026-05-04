## Context

KatanA `release/v0.23.0` ブランチで先行実装した `crates/katana-core/src/ai/` には、`AiProvider` trait・`AiProviderRegistry`・Ollama adapter・各種 DTO（`AiRequest` / `AiResponse` / `AiCapabilities` / `AiModel`）が存在する。これらは将来 `katana-chat-ui` repo に完全移管する予定。

本 change は移管の最初の一歩として、KatanA が内部定義していた型を `katana-acp-client` v0.0.1 の neutral interface へ切り替える。これにより v0.24.0 で chat widget（`katana-chat-ui-egui`）を取り込む際、既に共有 interface 上に乗っている状態を作る。

## Goals / Non-Goals

**Goals:**

- KatanA `ai/` module の型定義を `katana-acp-client` v0.0.1 の re-export に置き換える
- `katana-acp-client` が egui ゼロであることを `cargo tree` で検証する
- 既存挙動（Ollama 接続・chat UI・autofix）はそのまま動作することを `cargo test` で確認する
- 後続 v0.24.0 が widget 差し替えと `ai/` 完全削除に集中できる土台を作る

**Non-Goals:**

- chat panel の widget 差し替え（v0.24.0 の責務）
- `katana-core/src/ai/` の実装本体削除（v0.24.0 の責務）
- `DocumentContext` を KatanA の document 状態と接続する作業（v0.24.0 の責務）
- Vertex AI / Bedrock / OpenAI provider の追加（後続 milestone）

## Decisions

### 1. neutral interface のみを intake し、実装本体は手元に残す

`katana-acp-client` v0.0.1 は egui を含まない pure な interface crate である。ここでは型定義の置換に絞り、KatanA 内の Ollama 実装本体は v0.24.0 まで残す。

- 採用理由:
  - intake と widget 差し替えを 1 release に詰めると変更範囲が広がり、回帰検出が難しくなる
  - 段階的に置き換えれば、各段階で `cargo test` が通る状態を維持できる
- 代替案:
  - 一括で widget 差し替えと `ai/` 削除も行う: 検証範囲が広がるため不採用

### 2. type alias / re-export で型を入れ替える

`crates/katana-core/src/ai/mod.rs` の独自定義を削除し、`pub use katana_acp_client::{...}` で re-export に切り替える。シンボル名と signature は v0.0.1 設計で互換性を持たせている前提。

- 採用理由:
  - 呼び出し側コードは変更不要
  - 型互換性が保たれていることが `cargo build` で即時検証できる
- 代替案:
  - 呼び出し側を全て import path 変更: 変更範囲が広がるため不採用

## Risks / Trade-offs

- **[Risk]** `katana-acp-client` v0.0.1 の型シグネチャと KatanA 内独自定義の差異
  -> Mitigation: `katana-acp-client` 側で KatanA の型を参考に設計済み。差異がある場合は `katana-acp-client` v0.0.2 で吸収する

- **[Risk]** re-export 切り替えだけでは `katana-acp-client` の型が KatanA の他 module から正しく見えない
  -> Mitigation: `cargo build` と `cargo test` を全件通過する gate にする

## Migration Plan

1. `Cargo.toml` に `katana-acp-client = { git = "...", tag = "v0.0.1" }` を追加
2. `crates/katana-core/src/ai/mod.rs` の型定義を `pub use katana_acp_client::{...}` に置き換える
3. `cargo build` / `cargo test` を通過させる
4. `cargo tree` で `katana-acp-client` に `egui` が含まれないことを確認する

## Verification

- `cargo build` がエラーなしで通過する
- `cargo test` の既存 test が全て通過する
- `cargo tree -p katana-acp-client | grep -i egui` が空である
- `cargo tree -p katana-core | grep katana-acp-client` で依存関係が確立されている
