## Why

KatanA currently relies on egui `TextEdit` for the source-input strengthening work started before the preview-local editing roadmap. During v0.22.5 input stabilization, that path hit egui's platform-specific emoji and special-character rendering limitations; the team reached a likely architectural answer but intentionally deferred the value and scheduling decision.

v0.27.0 and v0.29.0 approach the same broad pain from a different, valid direction: make KatanA less code-editor-centric by strengthening preview-driven, local correction flows. This change preserves the separate native input-surface idea without weakening that preview-first direction.

## What Changes

- Introduce a version-undecided, explicitly deferred native input surface candidate that can eventually replace egui `TextEdit` for Markdown text entry if the work is later judged worth doing.
- Define an input model that owns buffer edits, cursor, selection, IME composition, clipboard, undo/redo, and scroll/caret visibility independently from egui widgets.
- Keep v0.27.0/v0.29.0 free to pursue the preview-first, local-correction approach; this change records the heavier TextEdit-replacement path as a separate lower-priority option.
- Preserve the product direction that KatanA defaults to viewing/preview, with rich input appearing only when editing is explicitly active.
- Avoid React, WebView, DOM runtime, or bundled web app approaches for this problem space.

## Capabilities

### New Capabilities

- `native-input-surface`: Defines a deferred native, egui-TextEdit-independent Markdown input surface candidate, including input model ownership, IME/clipboard behavior, emoji-safe rendering boundaries, and integration contracts with preview and editor components.

### Modified Capabilities

- None.

## Impact

- Affected future areas: `crates/katana-editor`, `crates/katana-ui/src/views/panels/editor`, Markdown authoring commands, preview-driven local edit surfaces, font/emoji rendering, shortcut dispatch, clipboard/image paste, and integration tests.
- This change is intentionally deferred and version-undecided because the team has not yet decided whether the implementation cost is worth paying. It must be scheduled only after an explicit future prioritization decision and coordinated with v0.27.0/v0.29.0 rather than implemented inside either change by default.
