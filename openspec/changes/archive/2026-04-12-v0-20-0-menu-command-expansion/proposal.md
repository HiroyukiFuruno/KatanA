## Why

現在の menu surface は macOS native menu と一部 command palette に分散しており、File / View / Help の到達性と command coverage が十分に整理されていない。`v0.21.0` の shortcut customization を成立させるには、その前に user-facing command inventory と menu grouping を正規化する必要がある。

## What Changes

- File / View / Help を中心に app command surface を拡張する
- `AppAction` と user-facing label / availability / grouping を結び付ける command inventory を導入する
- macOS native menu、non-macOS in-app command surface、command palette で同じ command metadata を再利用する
- disabled state、document/workspace 未選択時の挙動、future shortcut editor との接続点を定義する
- export、settings、refresh、diagnostics、release notes など既存 command を menu group に整理する

## Capabilities

### New Capabilities

- `desktop-command-surface`: menu、palette、future shortcut editor が共有する command inventory と grouping を提供する

### Modified Capabilities

- `menu-enhancement`: File / View / Help を中心に menu coverage と availability contract を拡張する

## Impact

- 主な影響範囲は `crates/katana-ui/src/macos_menu.m`、`crates/katana-ui/src/native_menu.rs`、`crates/katana-ui/src/state/command_palette_providers.rs`、`crates/katana-ui/src/views/modals/command_palette.rs`、`crates/katana-ui/src/app_state.rs`
- `v0.21.0` の shortcut customization の prerequisite になる
