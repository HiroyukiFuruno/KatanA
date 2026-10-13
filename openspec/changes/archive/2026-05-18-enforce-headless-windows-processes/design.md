## Context

KatanA の Windows 配布では、background process（diagram rendering, in-app update, file download, theme detection など）から `std::process::Command::new` を直接呼ぶと一瞬コンソールウィンドウがフラッシュする問題が長年再発してきた。

過去のリリースで以下の対策が入っている:

- `crates/katana-core/src/system/process.rs::ProcessService::create_command` が中央ファサードとして `#[cfg(windows)]` で `CREATE_NO_WINDOW (0x08000000)` を `creation_flags` に設定。
- `crates/katana-linter` の AST lint rule `no-direct-process-command` が `Command::new` の直呼び出しを禁止し、ファサード経由を強制。

しかし以下の漏れが残っており、繰り返し再発の原因となっている:

1. **build.rs**: `crates/katana-ui/build.rs` が `Command::new("rustc" | "git")` を直接呼ぶ。build script は `katana-core` を依存に持てない（循環）ため、これまで例外扱いになっていた。
2. **scripts/screenshot**: workspace 外の独立 crate のため `katana-linter` の `target_dirs` から外れ、計 7 箇所の `Command::new` が無検査だった。
3. **lint 範囲の固定化**: `crates/katana-linter/tests/ast_linter.rs::target_crates` が `crates/katana-*/src` に hard-coded されており、新規 crate / build script を追加すると自動で検査対象に含まれない。

## Goals

- Windows での console window 表示を完全に防止する。
- `Command::new` の新規追加・移動が起きても、AST lint で漏れを機械的に検出する。
- 既存 OpenSpec change `extract-katana-ast-lint` の方向性（共通 rule、repo-local adapter）と整合させる。

## Non-Goals

- `ProcessService` API そのものの再設計（既存 API を維持）。
- macOS/Linux 側のプロセス起動挙動の変更。
- `scripts/screenshot` を workspace member に取り込むこと（独立 workspace のまま）。

## Decisions

### Build Script Facade

build.rs から `katana-core` を依存できないという制約を回避するため、`crates/katana-ui/build_support/process.rs` に **build script 専用の最小ヘルパー** を置き、`build.rs` から `include!("build_support/process.rs")` で取り込む。

```rust
/* build_support/process.rs (include! 経由で build.rs に展開される) */
fn create_build_command(program: &str) -> std::process::Command {
    #[allow(unused_mut)]
    let mut command = std::process::Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}
```

代替案として「build.rs に直接 `#[cfg(windows)]` ブロックを書く」も検討したが、build.rs が複数 crate に増えると同じ block を散在させることになり、本 change の動機（漏れの根絶）と矛盾する。include! 方式なら lint 側で「`include!` で取り込まれた facade ファイル」を許可リストに加えるだけで済む。

### Allowlist Scope

`process_command.rs` rule の許可ファイルパスを以下に限定する:

- `crates/katana-core/src/system/process.rs`
- `crates/katana-ui/build_support/process.rs`

それ以外の場所で `Command::new` を呼ぶ場合は違反として報告。

### Linter Scan Range Expansion

`target_dirs` を **明示的列挙** から **workspace 設定駆動** へ寄せる。当面は `target_crates()` に以下を追加:

- `scripts/screenshot/src/`
- workspace root 直下の `crates/*/build.rs` をスキャンする専用ロジック

build.rs は通常 `crates/<name>/build.rs` という位置で、`src/` 配下ではないため既存 `collect_rs_files` の `tests/` 除外と同じ仕組みに乗らない。新たに `LinterFileOps::collect_build_scripts(workspace_root)` を追加して明示的に集める。

### AST-Lint vs. KAL Boundary

`extract-katana-ast-lint` change により共通 rule が外部 crate `katana-ast-lint` へ移管される計画がある。本 change で強化する `process_command.rs` は KatanA 固有の **許可リストパス** を持つため、共通 rule に直書きせず、許可リストを呼び出し側 adapter から渡せる API 形に整える:

```rust
ProcessCommandOps::lint_with_allowlist(path, syntax, &allowlist)
```

これにより `katana-ast-lint` 移管時にも repo-local concerns を adapter 側に閉じ込められる。

### Verification Strategy

AST lint は静的検査なので、Linux/macOS のホスト上でも全 target を検査できる。`cargo test -p katana-linter` が CI で発火するため、追加の Windows ホスト実機テストは不要（ただしリリース直前の手動確認は引き続き推奨）。

## Risks

- **build_support/process.rs の include! が悪用される**: include! で取り込まれた facade ファイル名で `Command::new` を許可するため、将来同名のファイルを別目的で作ると検査が緩むリスク。→ 許可リストは **絶対パス基準**（`crates/katana-ui/build_support/process.rs`）で厳格化することで回避。
- **scripts/screenshot が katana-core 依存を持つことで build chain が長くなる**: 既に依存済みのため追加コストはゼロ。
- **lint 範囲拡張により既存違反が表面化**: 本 change で発見した 7 箇所以外にも残る可能性。→ lint を有効化する前に grep で全走査し、tasks に明記。

## Open Questions

- `scripts/screenshot` 以外に lint 対象に加えるべき workspace 外 crate はあるか？（`scripts/ci/`, `scripts/release/` など。本 change のスコープでは screenshot のみとし、他は別 change で追加）
- `katana-ast-lint` 外部 crate への移管タイミング: 本 change は `crates/katana-linter` 内に閉じて完結させる。`extract-katana-ast-lint` の進行に応じて、後続 change で migration を扱う。
