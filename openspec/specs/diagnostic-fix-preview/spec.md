## Purpose

Diagnostic fix preview defines how KatanA presents proposed Markdown lint fixes before applying them, so users can understand and approve file changes before content is modified.

## Requirements

### Requirement: Preview Diagnostic Fixes before applying

The system SHALL provide a file-level diff review flow before applying manual or automated fixes that modify file content.

#### Scenario: User applies fixes for a single file from the Problems panel

- **WHEN** the user requests a file-level fix from the Problems panel
- **THEN** the system displays a diff review screen before modifying the file
- **AND** the diff clearly distinguishes removed lines from added lines
- **AND** the screen provides "Cancel" and "Apply Fix" actions
- **AND** the file content is modified only after the user chooses "Apply Fix"

#### Scenario: User applies fixes across multiple files

- **WHEN** the user requests a workspace-level or multi-file fix
- **THEN** the system displays one file diff at a time
- **AND** the user can move through the file diffs like pages
- **AND** the user can accept or reject each file's changes independently

#### Scenario: LLM-generated fixes are ready to apply

- **WHEN** an LLM-generated fix produces proposed content changes for one or more files
- **THEN** the system routes those changes through the same file-level diff review flow
- **AND** no LLM-generated content is written to files before user approval

#### Scenario: User changes the default diff display mode

- **WHEN** the user changes the persistent diff display setting
- **THEN** future diff review screens use the selected mode by default
- **AND** the setting persists across application restarts

#### Scenario: User temporarily switches diff display mode

- **WHEN** the user switches between Split and Inline inside an open diff review screen
- **THEN** the visible diff changes display mode immediately
- **AND** the persistent setting is not changed by that temporary switch

#### Scenario: Diff review opens with default Split mode

- **WHEN** no persistent diff display setting has been changed
- **THEN** the diff review screen opens in Split mode

#### Scenario: User hovers over the Fix button in the Problems panel

- **WHEN** the user hovers the cursor over a "Fix" button associated with a Diagnostic in the Problems panel
- **THEN** the system displays a Tooltip containing a preview of the text changes
- **AND** the preview clearly distinguishes between the original text being removed and the new text being inserted
- **AND** clicking the Fix button still routes file content modification through the file-level diff review flow

#### Scenario: Diagnostic has no associated fix

- **WHEN** the user views a Diagnostic that does not contain any `DiagnosticFix`
- **THEN** no Fix button is displayed, and therefore no preview is available
