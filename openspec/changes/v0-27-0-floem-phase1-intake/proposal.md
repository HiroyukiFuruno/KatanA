## Why

egui の IME 破損・カラー絵文字欠如・immediate mode CPU コスト・vendor パッチ依存という構造的問題を解消するため、editor と preview の 2 コンポーネントを Floem impl crate（winit + wgpu + vello + cosmic-text + taffy）に差し替える。`katana-language-editor-floem` と `katana-document-preview-floem` の開発が完了した段階で KatanA に取り込む。

## What Changes

- `Cargo.toml` に `katana-language-editor-floem` を git dependency として追加し、`katana-language-editor-egui` への依存を除去する
- `Cargo.toml` に `katana-document-preview-floem` を git dependency として追加し、`katana-document-preview-egui` への依存を除去する
- `vendor/egui_commonmark/` パッチを除去する（不要になる）
- eframe との共存期間中は window・イベントループを egui 側に保持したまま、editor / preview 領域のみ Floem surface で描画する

## Capabilities

### Modified Capabilities

- `editor`: `katana-language-editor-floem` による IME 完全対応・カラー絵文字対応に切り替わる
- `document-preview`: `katana-document-preview-floem` による vello retained 描画に切り替わる

## Impact

- 追加: `Cargo.toml` に `katana-language-editor-floem`、`katana-document-preview-floem` の git dependency
- 削除: `katana-language-editor-egui`、`katana-document-preview-egui` への依存
- 削除: `vendor/egui_commonmark/` パッチ
- Phase 2（chrome 全体の Floem 化・eframe 完全除去）は v0.28.0 で扱う
- 各 Floem impl crate の実装は各リポジトリの openspec を参照
