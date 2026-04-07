## Why

GitHub Flavored Markdown alert/admonition blocks currently use spacing that feels uneven. The title row and whole-block margins need a tighter and more intentional reading rhythm.

This is isolated markdown rendering polish and should be specified independently as `0.16.7`, so another agent can patch the vendored alert renderer without carrying unrelated shell or preview-sync work.

## What Changes

- Adjust alert title-row padding to use asymmetric top/bottom spacing
- Reduce whole-block vertical margin so alert/admonition blocks sit more naturally between surrounding paragraphs and lists
- Keep the fix localized to the vendored alert renderer rather than layering preview-only spacing hacks above it
- Add fixture-based layout assertions for alert spacing and regression checks against nearby block types

## Capabilities

### New Capabilities

### Modified Capabilities
- `markdown-authoring`: GitHub Flavored Markdown alert/admonition blocks use compact vertical rhythm

## Impact

- Affected code:
  - `vendor/egui_commonmark_upstream/egui_commonmark_backend/src/alerts.rs`
  - `crates/katana-ui/src/preview_pane/section.rs`
- Affected tests:
  - `crates/katana-ui/tests/integration/*.rs`
  - markdown fixture tests under the preview/rendering path
- No external API change is expected, but alert/admonition preview layout changes at the UI contract level.
