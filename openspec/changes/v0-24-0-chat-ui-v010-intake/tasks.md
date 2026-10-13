# Tasks: v0.24.0 katana-chat-ui v0.1.0 intake — KatanA

> chat state 管理・egui rendering はすべて `katana-chat-ui` repo 側で行う。
> katana-chat-ui 側の実装タスクは [katana-chat-ui openspec](https://github.com/HiroyukiFuruno/katana-chat-ui) を参照。
> 本 tasks.md は KatanA 側の intake（git dependency 追加 + chat panel 接続）のみを扱う。

## Branch Rule

`release/v0.24.0` ブランチを切って作業する。

---

## 準備完了条件（Definition of Ready）

- [ ] `katana-chat-ui` v0.1.0 release tag が切られていること
- [ ] `katana-chat-ui-egui` の `ChatPanelWidget::show()` API が確定していること
- [ ] autofix diff surface API が確定していること

---

## 1. git dependency を追加・bump する

- [ ] 1.1 root `Cargo.toml` の `katana-acp-client` の tag を v0.0.1 → v0.1.0 に bump する
- [ ] 1.2 root `Cargo.toml` に以下を追加する
  ```toml
  katana-chat-ui = { git = "https://github.com/HiroyukiFuruno/katana-chat-ui", tag = "v0.1.0" }
  katana-chat-ui-egui = { git = "https://github.com/HiroyukiFuruno/katana-chat-ui", tag = "v0.1.0" }
  ```
- [ ] 1.3 `cargo build` が通ること
- [ ] 1.4 `cargo tree` で `katana-chat-ui`（neutral）に `egui` が含まれないことを確認する

---

## 2. chat サイドパネルを KatanA UI に接続する

### 準備完了条件

- [ ] Task 1 完了

- [ ] 2.1 `katana-ui` の chat サイドパネルを `ChatPanelWidget::show()` 呼び出しに差し替える
- [ ] 2.2 `DocumentContext` を現在 document 状態から構築して渡す
- [ ] 2.3 autofix diff surface を `katana-chat-ui` の widget 呼び出しに差し替える
- [ ] 2.4 AI settings（Ollama endpoint・モデル選択）を `katana-chat-ui` の settings schema 経由に移す

---

## 3. KatanA 側 ai/ 実装本体を削除する

### 準備完了条件

- [ ] Task 2 完了

- [ ] 3.1 `crates/katana-core/src/ai/` 配下の Ollama 実装本体を削除する
- [ ] 3.2 `crates/katana-core/src/ai/` 配下の provider registry 独自実装を削除する
- [ ] 3.3 `crates/katana-core/src/ai/` 配下の chat service 独自実装を削除する
- [ ] 3.4 `crates/katana-core/src/ai/mod.rs` には `katana-chat-ui` / `katana-acp-client` の re-export のみが残ることを確認する
- [ ] 3.5 `git grep` で KatanA 内に Ollama / vendor SDK の直接 import が残っていないことを確認する
- [ ] 3.6 `cargo test` が通過すること

---

## 4. 検証と commit

- [ ] 4.1 `just check` がエラーなし（exit code 0）で通過すること
- [ ] 4.2 `release/v0.24.0` ブランチから PR を作成し master へ merge する
