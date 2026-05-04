# Tasks: v0.28.0 Chat UI migration — KatanA intake

> AI / chat / autofix 実装はすべて `katana-chat-ui` repo 側で行う。
> katana-chat-ui 側の実装タスクは [katana-chat-ui openspec](https://github.com/HiroyukiFuruno/katana-chat-ui) を参照。
> 本 tasks.md は KatanA 側の intake（git dependency 追加 + 既存実装除去 + ChatPanel API 呼び出しへの差し替え）のみを扱う。

## Branch Rule

interface 整理リファクタリングとして `master` で直接作業する（バージョンブランチ不要）。

---

## 準備完了条件（Definition of Ready）

- [ ] `katana-chat-ui` `v0.1.0` release tag が切られていること
- [ ] `ChatPanel::show(ui, context, config)` API が確定していること
- [ ] autofix diff surface API が確定していること
- [ ] `katana-chat-ui`（neutral interface）が egui を含まないことを確認していること

---

## 1. katana-chat-ui を git dependency として追加する

- [ ] 1.1 root `Cargo.toml` の workspace dependencies に追加する
  ```toml
  katana-acp-client = { git = "https://github.com/HiroyukiFuruno/katana-chat-ui", tag = "v0.1.0" }
  katana-chat-ui = { git = "https://github.com/HiroyukiFuruno/katana-chat-ui", tag = "v0.1.0" }
  ```
- [ ] 1.2 `cargo build` が通ることを確認する

---

## 2. KatanA 側 chat / autofix を ChatPanel API 経由に切り替える

### 準備完了条件

- [ ] Task 1 完了

- [ ] 2.1 `katana-ui` の chat side-panel を `ChatPanel::show()` 呼び出しに差し替える
- [ ] 2.2 autofix diff surface を `katana-chat-ui` の reusable widget 呼び出しに差し替える
- [ ] 2.3 AI settings（Ollama endpoint、モデル選択）を `katana-chat-ui` の settings schema 経由に移す

---

## 3. KatanA 側の AI / chat / autofix 実装を除去する

### 準備完了条件

- [ ] Task 2 完了

- [ ] 3.1 `crates/katana-core/src/ai/` を除去する（`ollama.rs`、`registry.rs`、`types.rs`）
- [ ] 3.2 `crates/katana-ui/src/app/chat.rs`、`autofix.rs`、`autofix_request.rs`、`autofix_support.rs` を除去する
- [ ] 3.3 `crates/katana-ui/src/state/autofix.rs`、`diff_preview.rs`、`chat.rs` を除去する
- [ ] 3.4 `crates/katana-platform/src/settings/types/ai.rs` を除去する
- [ ] 3.5 `git grep OllamaProvider` / `git grep AiProvider` で KatanA 内に直接参照が残っていないことを確認する

---

## 4. 検証と commit

- [ ] 4.1 `just check` がエラーなし（exit code 0）で通過すること
- [ ] 4.2 `./scripts/openspec validate v0-28-0-chat-ui-migration --strict` を実行し OpenSpec の整合性を確認する
- [ ] 4.3 commit & push（`master` 直接）
