## Why

v0.28.0（Floem Phase 2・egui 完全除去）完了後、v1.0.0 として公式リリースするための品質確認・polish を行う。egui 時代の制約（IME 破損・カラー絵文字欠如・vendor パッチ）がなくなった状態で、ユーザー体験の最終仕上げを行う。

## What Changes

- 全機能の結合テスト実行・品質ゲート通過確認
- Floem 移行後の UI polish（レイアウト調整・アニメーション・accessibility）
- `cargo clippy --workspace -- -D warnings` と `cargo fmt --all --check` のパス確認
- macOS / Linux / Windows 各プラットフォームでのビルド・動作確認
- リリースビルドの作成・公証・配布準備

## Capabilities

### Modified Capabilities

- 全 capability の統合検証・polish

## Impact

- DoR: v0.28.0（Floem Phase 2・egui 完全除去）完了後
- 全クレートの結合テスト
- macOS / Linux / Windows リリースビルド
