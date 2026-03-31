## Why

Code / Preview の分割表示でスクロール同期が既定で有効な状態でも、文書末尾まで正しく追従できず、同期後に上下へガタつくことがあります。現状実装は単一の共有 `fraction` と前フレームの preview scroll extent に依存しており、末尾区間と往復同期の収束性を保証できていません。

## What Changes

- Code / Preview の split mode におけるスクロール同期を、文書全体の scrollable range を最後まで扱える契約として定義する
- 見出し anchor がある文書でも、最後の見出し以降の tail 区間を含めて editor / preview の末尾同期を成立させる
- 同期を受けた側が即座に逆方向の corrective scroll を再発火しない収束条件を定義する
- heading anchor が少ない文書や heading が存在しない文書でも破綻しない fallback を定義する
- vertical / horizontal split の両方で、末尾同期と no-jitter を固定化する回帰テストを追加する

## Capabilities

### New Capabilities
- `split-scroll-sync`: split mode における editor / preview 間の双方向スクロール同期、末尾到達、収束条件を扱う

### Modified Capabilities
<!-- なし -->

## Impact

- `crates/katana-ui/src/state/scroll.rs`
- `crates/katana-ui/src/views/panels/editor/ui.rs`
- `crates/katana-ui/src/views/panels/editor/logic.rs`
- `crates/katana-ui/src/views/panels/preview.rs`
- `crates/katana-ui/src/preview_pane/pane.rs`
- `crates/katana-ui/src/preview_pane/section.rs`
- `crates/katana-ui/src/views/layout/split.rs`
- `crates/katana-ui/src/shell_ui_tests.rs`
- 必要に応じて scroll sync 用の共有 utility module
