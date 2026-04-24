## Why

v0.22.5 is a redo of the code input improvements work for the `release/v0.22.5` integration branch. The previous archived change still contains stale version labels and a dummy task, while the release workflow needs an active OpenSpec change that captures the latest user review feedback.

The release priority is not to expand authoring scope. It is to fix input regressions first: Markdown must be typeable, input support UI must be visible, scrolling must remain recoverable upward, and image attach/paste paths must be discoverable and usable without fighting editor input behavior.

## What Changes

- Treat editor input mode as a protected text-entry context: application shortcuts that conflict with native editor/input behavior MUST be disabled while the text editor is focused.
- Restore and verify the editor authoring support toolbar for editable Markdown documents.
- Keep ordinary Markdown typing, IME composition, selection, undo/redo, and normal paste behavior intact.
- Make clipboard images work through the normal paste flow when the clipboard contains image data.
- Add a command palette route for attaching an image file to the active Markdown document.
- Preserve the existing image ingest contract: save images under the configured document-relative asset directory and insert a relative Markdown image reference.
- Fix scroll regressions so the editor can be scrolled back to the top after typing, using the toolbar, or synchronizing with preview.

## Capabilities

### New Capabilities

- `markdown-asset-ingest`: Image file attach, clipboard image paste, document-relative asset saving, and Markdown image reference insertion.

### Modified Capabilities

- `markdown-authoring`: Editor input safety, source-first authoring controls, toolbar visibility, and editor scroll recovery.

## Impact

- Primary UI/action areas: `crates/katana-ui/src/views/panels/editor/ui.rs`, `crates/katana-ui/src/views/panels/editor/toolbar.rs`, `crates/katana-ui/src/views/panels/editor/logic.rs`, `crates/katana-ui/src/app/action/process_authoring.rs`, and `crates/katana-ui/src/app/action/image_ingest.rs`.
- Shortcut and command routing areas: `crates/katana-ui/src/state/shortcut_context.rs`, `crates/katana-ui/src/shell_ui/shell_ui_shortcuts.rs`, `crates/katana-ui/src/state/command_inventory/edit_commands.rs`, and command palette providers/results.
- Verification areas: focused unit tests for shortcut context and image ingest, editor logic tests for scroll recovery, and UI/integration coverage proving Markdown input, toolbar visibility, command palette image attach, and normal clipboard image paste.
