# Tasks: v0.25.0 katana-language-editor-floem + katana-document-preview-floem intake — KatanA

> egui から Floem への移行 Phase 1（editor / preview）。
> 実装はそれぞれの外部 repo 側で行う。
> 本 tasks.md は KatanA 側の intake（impl crate 差し替え）のみを扱う。

## Branch Rule

`master` で直接作業する（バージョンブランチ不要）。

---

## 準備完了条件（Definition of Ready）

- [ ] `katana-language-editor-floem` v0.1.0 release tag が切られていること
- [ ] `katana-document-preview-floem` v0.1.0 release tag が切られていること
- [ ] IME・カラー絵文字が Floem 実装で動作確認済みであること

---

## 1. impl crate を -egui から -floem に差し替える

- [ ] 1.1 root `Cargo.toml` の以下を更新する
  ```toml
  # before
  katana-language-editor-egui = { git = "...", tag = "v0.1.0" }
  katana-document-preview-egui = { git = "...", tag = "v0.1.0" }
  # after
  katana-language-editor-floem = { git = "...", tag = "v0.1.0" }
  katana-document-preview-floem = { git = "...", tag = "v0.1.0" }
  ```
- [ ] 1.2 `cargo build` が通ることを確認する
- [ ] 1.3 `cargo tree` で egui / epaint が残っていないことを確認する（katana-chat-ui-egui が残る場合は許容）

---

## 2. eframe 依存の除去（可能な範囲で）

- [ ] 2.1 editor / preview が Floem に移った後、`katana-ui` の eframe 依存が減っていることを確認する
- [ ] 2.2 残る egui 依存（chat panel 等）を記録する（Phase 2 以降の対象）

---

## 3. 動作検証

- [ ] 3.1 日本語 IME 入力がエディタで正しく動作することを確認する
- [ ] 3.2 カラー絵文字がエディタ・preview 両方で表示されることを確認する
- [ ] 3.3 `just check` がエラーなし（exit code 0）で通過すること
- [ ] 3.4 commit & push（`master` 直接）
