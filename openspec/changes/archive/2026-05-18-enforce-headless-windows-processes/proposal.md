## Why

Windows 上で KatanA を実行するとき、外部プロセス起動の一部で console window が一瞬表示される問題が再発している。過去複数回のリリースで個別修正を入れているが（CHANGELOG: "Headless-by-Default enforcement", "Mermaid CLI on Windows", "Silent Background Updates", "Windows distribution stability"）、根本原因は **AST lint の検査範囲が狭く、`Command::new` の漏れを機械的に検出できていない** ことにある。

具体的には:

- `crates/katana-ui/build.rs` が `std::process::Command::new("rustc" | "git")` を直接呼び、`CREATE_NO_WINDOW` フラグなし。build script だが Windows でビルドする全開発者・CI で発火する。
- `scripts/screenshot/src/{executor,executor_harness,capture}.rs` で計 7 箇所が `Command::new` を直接使用。
- `crates/katana-linter` の `target_crates()` が `crates/katana-*/src` に限定されており、`build.rs` と `scripts/screenshot/` を一切検査していない。

中央ファサード `ProcessService::create_command` と AST lint rule `no-direct-process-command` 自体は実装済みなので、本 change では **lint の検査範囲を漏れなく広げ、漏れている呼び出しを ProcessService 経由へ集約する**。

## What Changes

- `crates/katana-ui/build_support/process.rs` に build script 用の `create_command_no_window` 共有ヘルパーを置き、`build.rs` から `include!()` で取り込む（`katana-core` への循環依存を避けつつ単一ソース化）。
- `scripts/screenshot/` の 7 箇所を `katana_core::system::ProcessService::create_command` 経由に置換。
- `crates/katana-linter`:
  - `target_dirs` に `scripts/screenshot/src/` を追加。
  - `build.rs` ファイル群をスキャン対象に加える `LinterFileOps::collect_build_scripts` を導入。
  - `process_command.rs` rule の許可リストを「facade ファイルのみ」に厳格化（`system/process.rs` と `build_support/process.rs`）。
  - build script 内の `Command::new` 直呼び出しは、その関数で `creation_flags` を併用しているかを検査するパターンも追加。
- `tests/ast_linter.rs` に scripts/build.rs を含めた回帰テストを追加。

## Capabilities

### New Capabilities

- `headless-process-enforcement`: Windows での console window 表示を防ぐため、外部プロセス起動の集約と AST lint による違反検知を統制する。

## Impact

- `crates/katana-ui`: `build.rs` の構造変更、`build_support/process.rs` 新設。
- `crates/katana-core`: 既存 `ProcessService::create_command` の許可リスト更新のみ（API 変更なし）。
- `crates/katana-linter`: 検査範囲拡張、rule 強化、テスト追加。
- `scripts/screenshot`: `ProcessService` 経由への置換（既に katana-core を path 依存）。
- 既存 OpenSpec change `extract-katana-ast-lint` と整合: 本 change で強化した rule は同 change で外部 crate `katana-ast-lint` へ移管されることを前提に、`katana-linter` 内の許可リスト形式を adapter で渡せる構造にする（path 直書きを避ける）。
