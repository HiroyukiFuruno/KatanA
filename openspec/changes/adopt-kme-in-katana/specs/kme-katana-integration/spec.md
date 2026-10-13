## ADDED Requirements

### Requirement: KatanA integrates KMM through public contracts

KatanA SHALL consume KMM, viewer, editor, export, and widget functionality through public contracts without storing parser or renderer internals in application state.

#### Scenario: KatanA stores selected metadata

- **WHEN** the user selects a rendered Markdown node
- **THEN** KatanA stores public descriptor data only
- **THEN** KatanA does not store parser AST nodes, vendor widget state, or renderer-private references

### Requirement: KatanA integration uses shared AST lint gate

KatanA SHALL use the shared AST lint governance before integrating separated KMM ecosystem repositories.

#### Scenario: Validate integration readiness

- **WHEN** KatanA starts integrating KMM, viewer, editor, export, or widget repositories
- **THEN** the P0 `katana-ast-lint` quality gate is available
- **THEN** KatanA does not accept repository-specific lint drift as the integration baseline

### Requirement: KatanA keeps metadata synchronized on save

KatanA SHALL connect editor save flow to KMM metadata target resolution.

#### Scenario: Save a document with metadata

- **WHEN** the active Markdown document is saved after edits
- **THEN** the editor invokes KMM target resolution with old source, new source, and metadata
- **THEN** resolved targets are updated
- **THEN** unresolved targets are preserved for UI review

### Requirement: KatanA validates KMM migration with canonical fixtures

KatanA SHALL use the canonical fixture set before replacing current Markdown behavior with KMM-backed behavior.

#### Scenario: Validate migration readiness

- **WHEN** KMM-backed viewer or export is enabled
- **THEN** `sample.md`, README badge, alert, and description list fixtures are checked
- **THEN** current KatanA behavior is not silently dropped

### Requirement: KatanA owns editor-viewer synchronization control

KatanA SHALL coordinate editor and viewer synchronization itself.

#### Scenario: KatanA synchronizes editor and viewer

- **WHEN** KatanA aligns editor and viewer positions
- **THEN** KatanA uses KMM node id, source range, line-column, raw snippet, and fingerprint
- **THEN** KatanA sends scroll, selection, or highlight commands to the viewer or editor
- **THEN** KMM, KLE, and KDV do not coordinate synchronization with each other
