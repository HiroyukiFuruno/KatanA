## Purpose

This is a legacy capability specification that was automatically migrated to comply with the new OpenSpec schema validation rules. Please update this document manually if more context is required.

## Requirements

### Requirement: Markdown documents can be edited as local workspace files

The system SHALL allow the active Markdown document to be edited in memory and saved back to its workspace file.

#### Scenario: Modify the active Markdown document

- **WHEN** the user types into the active Markdown editor
- **THEN** the system updates the in-memory document buffer
- **THEN** the document is marked as having unsaved changes

#### Scenario: Save edits to disk

- **WHEN** the user saves a dirty Markdown document
- **THEN** the system writes the current buffer to the document's file path in the active workspace
- **THEN** the document is marked as clean after a successful write

#### Scenario: Editing does not implicitly save the source file

- **WHEN** the active Markdown buffer changes without an explicit save action
- **THEN** the workspace file contents remain unchanged on disk
- **THEN** the document remains marked as having unsaved changes

### Requirement: Preview rendering stays synchronized with the active buffer

The system SHALL render preview output from the current in-memory Markdown buffer rather than from the last saved file contents.

#### Scenario: Update preview after an edit

- **WHEN** the active Markdown buffer changes
- **THEN** the preview renderer uses the updated buffer contents
- **THEN** the preview pane reflects the edit without requiring the file to be saved first

### Requirement: GitHub Flavored Markdown is supported in preview output

The system SHALL parse and render GitHub Flavored Markdown constructs supported by the chosen Markdown engine.

#### Scenario: Render common GFM structures

- **WHEN** the active document contains headings, lists, fenced code blocks, and tables supported by the Markdown engine
- **THEN** the preview output preserves those structures in rendered form
- **THEN** unsupported content degrades gracefully without crashing the application

### Requirement: Documents open in preview mode by default

The system SHALL resolve the active view mode to preview mode when the active document has no explicit user-selected view mode.

#### Scenario: Editable Markdown defaults to preview

- **WHEN** the user opens an editable Markdown document
- **THEN** the system displays the document in preview mode by default
- **THEN** the system MUST NOT switch the document to code mode unless the user explicitly requests an editing view

#### Scenario: Explicit view selection remains effective

- **WHEN** the user explicitly switches the active document to code or split mode
- **THEN** the system applies the selected mode for that active document
- **THEN** later default-view resolution MUST NOT override that explicit active-tab selection

### Requirement: Editor input mode protects native text entry

The system MUST NOT consume editor-owned or OS-owned text-entry shortcuts as application commands while an editable Markdown editor has keyboard focus.

#### Scenario: Markdown text input is preserved

- **WHEN** the user focuses an editable Markdown document and types Markdown source text
- **THEN** the system updates the in-memory document buffer with the typed Markdown
- **THEN** the system does not trigger unrelated application commands from the typed input

#### Scenario: Protected editor shortcut is not consumed by app command dispatch

- **WHEN** the editor has keyboard focus and the user presses a protected text-entry shortcut such as paste, copy, cut, undo, redo, select all, cursor movement, selection movement, newline, indentation, backspace, or delete
- **THEN** the system MUST allow the editor or operating system input method to handle that shortcut
- **THEN** the system MUST NOT dispatch a global or editor authoring command for that shortcut in the same frame

#### Scenario: Non-conflicting editor command remains available

- **WHEN** the editor has keyboard focus and the user invokes an editor command that does not conflict with protected text entry
- **THEN** the system MAY dispatch that editor command
- **THEN** the command MUST preserve the Markdown source-first editing contract

### Requirement: Editable Markdown documents show contextual authoring support UI

The system SHALL show Markdown input support controls as a cursor-adjacent popup while editable Markdown input is active.

#### Scenario: Authoring controls appear under the input cursor

- **WHEN** the user focuses an editable Markdown document in an editing view and the editor has an input cursor
- **THEN** the system shows the authoring controls below the current input cursor
- **THEN** the controls include common Markdown insertion actions such as inline formatting, headings, lists, quotes, code blocks, and image insertion

#### Scenario: Authoring controls are not persistently shown over preview-first UI

- **WHEN** the active document is displayed in preview mode
- **THEN** the system MUST NOT show a persistent editor toolbar
- **THEN** the user can still explicitly switch to an editing view to access input controls

#### Scenario: Authoring control groups are visually separated and aligned

- **WHEN** the system shows multiple authoring control groups in an editable Markdown document
- **THEN** the system separates adjacent groups with visible `|` separators
- **THEN** each separator is vertically centered with the toolbar icon buttons
- **THEN** each separator preserves stable toolbar spacing without shifting icon controls

#### Scenario: Contextual controls stay out of read-only documents

- **WHEN** the active document is read-only, reference-only, or a virtual built-in document
- **THEN** the system MUST NOT show mutating authoring controls for that document

### Requirement: Authoring controls preserve Markdown source editing

The system SHALL apply toolbar and command palette authoring actions by editing Markdown source text in the active buffer.

#### Scenario: Toolbar action inserts Markdown syntax

- **WHEN** the user triggers a toolbar authoring action in an editable Markdown document
- **THEN** the system updates the active Markdown buffer with the corresponding Markdown syntax
- **THEN** the document dirty state and preview refresh behavior follow the same path as ordinary typing

### Requirement: Editor scroll can return to the document start

The system SHALL allow the user to manually scroll an editable Markdown editor back to the top after typing, toolbar actions, search navigation, or preview scroll sync.

#### Scenario: User scrolls back to top after editing

- **WHEN** the user scrolls down in a long editable Markdown document and then scrolls upward to the start of the document
- **THEN** the system displays the beginning of the document again
- **THEN** stale cursor, search, or scroll-sync state MUST NOT force the editor back down

#### Scenario: Scroll sync does not fight manual upward scroll

- **WHEN** split view scroll sync is enabled and the user manually scrolls the editor upward
- **THEN** the editor scroll position follows the user's input
- **THEN** preview synchronization MUST NOT prevent the editor from reaching the top

