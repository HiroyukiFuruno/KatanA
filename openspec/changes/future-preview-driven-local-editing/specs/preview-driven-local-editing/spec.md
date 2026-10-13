## ADDED Requirements

### Requirement: Preview exposes editable node targets

The system SHALL expose editable preview node targets through renderer-neutral descriptors produced by the preview adapter.

#### Scenario: Discover editable nodes after rendering

- **WHEN** the active Markdown buffer is rendered in preview
- **THEN** the preview adapter returns editable node descriptors for supported Markdown nodes
- **THEN** each descriptor includes a stable node identity, node kind, source range, current source snippet, supported edit commands, and hit-test metadata

#### Scenario: Keep parser internals out of UI state

- **WHEN** KatanA UI stores the currently selected editable node
- **THEN** it stores adapter-owned descriptor data
- **THEN** it does not store parser nodes, vendor widget state, or renderer-specific private references

### Requirement: Preview node actions open local edit sessions

The system SHALL allow the user to start a local edit session from a selected preview node.

#### Scenario: Open a block edit session

- **WHEN** the user activates edit on a supported paragraph, heading, fenced code, diagram, math, or table node in preview
- **THEN** the system opens a local edit surface scoped to that node
- **THEN** the rest of the document remains in preview mode

#### Scenario: Open an inline metadata edit session

- **WHEN** the user activates edit on a supported link or image node in preview
- **THEN** the system opens a local edit surface for the URL, label, alt text, title, or equivalent metadata supported by that node
- **THEN** the system does not require full-document source editing for that local change

### Requirement: Local edit commit patches the in-memory Markdown buffer

The system SHALL commit local preview edits by applying validated source range patches to the in-memory Markdown buffer.

#### Scenario: Commit a valid local edit

- **WHEN** the user confirms a local edit and the target source range still matches the expected original content
- **THEN** the system replaces only that source range with the edited Markdown source
- **THEN** the active Markdown buffer is marked dirty
- **THEN** preview re-renders from the updated in-memory buffer

#### Scenario: Reject a stale local edit

- **WHEN** the user confirms a local edit but the target source range no longer matches the expected original content or buffer snapshot
- **THEN** the system does not apply the patch
- **THEN** the user is given a recoverable stale-edit state that requires reselecting or refreshing the target node

### Requirement: Preview-driven editing remains adapter-isolated

The system MUST keep preview-driven editing isolated behind adapter contracts rather than coupling KatanA UI to parser or renderer internals.

#### Scenario: Add a new editable node kind

- **WHEN** support for a new preview-editable node kind is added
- **THEN** the adapter exposes the node through descriptor and command DTOs
- **THEN** KatanA UI consumes those DTOs without depending on the renderer's internal node representation

#### Scenario: Change the underlying renderer

- **WHEN** the underlying Markdown renderer changes
- **THEN** existing preview-driven edit flows continue to use the adapter descriptor and patch command contract
- **THEN** renderer-specific changes remain inside the adapter implementation

### Requirement: Domain-specific block editors are available for rich Markdown blocks

The system SHALL provide local edit surfaces for domain-specific Markdown blocks used by KatanA preview workflows.

#### Scenario: Edit a diagram block

- **WHEN** the user edits a Mermaid or Draw.io block from preview
- **THEN** the system opens a block editor containing the raw fenced block payload or a structured editor for that diagram kind
- **THEN** commit writes the updated fenced block payload back to the in-memory Markdown buffer

#### Scenario: Edit a math or code block

- **WHEN** the user edits a math or fenced code block from preview
- **THEN** the system opens a block editor preserving the fence language and payload boundaries
- **THEN** commit updates the block without rewriting unrelated Markdown content

#### Scenario: Edit a table block

- **WHEN** the user edits a Markdown table from preview
- **THEN** the system provides a table-oriented local edit surface or a raw table source fallback scoped to that table
- **THEN** commit updates only the table source range

### Requirement: Cancel and dirty-state semantics are explicit

The system SHALL keep cancel, dirty, save, and inspector behavior explicit for preview-driven local edits.

#### Scenario: Cancel a local edit

- **WHEN** the user cancels a local edit session
- **THEN** the in-memory Markdown buffer remains unchanged
- **THEN** the document dirty state remains unchanged

#### Scenario: Inspect source without editing

- **WHEN** the user opens the source inspector for a preview node or active document
- **THEN** the inspector presents the relevant Markdown source without modifying the in-memory buffer
- **THEN** direct source modification is unavailable unless the user explicitly enters fallback source-edit mode
