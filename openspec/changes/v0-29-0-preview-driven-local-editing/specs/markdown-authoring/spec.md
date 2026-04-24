## MODIFIED Requirements

### Requirement: Markdown documents can be edited as local workspace files

The system SHALL allow the active Markdown document to be edited in memory through preview-driven local edits and, when explicitly enabled, fallback source-edit mode, then saved back to its workspace file.

#### Scenario: Modify the active Markdown document from preview

- **WHEN** the user confirms a local edit that was opened from an editable preview node
- **THEN** the system applies the corresponding source range patch to the in-memory document buffer
- **THEN** the document is marked as having unsaved changes

#### Scenario: Save edits to disk

- **WHEN** the user saves a dirty Markdown document
- **THEN** the system writes the current buffer to the document's file path in the active workspace
- **THEN** the document is marked as clean after a successful write

#### Scenario: Editing does not implicitly save the source file

- **WHEN** the active Markdown buffer changes without an explicit save action
- **THEN** the workspace file contents remain unchanged on disk
- **THEN** the document remains marked as having unsaved changes

#### Scenario: Inspect source without changing the document

- **WHEN** the user views Markdown source through the read-only source inspector
- **THEN** the in-memory document buffer remains unchanged
- **THEN** the document dirty state remains unchanged

#### Scenario: Use fallback source-edit mode explicitly

- **WHEN** fallback source-edit mode is enabled and the user edits the active Markdown source directly
- **THEN** the system updates the in-memory document buffer
- **THEN** the document is marked as having unsaved changes
