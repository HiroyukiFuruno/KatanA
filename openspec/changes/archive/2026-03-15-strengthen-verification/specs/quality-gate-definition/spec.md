## ADDED Requirements

### Requirement: タスク完了の品質ゲート

AI エージェントがタスクを「完了」と報告するために、以下のすべてのゲートをパスしなければならない（SHALL）。

#### Scenario: Clippy ゲート

- **WHEN** タスクの完了を報告する前に `cargo clippy --workspace -- -D warnings` を実行する
- **THEN** warning / error がゼロで終了コード 0 を返す

#### Scenario: テストゲート

- **WHEN** タスクの完了を報告する前に `cargo test --workspace` を実行する
- **THEN** すべてのテストがパスする

#### Scenario: フォーマットゲート

- **WHEN** タスクの完了を報告する前に `cargo fmt --all -- --check` を実行する
- **THEN** フォーマット差分がゼロで終了コード 0 を返す

#### Scenario: カバレッジ確認

- **WHEN** 新しいロジックコードを追加した場合
- **THEN** 対応するユニットテストが追加され、そのロジックの行カバレッジが 100% である

### Requirement: CI パイプラインの品質ゲート統合

CI パイプラインに以下のすべてのステップを含めなければならない（SHALL）。

#### Scenario: CI の完全な品質ゲート

- **WHEN** PR が main ブランチに対して作成される
- **THEN** 以下のステップがすべて成功する:
  1. `cargo fmt --all -- --check`
  2. `cargo clippy --workspace -- -D warnings`
  3. `cargo test --workspace`
  4. カバレッジ 100%（`cargo llvm-cov --workspace --fail-under-lines 100`）

### Requirement: pre-push フックの品質ゲート

pre-push フックに完全な品質チェックを含めなければならない（SHALL）。

#### Scenario: pre-push 実行

- **WHEN** `git push` を実行する
- **THEN** fmt チェック、clippy チェック（-D warnings）、テスト実行がすべてパスした場合のみ push が成功する

### Requirement: coding_rules.md の完了定義更新

`coding_rules.md` の「完了の定義」セクションに、品質ゲートの各項目を明記しなければならない（SHALL）。

#### Scenario: 完了定義の内容

- **WHEN** `coding_rules.md` の「完了の定義 (Definition of Done)」を確認する
- **THEN** 以下の項目が含まれている:
  1. Clippy warning-free（`cargo clippy --workspace -- -D warnings` がパス）
  2. テスト全パス（`cargo test --workspace` がパス）
  3. 新規ロジックにはテストが付随している
  4. テストは `tests/` ディレクトリに配置されている（`src/` 内の `#[cfg(test)]` 禁止）
  5. カバレッジ 100%（`cargo llvm-cov --workspace --fail-under-lines 100`）
