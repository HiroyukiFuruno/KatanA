## Why

Switching tabs currently breaks the continuity of long-running preview work. In the current implementation, inactive tab renders are explicitly aborted, so Mermaid, PlantUML, Draw.io, and image loading can remain stuck in a loading-like state when the user returns to a tab that had not finished rendering yet.

The preview pipeline already keeps per-tab panes, but it does not preserve an explicit per-tab render lifecycle that distinguishes "the source or asset has finished loading in the background" from "the active preview has drawn that result." This gap now blocks predictable tab switching and makes heavy preview content feel unreliable.

## What Changes

- Introduce a per-tab preview lifecycle contract so background preview work can continue across tab switches and hand its completed results back to the next activation of that tab
- Stop canceling inactive tab diagram/image work solely because another tab became active; only cancel work when the source changes, the tab closes, or the job is explicitly invalidated
- Add explicit per-tab preview session state, stored per section inside each tab session, that separates a loaded/completed background result from a drawn/attached visible preview state
- Ensure Mermaid, PlantUML, Draw.io, and tab-owned image-backed preview sections resume from the in-flight or completed background work instead of restarting from a stale loading state on tab revisit
- Require implementation-time verification of CommonMark/HTTP image paths under the same reproduction and fold them into the same lifecycle within this change if they exhibit the same tab-switch regression
- Add regression coverage for tab switching during unfinished preview work and for hydration when the tab becomes active again after background completion

## Capabilities

### New Capabilities
- `tab-preview-lifecycle`: Per-tab preview session state, background render continuity, and reattachment/hydration behavior when the user returns to a tab

### Modified Capabilities
- `diagram-block-preview`: Diagram blocks continue background rendering across tab switches and reattach finished results without re-entering a stuck loading state
- `local-asset-preview`: Local image preview loading continues across tab switches and reuses completed results when the user revisits the tab

## Impact

- Affected code:
  - `crates/katana-ui/src/app/action.rs`
  - `crates/katana-ui/src/app/document.rs`
  - `crates/katana-ui/src/app/preview.rs`
  - `crates/katana-ui/src/preview_pane/types.rs`
  - `crates/katana-ui/src/preview_pane/core_render.rs`
  - `crates/katana-ui/src/preview_pane/background.rs`
  - `crates/katana-ui/src/preview_pane/images.rs`
  - `crates/katana-ui/src/shell/types.rs`
- Affected tests:
  - `crates/katana-ui/src/shell/shell_tests.rs`
  - `crates/katana-ui/src/preview_pane/tests.rs`
  - `crates/katana-ui/tests/integration/*.rs`
- No external API change is expected, but preview lifecycle behavior and tab-switch responsiveness will change at the UI contract level.
