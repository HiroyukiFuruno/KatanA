## ADDED Requirements

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

### Requirement: Editable Markdown documents show authoring support UI

The system SHALL show the Markdown input support toolbar for editable Markdown documents without requiring a stale cursor selection state.

#### Scenario: Toolbar appears for editable Markdown

- **WHEN** the user opens an editable Markdown document in editor mode
- **THEN** the system shows the authoring toolbar in the editor panel
- **THEN** the toolbar includes controls for common Markdown insertion actions such as inline formatting, headings, lists, quotes, and code blocks

#### Scenario: Toolbar groups are visually separated and aligned

- **WHEN** the system shows multiple authoring toolbar groups in an editable Markdown document
- **THEN** the system separates adjacent groups with visible `|` separators
- **THEN** each separator is vertically centered with the toolbar icon buttons
- **THEN** each separator preserves stable toolbar spacing without shifting icon controls

#### Scenario: Toolbar stays out of read-only documents

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
