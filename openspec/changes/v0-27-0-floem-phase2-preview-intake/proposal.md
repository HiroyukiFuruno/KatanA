## Why

v0.26.0（Phase 1: editor + chat input の Floem 化）完了後、preview 層を Floem 実装に切り替える。preview の vello retained 描画により、egui_commonmark の vendor パッチ依存と immediate mode 再描画コストが根本解決する。本 change（Phase 2）は preview のみを差し替え、chrome（toolbar / sidebar / split pane / tab bar / window loop）は v0.28.0（Phase 3）で扱う。

## DoR（Definition of Ready）

- v0.26.0（Floem Phase 1: editor + chat input intake）が完了していること
- `katana-document-preview-floem` v0.1.0 が release 済みであること
- vello で Markdown / 画像 / 図表が retained 描画できることが確認済みであること

## What Changes

- `Cargo.toml` に `katana-document-preview-floem` v0.1.0 を git dependency として追加し、`katana-document-preview-egui` への依存を除去する
- KatanA preview pane を Floem widget に切り替える
- `vendor/egui_commonmark/` パッチを除去する（preview 移行で不要になる）
- chrome（toolbar / sidebar / split pane / tab bar / window loop）は本 change では egui / eframe のままにする（v0.28.0 で扱う）

## Capabilities

### Modified Capabilities

- `document-preview`: `katana-document-preview-floem` 経由の vello retained 描画に切り替える

## Impact

- 追加: `Cargo.toml` に `katana-document-preview-floem` v0.1.0 の git dependency
- 削除: `katana-document-preview-egui` への依存
- 削除: `vendor/egui_commonmark/` パッチおよび関連 `[patch.crates-io]` エントリ
- 変更なし: chrome は egui / eframe のまま（v0.28.0 で対応）
