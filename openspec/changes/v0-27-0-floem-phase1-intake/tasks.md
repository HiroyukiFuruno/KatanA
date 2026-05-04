# Tasks: v0.27.0 Floem Phase 1 intake（editor + chat input）— KatanA

> editor / chat-input の Floem 実装はそれぞれの外部 repo 側で行う。
> 本 tasks.md は KatanA 側の intake（-egui から -floem への impl crate 差し替え）のみを扱う。

## Branch Rule

`release/v0.27.0` ブランチを切って作業する。

---

## 準備完了条件（Definition of Ready）

- [ ] `katana-language-editor-floem` v0.1.0 release tag が切られていること
- [ ] `katana-chat-ui-floem` v0.1.0 release tag が切られていること
- [ ] IME・カラー絵文字が editor / chat input 両方の Floem 実装で動作確認済みであること

---

## 1. impl crate を -egui から -floem に差し替える

- [ ] 1.1 root `Cargo.toml` を更新する
  ```toml
  # 追加
  katana-language-editor-floem = { git = "https://github.com/HiroyukiFuruno/katana-language-editor", tag = "v0.1.0" }
  katana-chat-ui-floem = { git = "https://github.com/HiroyukiFuruno/katana-chat-ui", tag = "v0.1.0" }
  # 除去
  # katana-language-editor-egui
  # katana-chat-ui-egui
  ```
- [ ] 1.2 `cargo build` が通ること
- [ ] 1.3 KatanA shell から Floem widget を `paint_callback` 経由で eframe と共存させる初期統合を行う

---

## 2. 動作検証

- [ ] 2.1 日本語 IME 入力が editor で正しく動作することを確認する
- [ ] 2.2 日本語 IME 入力が chat input で正しく動作することを確認する
- [ ] 2.3 カラー絵文字が editor / chat input 両方で表示されることを確認する
- [ ] 2.4 残る egui 依存（preview / chrome）を記録する（v0.28.0 / v0.29.0 の対象）
- [ ] 2.5 `just check` がエラーなし（exit code 0）で通過すること

---

## 3. commit

- [ ] 3.1 `release/v0.27.0` ブランチから PR を作成し master へ merge する
