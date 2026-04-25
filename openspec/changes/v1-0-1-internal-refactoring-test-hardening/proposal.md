## Why

策定日: 2026-04-25

v1.0.0 正式リリース後に安定して開発を続けるには、機能追加より先に内部構造と回帰検知の土台を整える必要がある。2026-04-25 時点の解析では、`katana-ui` に UI、状態、操作、領域ロジック、巨大テストが集まりやすく、正式リリース後のデグレード検知として十分とは言いにくい。

## What Changes

- v1.0.1 の最初の規格として、内部リファクタリングとテスト強化を扱う。
- ディレクトリ再設計で済む箇所と、内部実装の責務分離まで必要な箇所を分けて進める。
- 着手時に 2026-04-25 からの差分を確認し、計画を最新のコード状態へ合わせて更新する。
- `katana-ui` の application action、state、shell、view、領域 service の境界を再定義する。
- 巨大な単体テスト / 統合テストを責務単位に分割し、release 後の回帰検知 gate を明確にする。
- 既存 behavior を変えず、feature work ではなく保守性と検知力の改善に絞る。

## Capabilities

### New Capabilities

- `internal-architecture-refactoring`: v1.0.1 以降の保守性を支える module 境界、service 境界、移行計画を定義する。
- `release-regression-safety`: 正式リリース後のデグレードを検知する単体テスト / 統合テスト / fixture / gate を定義する。

### Modified Capabilities

- なし。ユーザー向け挙動は変えず、内部構造と検証体制を改善する。

## Impact

- `crates/katana-ui/src/app_action.rs`
- `crates/katana-ui/src/app_state.rs`
- `crates/katana-ui/src/shell/*`
- `crates/katana-ui/src/shell_ui/*`
- `crates/katana-ui/src/views/*`
- `crates/katana-ui/src/preview_pane/*`
- `crates/katana-ui/tests/integration/*`
- `Makefile` / `scripts/runner/*` の test gate
