## Why

v0.26.0（Phase 1: editor + chat input）と v0.27.0（Phase 2: preview）で入力サーフェスと描画層を Floem に切り替えた後、本 change（Phase 3）は KatanA chrome（toolbar / sidebar / split pane / tab bar / window loop）を eframe / egui から Floem に完全移行し、`egui` / `eframe` / `egui_extras` / `egui_*` 系依存をゼロにする。

## DoR（Definition of Ready）

- v0.26.0（Phase 1）が完了している
- v0.27.0（Phase 2）が完了している
- Floem の taffy レイアウト上で KatanA の chrome 構造（sidebar、split pane、tab bar、toolbar）が再現できる見通しが立っている
- 詳細設計は本 change 着手時に確定する（計画未定）

## What Changes

（着手時に詳細化する）

- eframe アプリループを Floem のウィンドウ・イベントループ（winit 直接利用）に完全置き換える
- toolbar / sidebar / split pane / tab bar を taffy + vello で実装する
- `Cargo.toml` から `egui`、`eframe`、`egui_extras`、`egui_*` 系依存を全て除去する

## Capabilities

### New Capabilities

- `ui-chrome`: KatanA chrome（toolbar / sidebar / split pane / tab bar / window loop）を Floem ベースで実装し、egui / eframe ゼロを保証する

## Impact

- 削除: `egui`、`eframe`、`egui_extras`、`egui_*` 系の依存すべて
- 詳細は本 change 着手時に確定する
