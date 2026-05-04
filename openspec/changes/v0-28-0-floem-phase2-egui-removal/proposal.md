## Why

v0.27.0 で editor と preview を Floem に切り替えた後、chrome（toolbar / sidebar / split pane / tab bar）を eframe / egui から Floem に完全移行し、egui・eframe への依存をゼロにする。

## DoR（Definition of Ready）

- v0.27.0（Floem Phase 1 intake）が完了していること
- Floem の taffy レイアウト上で KatanA の chrome 構造（sidebar、split pane、tab bar、toolbar）が再現できる見通しが立っていること
- 詳細設計は本 change 着手時に確定する（計画未定）

## What Changes

（着手時に詳細化する）

- eframe アプリループを Floem のウィンドウ・イベントループに完全置き換える
- toolbar / sidebar / split pane / tab bar を taffy + vello で実装する
- `Cargo.toml` から `egui`、`eframe`、`egui_*` 系依存を全て除去する

## Capabilities

### Modified Capabilities

- `ui-chrome`: eframe から Floem に完全移行する

## Impact

- 削除: `egui`、`eframe`、`egui_extras`、`eframe` 関連依存すべて
- 詳細は本 change 着手時に確定する
