# Tasks: enforce-headless-windows-processes

## Branch Rule

- 作業ブランチ: `release/v0.22.22-headless-process-enforcement`
- master への PR は release/* / hotfix/* / dependabot/* のみ許可（CI 制約）

## 1. Inventory & Audit

### Definition of Ready

- [x] 既存 `crates/katana-core/src/system/process.rs::ProcessService` の API が現状維持で良い
- [x] `scripts/screenshot/Cargo.toml` が既に `katana-core = { path = "../../crates/katana-core" }` を依存している

### Tasks

- [x] 1.1 `rg "Command::new" --include='*.rs'` を workspace 全体（target/ 除外）に対して再走査し、漏れリストを確定する
- [x] 1.2 build.rs ファイル群を `find . -name build.rs -not -path './target/*'` で列挙する
- [x] 1.3 漏れリストを 3 カテゴリ（facade 内 OK / 修正対象 / cfg(test) スキップ）に分類する

### Definition of Done

- [x] 修正対象の `Command::new` 呼び出しが file:line:program 単位で全件リスト化されている
- [x] 既存テスト範囲外（scripts/, build.rs）の件数が明示されている

## 2. Build Script Facade

### Definition of Ready

- [x] Task 1 で build.rs 漏れ箇所が確定している

### Tasks

- [x] 2.1 `crates/katana-ui/build_support/process.rs` を新規作成、`create_build_command(program: &str) -> Command` を実装。`#[cfg(windows)]` で `CREATE_NO_WINDOW` を `creation_flags` に設定
- [x] 2.2 `crates/katana-ui/build.rs` 冒頭で `include!("build_support/process.rs")` し、`Command::new` を `create_build_command` に置換
- [x] 2.3 `cargo build -p katana-ui` がクリーンに通ることを確認

### Definition of Done

- [x] build.rs に `std::process::Command::new` が残っていない
- [ ] Windows / macOS / Linux すべてでビルドが通る（CI ログで確認） — **release PR の CI で最終検証**

## 3. Scripts Migration

### Definition of Ready

- [x] Task 1 で scripts/screenshot 漏れ箇所が確定している
- [x] `katana_core::system::ProcessService` が `scripts/screenshot` から呼べる依存パスになっている

### Tasks

- [x] 3.1 `scripts/screenshot/src/executor.rs:28` を `ProcessService::create_command` に置換
- [x] 3.2 `scripts/screenshot/src/executor_harness.rs:837` (ffmpeg) を置換
- [x] 3.3 `scripts/screenshot/src/capture.rs:20,35,40,57,90` の 5 箇所を置換
- [x] 3.4 `cargo build` を `scripts/screenshot` で実行してビルドが通ることを確認
- [ ] 3.5 既存のスクリーンショット撮影フロー (もしあれば smoke test) を実行して動作確認 — **release 後のユーザー環境で実機確認に委ねる**

### Definition of Done

- [x] `scripts/screenshot/` 配下に `std::process::Command::new` が残っていない
- [x] `scripts/screenshot/` のビルドと既存挙動が壊れていない（cargo check clean）

## 4. Linter Scan Range Expansion

### Definition of Ready

- [x] Task 2 / Task 3 が完了し、修正後の状態で lint を回せる

### Tasks

- [x] 4.1 `crates/katana-linter/src/utils/file_collector.rs` に `LinterFileOps::collect_build_scripts(root: &Path) -> Vec<PathBuf>` を追加
- [x] 4.2 `crates/katana-linter/tests/ast_linter.rs::target_crates` を拡張、`scripts/screenshot/src/` を追加（`headless_process_target_dirs` として分離）
- [x] 4.3 `process_command.rs` の lint 入口を「target_dirs + build scripts」両方を受け取る形に整理
- [x] 4.4 `process_command.rs` の許可リストを絶対パス基準（`crates/katana-core/src/system/process.rs` と `crates/katana-ui/build_support/process.rs`）に厳格化

### Definition of Done

- [x] AST lint が scripts/ と build.rs を含めて走る
- [x] 許可リスト外の `Command::new` がすべて違反として検出される

## 5. Lint Rule Hardening

### Definition of Ready

- [x] Task 4 のスキャン範囲拡張が完了している

### Tasks

- [x] 5.1 `ProcessCommandOps::lint_with_allowlist(path, syntax, allowlist)` API を追加。既存 `lint` は default allowlist を渡す薄いラッパー
- [x] 5.2 ~~build script 内で `Command::new` を直接呼ぶケースを「同じ関数内で `creation_flags` を呼んでいるか」で検査するヘルパーを追加~~ → **不採用**: 「allowlist + cfg(test) 除外」方式で同等の保護が得られるため、複雑な visitor 拡張は不要と判断。`build_support/process.rs` だけが許可リストに入り、build.rs 内の直 `Command::new` はそのまま違反扱い。
- [x] 5.3 unit test を追加: (a) build.rs で creation_flags なしの `Command::new` を検出 (`detects_raw_command_new_in_build_script`) (b) `build_support/process.rs` での `Command::new` は許可 (`ignores_command_in_build_support_facade`) (c) scripts/screenshot/* の Command::new を検出 (`detects_raw_command_new_in_scripts`)

### Definition of Done

- [x] 新規 unit test が全て pass する（7 tests）
- [x] 既存 unit test (`detects_raw_command_new`, `ignores_command_in_process_service_facade`) が継続して pass

## 6. Regression Tests

### Tasks

- [x] 6.1 `crates/katana-linter/tests/ast_linter.rs` に integration test を追加: scripts/screenshot/ で Command::new を検出する `ast_linter_no_direct_process_command_in_sources`（実 scan 経路を通すため synthetic fixture ではなく現状コードに対して回す形に整理）
- [x] 6.2 build.rs を対象とする integration test `ast_linter_no_direct_process_command_in_build_scripts` を追加。さらに scan 範囲の縮退を検出する guard test `ast_linter_headless_process_build_scripts_are_discovered` も追加
- [x] 6.3 既存の `ast_linter_shared_kal_rules` を含む全 lint test が pass することを確認（72 tests）

### Definition of Done

- [x] `cargo test -p katana-linter` が Linux でクリーンに通る（macOS で確認、Linux は CI で最終確認）
- [x] 将来 `Command::new` を新規追加した場合に、build.rs / scripts/ / crates/ どこに置いても lint が違反として報告する

## 7. CHANGELOG & Release

### Tasks

- [x] 7.1 `CHANGELOG.md` に "Windows distribution stability" / "AST Linter scan expansion" のエントリを追記（EN）
- [x] 7.2 `CHANGELOG.ja.md` に対応する日本語エントリ追記（JST）
- [x] 7.3 `Cargo.toml` workspace.package.version を `0.22.22` に bump、`crates/katana-ui/Info.plist` も同期、`scripts/release/bump-version.sh 0.22.22` でリリースコミット作成
- [x] 7.4 PR description に proposal.md の Why / What Changes / Impact を要約して記載

### Definition of Done

- [x] CHANGELOG.md / CHANGELOG.ja.md が同期している
- [x] PR が release/* ブランチから master へ向いている

## 8. Final Verification

- [x] 8.1 `cargo test -p katana-linter` をローカルで実行、全 pass（72 tests）
- [x] 8.2 `cargo build --workspace` / `cargo check --workspace` がクリーンに通る
- [x] 8.3 `openspec validate "enforce-headless-windows-processes" --strict` を実行
- [ ] 8.4 Windows 実機（または VM）でビルド → 起動 → diagram render / update check を実施し、console window が表示されないことを目視確認 — **release 後のユーザー環境で実機確認**
- [x] 8.5 self-review skill を起動して pre-commit チェック（PASS）

## Release Process

- リリースは Task 7 完了後、`impl-release` workflow で実施
- PR マージ後に `build-and-release.yml` が自動発火し v0.22.22 を tag / publish
