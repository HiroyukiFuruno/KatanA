## Why

Preview hover highlighting and split-mode synchronization still drift around rich preview blocks. Mermaid, PlantUML, and Draw.io do not consistently highlight the matching source range, and rendered diagrams or alert/admonition blocks can shift source mapping before or after their boundaries.

This should be specified independently as `0.16.8`, because it is a preview/source-mapping problem rather than shell interaction, tab affordance, or alert-spacing work.

## What Changes

- Give Mermaid, PlantUML, Draw.io, and GitHub Flavored Markdown alert/admonition preview blocks stable block-level source mapping for hover highlight
- Keep split-sync/source anchors aligned before and after rich block boundaries
- Preserve stable source anchoring for rendered diagram blocks across pending-to-rendered replacement
- Add response-based integration coverage for hover highlight and split sync around diagrams and alert/admonition blocks

## Capabilities

### New Capabilities
- `block-highlight-improvements`: Stabilize preview-hover to source-highlight mapping for rendered diagrams and alert/admonition blocks
- `split-scroll-sync`: Keep split-mode preview/editor synchronization stable when rendered diagrams and alert/admonition blocks affect preview geometry

### Modified Capabilities
- `diagram-block-preview`: Preserve stable source anchoring for rendered Mermaid, PlantUML, and Draw.io blocks

## Impact

- Affected code:
  - `crates/katana-ui/src/views/panels/preview/ui.rs`
  - `crates/katana-ui/src/preview_pane/section.rs`
  - `crates/katana-ui/src/state/scroll_sync.rs`
  - `crates/katana-ui/src/preview_pane/types.rs`
  - `crates/katana-ui/src/preview_pane/core_render.rs`
- Affected tests:
  - `crates/katana-ui/tests/integration/*.rs`
  - `crates/katana-ui/src/*_tests.rs`
- No external API change is expected, but preview hover-highlight and split-sync behavior changes at the UI contract level.
