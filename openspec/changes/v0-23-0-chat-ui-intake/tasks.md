# Tasks: v0.23.0 katana-chat-ui v0.0.1 intake

> katana-chat-ui v0.0.1（ACP client foundation、egui 非依存）を KatanA に取り込む。  
> chat widget・autofix diff surface は v0.24.0 の責務。  
> katana-chat-ui 側の実装タスクは [katana-chat-ui openspec](https://github.com/HiroyukiFuruno/katana-chat-ui) を参照。

## Branch Rule

`release/v0.23.0` ブランチを切って作業する。

---

## 準備完了条件（Definition of Ready）

- [ ] `katana-chat-ui` v0.0.1 release tag が切られていること
- [ ] `katana-acp-client` crate の `AiProvider` trait・`DocumentContext`・`OllamaProvider`・DTO が確定していること

---

## 1. git dependency を追加する

- [ ] 1.1 root `Cargo.toml` に `katana-acp-client = { git = "https://github.com/HiroyukiFuruno/katana-chat-ui", tag = "v0.0.1" }` を追加する
- [ ] 1.2 `cargo build` が通ること
- [ ] 1.3 `cargo tree` で `katana-acp-client` に `egui` が含まれないことを確認する

---

## 2. katana-core の ai module を katana-acp-client に切り替える

- [ ] 2.1 `katana-core/src/ai/mod.rs` の `AiProvider` / `AiProviderRegistry` を `katana-acp-client` の型に差し替える
- [ ] 2.2 `katana-core/src/ai/mod.rs` の独自実装を削除する（`katana-acp-client` の re-export に置き換える）
- [ ] 2.3 `DocumentContext` を `katana-acp-client` の型として使えることを確認する
- [ ] 2.4 `cargo test` がエラーなしで通ること

---

## 3. 確認と commit

- [ ] 3.1 `just check` がエラーなし（exit code 0）で通過すること
- [ ] 3.2 `release/v0.23.0` ブランチから PR を作成し master へ merge する
