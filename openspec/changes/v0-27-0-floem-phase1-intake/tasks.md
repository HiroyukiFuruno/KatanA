# Tasks: v0.27.0 Floem Phase 1 intake（editor + preview）— KatanA

> editor / preview の Floem 実装はそれぞれの外部 repo 側で行う。
> 本 tasks.md は KatanA 側の intake（-egui から -floem への impl crate 差し替え）のみを扱う。

## Branch Rule

`release/v0.27.0` ブランチを切って作業する。

---

## 準備完了条件（Definition of Ready）

- [ ] `katana-language-editor-floem` v0.1.0 release tag が切られていること
- [ ] `katana-document-preview-floem` v0.1.0 release tag が切られていること
- [ ] IME・カラー絵文字が Floem 実装で動作確認済みであること

---

## 1. impl crate を -egui から -floem に差し替える

- [ ] 1.1 root `Cargo.toml` を更新する
  ```toml
  # 追加
  katana-language-editor-floem = { git = "https://github.com/HiroyukiFuruno/katana-language-editor", tag = "v0.1.0" }
  katana-document-preview-floem = { git = "https://github.com/HiroyukiFuruno/katana-document-preview", tag = "v0.1.0" }
  # 除去
  # katana-language-editor-egui
  # katana-document-preview-egui
  ```
- [ ] 1.2 `cargo build` が通ること
- [ ] 1.3 `vendor/egui_commonmark/` パッチを除去し、`[patch.crates-io]` から関連エントリを削除する

---

## 2. 動作検証

- [ ] 2.1 日本語 IME 入力がエディタで正しく動作することを確認する
- [ ] 2.2 カラー絵文字がエディタ・preview 両方で表示されることを確認する
- [ ] 2.3 残る egui 依存（chat panel 等）を記録する（v0.28.0 Phase 2 の対象）
- [ ] 2.4 `just check` がエラーなし（exit code 0）で通過すること

---

## 3. commit

- [ ] 3.1 `release/v0.27.0` ブランチから PR を作成し master へ merge する
