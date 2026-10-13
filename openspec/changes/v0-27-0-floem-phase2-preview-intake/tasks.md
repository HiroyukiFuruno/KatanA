# Tasks: v0.27.0 Floem Phase 2 intake（preview）— KatanA

> preview の Floem 実装は `katana-document-preview` repo 側で行う。
> 本 tasks.md は KatanA 側の intake（-egui から -floem への impl crate 差し替え）のみを扱う。

## Branch Rule

`release/v0.27.0` ブランチを切って作業する。

---

## 準備完了条件（Definition of Ready）

- [ ] v0.26.0（Phase 1: editor + chat input intake）が完了している
- [ ] `katana-document-preview-floem` v0.1.0 release tag が切られていること
- [ ] vello で Markdown / 画像 / 図表が retained 描画できることを確認済み

---

## 1. impl crate を -egui から -floem に差し替える

- [ ] 1.1 root `Cargo.toml` を更新する
  ```toml
  # 追加
  katana-document-preview-floem = { git = "https://github.com/HiroyukiFuruno/katana-document-preview", tag = "v0.1.0" }
  # 除去
  # katana-document-preview-egui
  ```
- [ ] 1.2 `vendor/egui_commonmark/` ディレクトリを除去する
- [ ] 1.3 root `Cargo.toml` の `[patch.crates-io]` から `egui_commonmark` 関連エントリを除去する
- [ ] 1.4 `cargo build` が通ること

---

## 2. 動作検証

- [ ] 2.1 Markdown preview が Floem 経由で描画されることを確認する
- [ ] 2.2 図表（Mermaid / Draw.io）が KDV / KRR 経由で preview に表示されることを確認する
- [ ] 2.3 残る egui 依存（chrome のみ）を記録する（v0.28.0 の対象）
- [ ] 2.4 `just check` がエラーなし（exit code 0）で通過すること

---

## 3. commit

- [ ] 3.1 `release/v0.27.0` ブランチから PR を作成し master へ merge する
