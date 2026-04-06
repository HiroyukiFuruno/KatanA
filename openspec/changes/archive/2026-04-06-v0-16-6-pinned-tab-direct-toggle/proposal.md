## Why

Pinned tabs already show a visible pin affordance, but that affordance is not actionable. Users must still open the context menu to unpin, which makes the visible control misleading and adds unnecessary friction.

This behavior is self-contained and should be specified independently as `0.16.6`, so another agent can implement the direct-toggle interaction without carrying unrelated modal or preview work.

## What Changes

- Separate the pinned-tab body hit target from the pin icon hit target
- Make clicking the visible pin icon trigger `TogglePinDocument` directly
- Preserve existing context-menu pin/unpin behavior, pinned ordering, and group rules
- Add UI interaction coverage for direct unpin via the visible pin icon

## Capabilities

### New Capabilities

### Modified Capabilities
- `tab-context-menu`: The visible pin icon on a pinned tab becomes a direct unpin affordance

## Impact

- Affected code:
  - `crates/katana-ui/src/views/top_bar/ui.rs`
  - `crates/katana-ui/src/app/action.rs`
  - `crates/katana-ui/src/app_state.rs`
- Affected tests:
  - `crates/katana-ui/tests/integration/*.rs`
  - `crates/katana-ui/src/shell/shell_tests.rs`
- No external API change is expected, but pinned-tab interaction behavior changes at the UI contract level.
