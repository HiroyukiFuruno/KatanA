## ADDED Requirements

### Requirement: Clippy warning-free ワークスペース
ワークスペース内のすべての crate は `cargo clippy --workspace -- -D warnings` をエラーなしで通過しなければならない（SHALL）。

#### Scenario: 既存 Clippy 違反の修正
- **WHEN** `cargo clippy --workspace -- -D warnings` を実行する
- **THEN** warning / error がゼロで終了コード 0 を返す

#### Scenario: drawio_renderer の render_edge 関数
- **WHEN** `render_edge` 関数を Clippy でチェックする
- **THEN** `too_many_lines` 違反が発生しない（関数を 30 行以下にリファクタリング済み）

#### Scenario: drawio_renderer の border_point 関数
- **WHEN** `border_point` 関数を Clippy でチェックする
- **THEN** `too_many_arguments` 違反が発生しない（引数を構造体にまとめてリファクタリング済み）

### Requirement: deny(warnings) の統一適用
すべての crate ルート（`lib.rs` / `main.rs`）に `#![deny(warnings)]` を配置しなければならない（SHALL）。

#### Scenario: lib.rs への deny 適用
- **WHEN** `katana-core/src/lib.rs` のクレートアトリビュートを確認する
- **THEN** `#![deny(warnings)]` が存在する

#### Scenario: main.rs への deny 適用
- **WHEN** `katana-ui/src/main.rs` のクレートアトリビュートを確認する
- **THEN** `#![deny(warnings)]` が存在する

### Requirement: ファイル単位の deny 重複排除
crate ルートで `#![deny(warnings)]` を宣言した場合、個別ファイルの `#![deny(clippy::too_many_lines, clippy::cognitive_complexity)]` は削除しなければならない（SHALL）。

#### Scenario: 全ファイルからの個別 deny 削除
- **WHEN** ワークスペース内の全 `.rs` ファイルを検索する
- **THEN** `#![deny(clippy::too_many_lines)]` や `#![deny(clippy::cognitive_complexity)]` 等の個別 deny アトリビュートが存在しない
