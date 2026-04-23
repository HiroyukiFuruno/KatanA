## Why

katana プロジェクトの品質ゲートが不十分で、以下の問題が頻発している：

1. **Clippy warning が放置** — CI は `cargo clippy -- -D warnings` でエラー扱いにしているが、ローカル開発時に warning を無視して push できてしまうケースがあった（pre-push フックの実行漏れ）。現在も `render_edge` 関数の `too_many_lines` と `border_point` 関数の `too_many_arguments` が未修正（ビルドが error で止まる状態）。
2. **UT カバレッジが不十分** — `settings.rs`（テストゼロ）、`shell.rs`（テスト不可能な UI ロジックが密結合）、`main.rs`（テスト無し）、`preview_pane.rs`（`#[cfg(test)]` はあるが `#[test]` が実質ゼロ）など、テストが存在しないモジュールが多数ある。
3. **UI の検証がコード化されていない** — 手動確認に依存しており、仕様を満たしたかの客観的な証拠がない。
4. **「完了」の定義が曖昧** — 仕様が満たせていないのに完了と判断される事象が発生している。

## What Changes

- **Clippy 厳格化**: Clippy の既存 warning / error をすべて修正し、ワークスペース全体で warning-free 状態を維持するルールを確立する。
- **UT カバレッジ 100%**: テストが存在しない全モジュール（`settings.rs`, `shell.rs` のロジック部分, `preview_pane.rs`, `main.rs` のセットアップロジック）にユニットテストを追加し、ロジック部分の行カバレッジ 100% を目指す。
- **UI テスト自動化**: egui のスナップショットテスト or スクリーンショット比較テストを導入し、UI レンダリング結果をコードで検証可能にする。
- **カバレッジ計測の CI 統合**: `cargo-llvm-cov` 等を CI に導入し、カバレッジ 100% を必須ゲートとする。一切の例外なし。
- **完了定義の厳格化**: AI エージェントのタスク完了チェックリストに「clippy warning-free」「UT カバレッジ基準達成」「UI テスト pass」を明記する。

## Capabilities

### New Capabilities

- `clippy-strict-enforcement`: Clippy の warning を全プロジェクトでエラー扱いに統一し、既存違反をすべて修正する
- `ut-full-coverage`: テスト未実装モジュールに対するユニットテスト追加と、カバレッジ計測・ゲートの導入
- `ui-automated-testing`: egui アプリケーションの UI 検証を自動化するテストインフラの構築（`egui_kittest` によるコンポーネント統合テスト + スナップショットテスト）
- `quality-gate-definition`: pre-push / タスク完了定義における品質ゲートの明確化と強制

### Modified Capabilities

（既存 spec なし）

## Impact

- **対象 crate**: `katana-core`, `katana-platform`, `katana-ui`（全 crate）
- **pre-push hook**: カバレッジチェック追加の検討
- **dev-dependencies**: `cargo-llvm-cov`（カバレッジ）、`egui_kittest`（コンポーネント統合テスト）の追加
- **既存コード**: `drawio_renderer.rs` の `render_edge` / `border_point` のリファクタリング（Clippy 違反修正）
- **テスト構成**: 全 crate で `src/` 内テストを `tests/` ディレクトリに分離
- **ルールファイル**: `coding_rules.md` の完了定義を更新
