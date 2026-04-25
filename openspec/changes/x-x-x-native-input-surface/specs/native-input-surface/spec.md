## ADDED Requirements

### Requirement: Native input model is independent from egui TextEdit

The system SHALL provide an input model for Markdown text entry that owns buffer edits, cursor state, selection state, composition state, undo/redo history, and pending authoring operations without relying on `egui::TextEdit` as the source of truth.

#### Scenario: Input model applies a text edit

- **WHEN** a user inserts or deletes text through the native input surface
- **THEN** the system updates the input model state and emits a buffer-change event without reading cursor, selection, or mutation state from `egui::TextEdit`

#### Scenario: Authoring command updates cursor state

- **WHEN** a Markdown authoring command wraps a selection or inserts a snippet
- **THEN** the system stores the resulting cursor and selection in the input model so the next frame can render the correct caret location

### Requirement: Native input surface preserves platform text-entry behavior

The system MUST preserve platform text-entry behavior for normal typing, IME composition, selection movement, clipboard text paste, clipboard image paste, copy, cut, undo, redo, select all, newline, indentation, backspace, and delete.

#### Scenario: IME composition remains stable

- **WHEN** a user enters Japanese or another composition-based language through the native input surface
- **THEN** the system keeps the composition range, committed text, caret position, and preview synchronization stable throughout the composition lifecycle

#### Scenario: Clipboard image paste remains available

- **WHEN** a focused native input surface receives a paste event and the clipboard contains image data
- **THEN** the system inserts or attaches the image at the current input-model cursor using the same image-ingest behavior as the existing Markdown editor flow

### Requirement: Input rendering avoids egui emoji coupling

The system SHALL isolate input text rendering from egui `TextEdit` and egui-owned emoji fallback behavior so emoji and special characters can be handled by a controlled KatanA input-rendering boundary.

#### Scenario: Emoji text is rendered by the input surface boundary

- **WHEN** Markdown source text contains emoji or special characters that egui would otherwise render inconsistently across platforms
- **THEN** the native input surface renders the text through KatanA-controlled font, raster, or layout behavior rather than delegating the input text run to `egui::TextEdit`

#### Scenario: Hit testing matches rendered graphemes

- **WHEN** a user clicks, drags, or moves the caret across emoji, combining characters, or multi-codepoint grapheme clusters
- **THEN** the system maps the pointer or keyboard movement to the correct input-model cursor position without splitting an invalid grapheme cluster

### Requirement: Existing editor and preview changes keep separate responsibilities

The system MUST keep the native input surface as a distinct deferred capability from the preview-first local-correction direction pursued by v0.27.0 and v0.29.0.

#### Scenario: v0.27.0 remains editor boundary work

- **WHEN** v0.27.0 extracts editor logic into `katana-editor`
- **THEN** that change may expose interfaces needed by the native input surface but MUST NOT be required to replace `egui::TextEdit`

#### Scenario: v0.29.0 remains preview-local editing work

- **WHEN** v0.29.0 adds preview-driven local edit surfaces
- **THEN** those surfaces may later adopt the native input model but MUST NOT own the full source-input replacement architecture

### Requirement: Native input surface remains version-undecided until prioritized

The system MUST treat the native input surface as a version-undecided deferred option until a future decision explicitly confirms its product value and release target.

#### Scenario: Future planning reviews the deferred option

- **WHEN** release planning reviews `x-x-x-native-input-surface`
- **THEN** the team decides whether source-input emoji correctness is worth the implementation cost before assigning a version or implementation branch

#### Scenario: Existing preview-first changes continue independently

- **WHEN** v0.27.0 or v0.29.0 is implemented before this deferred option is prioritized
- **THEN** those changes continue using their preview-first local-correction approach without being blocked by the native input surface

### Requirement: Rollout preserves fallback editing until parity is proven

The system SHALL keep a verified fallback source-edit path available until the native input surface has parity for input correctness, scroll behavior, preview synchronization, accessibility, and save/dirty semantics.

#### Scenario: Native input surface is not yet production-ready

- **WHEN** the native input surface lacks parity for IME, emoji, accessibility, or undo/redo behavior
- **THEN** the system keeps the existing repaired source-edit path available and does not make the native input surface the only editing path
