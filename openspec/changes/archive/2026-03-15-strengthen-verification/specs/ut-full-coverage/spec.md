## ADDED Requirements

### Requirement: テストファイルの src 外分離

すべてのテストコードは `src/` 内の `#[cfg(test)] mod tests` ではなく、crate 直下の `tests/` ディレクトリに分離しなければならない（SHALL）。

#### Scenario: 既存テストの移行

- **WHEN** `katana-core/src/document.rs` 内の `#[cfg(test)] mod tests` を確認する
- **THEN** テストコードが存在せず、対応するテストが `katana-core/tests/document.rs` に存在する

#### Scenario: 新規テストの配置

- **WHEN** 新しいモジュール `foo.rs` にテストを追加する
- **THEN** テストは `tests/foo.rs`（または `tests/foo/` ディレクトリ）に配置され、`src/foo.rs` 内に `#[cfg(test)]` ブロックが存在しない

#### Scenario: src 内にテストヘルパーが残らない

- **WHEN** crate 内の全 `.rs` ファイルを検索する
- **THEN** `src/` 配下に `#[cfg(test)]` が存在しない（テスト専用の公開ヘルパーメソッドを除く）

### Requirement: テスト未実装モジュールへの UT 追加

テストが存在しないすべてのモジュールにユニットテストを追加しなければならない（SHALL）。

#### Scenario: settings.rs のテスト追加

- **WHEN** `katana-platform/tests/settings.rs` に対して `cargo test` を実行する
- **THEN** `SettingsService::new()`, `load_from()`, `settings()`, `settings_mut()`, `Default` のすべての公開メソッドをカバーするテストが存在し、パスする

#### Scenario: shell.rs ロジック部分のテスト追加

- **WHEN** `shell.rs` から抽出されたロジック関数（`hash_str`, `relative_full_path`, `process_action` 等）に対して `cargo test` を実行する
- **THEN** 各関数の正常系・異常系をカバーするテストが `tests/` ディレクトリに存在し、パスする

#### Scenario: preview_pane.rs のテスト追加

- **WHEN** `katana-ui/tests/preview_pane.rs` に対して `cargo test` を実行する
- **THEN** `update_markdown_sections`, `full_render`, `extract_svg`, `decode_png_rgba` の各関数をカバーするテストが存在し、パスする

### Requirement: カバレッジ計測の導入

`cargo-llvm-cov` を使用して行カバレッジを計測できなければならない（SHALL）。

#### Scenario: ローカルでのカバレッジ計測

- **WHEN** `cargo llvm-cov --workspace` を実行する
- **THEN** 各 crate のファイルごとの行カバレッジが表示される

#### Scenario: カバレッジレポートの生成

- **WHEN** `cargo llvm-cov --workspace --html` を実行する
- **THEN** HTML 形式のカバレッジレポートが `target/llvm-cov/html/` に生成される

### Requirement: CI カバレッジゲート

CI パイプラインにカバレッジ閾値チェックを含めなければならない（SHALL）。

#### Scenario: 閾値未達でCI失敗

- **WHEN** 行カバレッジが 100% 未満である
- **THEN** CI ジョブが失敗ステータスで終了する

#### Scenario: 閾値達成でCI成功

- **WHEN** 行カバレッジが 100% である
- **THEN** CI ジョブが成功ステータスで終了する
