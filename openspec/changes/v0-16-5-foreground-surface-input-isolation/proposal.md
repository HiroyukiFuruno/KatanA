## Why

Foreground surfaces in Katana can still leak pointer interaction into background panes. When settings windows, command/search surfaces, context menus, popups, or fullscreen-style overlays are open, hover and click reactions can still update the workspace tree, editor, preview, or tab strip behind them.

This is a shell-level interaction contract problem, not an isolated widget bug. It should be specified independently as `0.16.5` so another agent can implement and verify the blocker behavior without mixing it with unrelated tab, markdown, or preview-sync work.

## What Changes

- Introduce a shell-owned foreground-surface blocker contract that centralizes whether background interaction must be suppressed for the current frame
- Explicitly include the settings window, command palette, file search modal, file-operation/about/meta/update/terms windows, tab/workspace context menus, history/breadcrumb/group popups, settings-local popups, splash overlays, and fullscreen/slideshow/detached surfaces in that contract
- Prevent background workspace tree, editor, preview, and tab-strip hover/click/context-menu reactions while the blocker is active
- Add UI integration coverage for representative windows, menus, popups, and overlay surfaces using response-based assertions

## Capabilities

### New Capabilities

### Modified Capabilities
- `workspace-shell`: Foreground windows, popups, menus, and overlays block pointer interaction from reaching background panes

## Impact

- Affected code:
  - `crates/katana-ui/src/shell_ui/mod.rs`
  - `crates/katana-ui/src/views/modals/command_palette.rs`
  - `crates/katana-ui/src/views/modals/search.rs`
  - `crates/katana-ui/src/views/modals/about.rs`
  - `crates/katana-ui/src/views/modals/meta_info.rs`
  - `crates/katana-ui/src/views/modals/file_ops.rs`
  - `crates/katana-ui/src/views/modals/update.rs`
  - `crates/katana-ui/src/views/modals/terms.rs`
  - `crates/katana-ui/src/views/app_frame/ui.rs`
  - `crates/katana-ui/src/views/panels/workspace/ui.rs`
  - `crates/katana-ui/src/views/top_bar/ui.rs`
  - `crates/katana-ui/src/settings/ui.rs`
  - `crates/katana-ui/src/settings/tabs/font.rs`
  - `crates/katana-ui/src/settings/tabs/theme.rs`
  - `crates/katana-ui/src/settings/tabs/workspace.rs`
  - `crates/katana-ui/src/views/splash.rs`
  - `crates/katana-ui/src/preview_pane/fullscreen.rs`
- Affected tests:
  - `crates/katana-ui/tests/integration/*.rs`
  - `crates/katana-ui/src/*_tests.rs`
 - No external API change is expected, but shell interaction behavior changes at the UI contract level.
