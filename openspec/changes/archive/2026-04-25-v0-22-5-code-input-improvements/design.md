## Context

The current release branch already contains pieces of v0.22 authoring work: `AppAction::AuthorMarkdown`, `AppAction::IngestImageFile`, `AppAction::IngestClipboardImage`, editor toolbar rendering, command inventory entries, and an image ingest path that writes assets and inserts Markdown image syntax. The redo must make these pieces safe for the actual editor input workflow.

Two implementation details are especially important:

- `ShortcutContextResolver::context_allows` currently allows global shortcuts while the editor context is active. This is useful for some app commands, but unsafe for text-entry combinations that the editor or OS input method owns.
- Clipboard image ingest currently exists as an explicit command. v0.22.5 requires the normal paste gesture to work for image data without breaking normal text paste.

## Goals / Non-Goals

**Goals:**

- Protect editor input mode from shortcut collisions with typing, native text editing, IME composition, selection, and normal paste.
- Preserve KatanA's preview-first product contract for newly opened documents.
- Keep Markdown source as the only editable document representation.
- Show authoring support controls only when editable Markdown input is active, anchored near the input cursor.
- Route image file attach through the command palette.
- Route clipboard image ingest through normal paste when image data is present, while leaving text paste untouched.
- Ensure manual upward scrolling to the document top is not overridden by scroll sync, cursor-line handling, or toolbar layout.

**Non-Goals:**

- Introducing a WYSIWYG editor or a second document model.
- Adding non-image asset ingest, drag-and-drop ingest, or media-library management.
- Reworking all shortcut customization UX beyond the conflict checks required for editor input safety.
- Changing the existing document-relative image save policy unless needed to fix paste/attach regressions.
- Replacing the current editor with a detached/custom input window in v0.22.5; that remains a design follow-up because it changes focus, IME, accessibility, and persistence behavior.

## Decisions

### 1. Treat editor input as a protected shortcut context

When the editor text widget has focus, protected text-entry shortcuts MUST pass through to the editor/OS instead of being consumed by app-level command dispatch. The protected set includes ordinary text input, IME composition, cursor movement, selection movement, copy, cut, undo, redo, select all, text paste, newline, indentation, backspace, delete, and platform equivalents.

Implementation direction:

- Keep `ShortcutContext::Editor`, but add an explicit protected-shortcut check before `ctx.input_mut(|i| i.consume_shortcut(...))` in `shell_ui_shortcuts.rs`.
- Do not rely only on command context metadata. Global commands must not fire over protected editor input keys.
- Remove or change default editor command shortcuts that collide with expected input behavior. The command palette and toolbar remain available for those commands.
- Add unit tests proving that protected combinations are not consumed as app commands in editor context, and that non-conflicting editor commands can still run.

Alternative considered: removing all global shortcuts while editing. This would be safe but too broad; non-conflicting global commands can remain useful if they do not interfere with text entry.

### 2. Normal paste handles images, explicit command handles attach

Normal paste must continue to be a text-edit operation when clipboard text exists. Clipboard image ingest should run from the same paste path only when image data is available for the active editable Markdown document and the paste would otherwise not insert text.

Implementation direction:

- Keep `AppAction::IngestImageFile` as a command inventory action so it appears in the command palette.
- Do not make `primary+V` a normal command inventory shortcut for image ingest. That would steal text paste from the editor.
- Add a paste-event bridge in the editor input path that detects image clipboard data and dispatches image ingest only for image paste cases.
- Keep the explicit `IngestClipboardImage` action as a fallback command if useful, but the release acceptance path is normal paste.
- Add tests for text clipboard pass-through and image clipboard ingest.

Alternative considered: using only `primary+shift+V` for image paste. This does not satisfy the release feedback because users expect normal paste to work for clipboard images.

### 3. Authoring controls are contextual editor input, not persistent release chrome

The input support controls must appear when the user is editing Markdown source, not as a permanent strip that competes with the preview-first surface. Editable Markdown input should show the controls as a foreground popup under the current input cursor. Read-only/reference/virtual documents must not show mutating controls.

