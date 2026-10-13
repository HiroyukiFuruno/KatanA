## ADDED Requirements

### Requirement: Reference mode displays code in the existing code pane without enabling editing

The system MUST present reference-mode documents in the current tab/editor surface while keeping them non-editable.

#### Scenario: Reference document opens in the code surface

- **WHEN** the system opens a demo-bundle code asset in reference mode
- **THEN** the active tab shows the file in the existing code-pane surface
- **THEN** the file is not opened in an editable text-entry path

#### Scenario: User attempts to type into a reference document

- **WHEN** the active document is in reference mode and the user types, pastes, or otherwise attempts to modify the content
- **THEN** the visible content remains unchanged
- **THEN** the document does not become dirty

### Requirement: Reference mode blocks save-driven mutation

The system MUST reject save and buffer-mutation flows for reference-mode documents.

#### Scenario: Save is invoked for a reference document

- **WHEN** the active document is in reference mode and the user invokes Save
- **THEN** the system does not write the file back to disk
- **THEN** the reference document remains clean and unchanged

#### Scenario: Non-UI mutation path targets a reference document

- **WHEN** an internal update or replace-text path is invoked for a reference-mode document
- **THEN** the request is ignored
- **THEN** the stored buffer remains identical to the loaded reference content
