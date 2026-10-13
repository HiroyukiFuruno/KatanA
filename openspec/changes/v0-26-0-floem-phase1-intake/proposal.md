## Why

egui の IME 破損・カラー絵文字欠如・immediate mode CPU コストは、ユーザーが**入力時に最初に触れる痛み**である。Floem 移行は最優先の **入力サーフェス（editor + chat input）** から段階的に進める。本 change（Phase 1）では editor と chat 入力を Floem impl crate に差し替える。preview と chrome は後続 Phase（v0.27.0 / v0.28.0）で扱う。

検証範囲を絞るため、Floem 移行を 3 phase に分割する:

- **v0.26.0（本 change） = Phase 1: editor + chat input** ← 入力 IME / 絵文字を最優先で解決
- v0.27.0 = Phase 2: preview（vello retained 描画）
- v0.28.0 = Phase 3: chrome / eframe 完全除去

## What Changes

- `Cargo.toml` に `katana-language-editor-floem` v0.1.0 を git dependency として追加し、`katana-language-editor-egui` への依存を除去する
- `Cargo.toml` に `katana-chat-ui-floem` v0.1.0 を git dependency として追加し、`katana-chat-ui-egui` への依存を除去する
- KatanA shell から editor / chat input の Floem widget を `paint_callback` 経由で eframe と共存させる
- preview は本 change では egui のままにする（v0.27.0 で扱う）
- chrome / toolbar / sidebar / split pane は本 change では egui / eframe のままにする（v0.28.0 で扱う）

## Capabilities

### Modified Capabilities

- `editor`: `katana-language-editor-floem` 経由に切り替え、IME / カラー絵文字の egui 制約を解消する
- `chat-side-panel`: `katana-chat-ui-floem` 経由に切り替え、chat 入力 IME / カラー絵文字の egui 制約を解消する

## Impact

- 追加: `Cargo.toml` に `katana-language-editor-floem` v0.1.0、`katana-chat-ui-floem` v0.1.0 の git dependency
- 削除: `katana-language-editor-egui`、`katana-chat-ui-egui` への依存
- 変更なし: preview / chrome は egui のまま（後続 Phase で対応）
- 各 Floem impl crate の実装は各リポジトリの openspec を参照