Implementation direction:

- Keep toolbar rendering near `EditorContent::show`, but render it through a foreground popup anchored to the `TextEdit` cursor rect after the current cursor range is known.
- Ensure toolbar layout uses shrink-to-fit sizing and does not create an expanding top panel.
- Toolbar buttons dispatch authoring actions without taking ownership of normal text input.
- Keep icon groups separated with centered `|` separators so grouped controls remain scannable.
- Add a UI or integration check that focuses an editable Markdown editor and verifies the contextual controls are reachable.

### 4. Newly opened documents default to preview

KatanA is primarily a reading tool. If there is no explicit runtime view-mode choice for the active tab, the active document must resolve to preview mode even when the document is editable.

Implementation direction:

- Change the top-level default view-mode resolver to return `PreviewOnly` for editable, reference, and virtual documents.
- Preserve explicit per-tab user choices stored in `tab_view_modes`; the default only applies when no explicit mode exists.
- Keep `ToggleCodePreview`, `ToggleSplitMode`, and explicit mode controls unchanged so users can still enter authoring surfaces intentionally.

### 5. Scroll recovery must be manually controllable

Manual upward scrolling should remain possible even after scroll sync, cursor-line highlighting, toolbar actions, or `scroll_to_line` navigation. If the user scrolls upward, stale programmatic scroll requests must not repeatedly pull the editor back down.

Implementation direction:

- Audit `ScrollState::scroll_to_line`, `last_scroll_to_line`, `ScrollSource`, and editor/preview echo handling.
- Clear one-shot scroll requests after they are consumed.
- Ensure manual editor scroll updates set the source only when the movement exceeds the dead zone and does not fight a user's upward wheel/trackpad input.
- Add tests around `handle_scroll_to_line` and `update_scroll_sync` for top recovery.

### 6. Verification must match the release feedback

The implementation is complete only when the following user review cases are demonstrated:

- Markdown text can be typed into an editable document.
- Newly opened documents default to preview mode.
- The authoring controls appear as a cursor-adjacent popup during editable input and can trigger Markdown insertion.
- The editor can scroll back to the document top after being scrolled down.
- Command palette search can find and run image file attach.
- Pasting a clipboard image with normal paste saves an asset and inserts Markdown image syntax.
- Pasting text with normal paste still inserts text.

## Risks / Trade-offs

- **Risk: app shortcuts become unavailable while editing.**
  Mitigation: suppress only protected text-entry conflicts and keep toolbar/palette routes for authoring commands.

- **Risk: clipboard APIs differ by platform.**
  Mitigation: keep image extraction behind the existing ingest action/path and report unsupported clipboard image data without changing text paste behavior.

- **Risk: toolbar focus steals the editor context.**
  Mitigation: make toolbar actions explicit button clicks and avoid keyboard handling in the toolbar itself.

- **Risk: scroll sync reintroduces downward pull after top recovery.**
  Mitigation: test one-shot scroll clearing and editor/preview source transitions after manual upward scroll.

## Migration Plan

1. Add regression tests or reproducible checks for the release feedback cases before changing behavior.
2. Implement protected editor shortcut handling and remove conflicting default bindings.
3. Move editor authoring controls to the contextual popup and verify Markdown authoring action dispatch.
4. Add command palette coverage for image attach and normal paste coverage for clipboard images.
5. Fix scroll recovery and run focused tests before the full `make check` gate.

## Open Questions

- Whether clipboard image paste should take priority when the clipboard contains both text and image data. The default implementation should prefer text paste unless product review explicitly wants image priority.
- Whether to keep `primary+shift+V` as a secondary image paste command after normal paste is implemented.
- Whether a future release should introduce a native input surface to bypass egui limitations around colored emoji, OS emoji fonts, and special-character rendering. This is now tracked separately by `x-x-x-native-input-surface` because it creates new interaction contracts for cursor anchoring, IME composition, focus transfer, undo/redo, accessibility, and preview synchronization.
