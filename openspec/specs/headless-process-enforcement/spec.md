## Purpose

KatanA SHALL guarantee that no external operating-system process spawned by the application — production code, supplementary script crates, or Cargo build scripts — can flash a console window on Windows. This is enforced by routing every `Command::new` call through approved facades that apply the Windows `CREATE_NO_WINDOW` flag, and by an AST lint rule whose scan range covers every Rust source path that can spawn a process at build time or at runtime.

This capability exists because the same regression (a stray `std::process::Command::new` causing a momentary console window) has reappeared in multiple past releases. Centralising both the spawning facade and the lint scope is the structural fix that prevents future drift.

## Requirements

### Requirement: All external process invocations route through the headless facade

The system SHALL invoke external operating-system processes only through approved facades that apply Windows `CREATE_NO_WINDOW` semantics, so that background processes never flash a console window on Windows.

#### Scenario: Production code spawns an external process

- **WHEN** Rust source code under `crates/katana-*` or `scripts/screenshot/` needs to launch an external program
- **THEN** it calls `katana_core::system::ProcessService::create_command` (or `create_command_visible` for the documented Java exception)
- **THEN** it does not call `std::process::Command::new` directly

#### Scenario: A Cargo build script spawns an external process

- **WHEN** a `build.rs` file in any workspace crate needs to launch an external program (e.g. `rustc --version`, `git rev-parse`)
- **THEN** it calls `create_build_command` from `crates/katana-ui/build_support/process.rs` (included via `include!`)
- **THEN** the helper applies `creation_flags(CREATE_NO_WINDOW)` under `#[cfg(windows)]`

### Requirement: AST lint scans build scripts and supplementary script crates

The AST linter SHALL inspect every Rust source file that can spawn processes at build time or at runtime, including files outside the primary `crates/katana-*/src` trees.

#### Scenario: Lint discovers a new direct Command::new in a build script

- **WHEN** a developer adds `std::process::Command::new(...)` to any `crates/<name>/build.rs`
- **WHEN** `cargo test -p katana-linter` runs
- **THEN** the `no-direct-process-command` rule reports the violation with file path and line number
- **THEN** the rule directs the developer to `create_build_command` in `build_support/process.rs`

#### Scenario: Lint discovers a new direct Command::new in scripts/

- **WHEN** a developer adds `std::process::Command::new(...)` anywhere under `scripts/screenshot/src/`
- **WHEN** `cargo test -p katana-linter` runs
- **THEN** the rule reports the violation
- **THEN** the rule directs the developer to `ProcessService::create_command`

### Requirement: Allowlist is path-anchored and minimal

The `no-direct-process-command` AST lint rule SHALL allow direct `Command::new` only in named facade files, with the allowlist expressed as absolute repo-relative paths.

#### Scenario: Facade file uses Command::new internally

- **WHEN** `crates/katana-core/src/system/process.rs` calls `std::process::Command::new`
- **THEN** the lint rule does not report a violation
- **WHEN** `crates/katana-ui/build_support/process.rs` calls `std::process::Command::new`
- **THEN** the lint rule does not report a violation

#### Scenario: A file that merely contains "process.rs" in its name uses Command::new

- **WHEN** a file path such as `crates/some-feature/src/process.rs` (not in the allowlist) calls `Command::new`
- **THEN** the lint rule reports the violation
- **THEN** the developer must either route through the facade or extend the allowlist through this OpenSpec capability

### Requirement: Allowlist is consumable by external rule packages

The lint rule API SHALL accept the allowlist as a parameter so that the upcoming external `katana-ast-lint` crate can use the same rule logic with repo-local allowlists provided by adapters.

#### Scenario: External rule adapter passes its own allowlist

- **WHEN** `katana-ast-lint` (or any future consumer) invokes the process-command rule
- **THEN** it calls `ProcessCommandOps::lint_with_allowlist(path, syntax, allowlist)`
- **THEN** the rule itself does not hard-code KatanA-specific paths

### Requirement: Regression tests cover scripts and build scripts

The AST lint test suite SHALL include integration tests that exercise both build-script paths and `scripts/screenshot/` paths, so the scan range cannot silently regress.

#### Scenario: Scan range regression is detected

- **WHEN** `target_dirs` or `collect_build_scripts` accidentally drops `scripts/screenshot/src/` or any `crates/<name>/build.rs`
- **WHEN** `cargo test -p katana-linter` runs
- **THEN** an integration test fails because the synthetic violation it injects is no longer detected
