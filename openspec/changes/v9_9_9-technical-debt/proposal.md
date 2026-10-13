# Proposal: Refactor Large UI Rendering Functions

## Note

v0.28.0（Floem Phase 2・egui 完全除去）完了後、`views/` の egui `Ui` ブロックは Floem の vello/taffy ベースのコードに置き換わる。本 change の対象ファイル・関数名はその時点で再評価する。egui 移行前に着手する場合は、Floem 移行で書き直す箇所を重複作業しないよう範囲を絞ること。

## Goal

Enforce the "30 lines per function" coding standard across the `katana-ui` crate.

## Discovery Context

During the extraction of components from `shell_ui.rs` into the `views/` modules (e.g., `views/panels/workspace.rs`, `views/top_bar.rs`), we observed that several rendering functions inherited directly from `shell_ui.rs` far exceed the 30-line limit defined in `docs/coding-rules.md`. To maintain safety and prevent regressions during the massive 5,000-line modularization, we preserved the internal structure of these functions.

## Technical Debt Item

- File: `crates/katana-ui/src/views/panels/workspace.rs`
  - Violation: `render_workspace_panel()`, `render_tree_node()`, etc. are extremely long.
- File: `crates/katana-ui/src/views/top_bar.rs`
  - Violation: `render_top_bar()` and deeply nested UI elements exceed both size limits and nesting depth limits.
- Other `views/**/*.rs` files contain similar large `egui::Ui` inline rendering blocks.

## Proposed Fix

1. Iteratively target individual `views/*` modules.
2. Decompose large UI blocks into smaller helper functions / internal sub-components.
3. Extract inline closures into private `impl` methods where applicable to reduce nesting.
4. Ensure no new bugs are introduced by confirming layout logic matches visual parity.

---

## Additional Discovery: Clippy Violations Outside the Diff

While running `cargo clippy --workspace --tests -- -D warnings` during the self-review of the `enforce-headless-windows-processes` change (2026-05-18), the following pre-existing clippy violations were observed. They are unrelated to the headless-process work and are recorded here per the self-review skill's "Fix it now, or record it now. Never leave it untracked." rule.

### katana-core

- `crates/katana-core/src/editor/mod.rs:104` — `default_constructed_unit_structs`: `NoopSyntaxHighlighter::default()` should be replaced with the literal unit construction `NoopSyntaxHighlighter`. Inherited from commit `6d00d31a` ("refactor: align export and editor interfaces", 2026-05-04).

### katana-ui

- `crates/katana-ui/src/linter_bridge.rs:299` — `needless_borrow`: reference immediately dereferenced.
- `crates/katana-ui/src/views/panels/explorer/drag.rs:239` — `needless_borrow`.
- `crates/katana-ui/src/views/panels/toc/tests/anchor_state_tests/shared_current_and_hover_tests.rs:64` — `vec_init_then_push`: `vec![0..1]` can be a constant-len Vec.
- `crates/katana-ui/src/shell/shell_tests.rs:1155` — `needless_update`: struct update has no effect.
- `crates/katana-ui/tests/ui_integration_parallel.rs:65` — `clashing_extern_declarations` / module loaded twice: `crates/katana-ui/tests/integration/preview_pane/styling.rs`.
- `crates/katana-ui/src/views/panels/problems/bulk_fixes.rs:149` — `unnecessary_clone`: replace with `std::slice::from_ref`.

### scripts/screenshot

- `scripts/screenshot/src/executor_harness.rs` (lines 118, 209, 413, 429, 932, 1019, etc.) — `needless_borrow` / `redundant_deref`. Pre-existing in the screenshot tooling.

### Recommendation

These violations were not introduced by the headless-process enforcement work and were not fixed in that PR to keep the diff focused. They should be addressed either as part of `v9_9_9-technical-debt` (after the v0.28.0 Floem migration) or as a dedicated cleanup change targeted at `cargo clippy -- -D warnings` health.
