## ADDED Requirements

### Requirement: KME platform responsibilities are explicit

The system SHALL define KME, editor, viewer, export, KatanA integration, and UI widget responsibilities before implementation begins.

#### Scenario: Repository responsibilities are split

- **WHEN** KME platform work is planned
- **THEN** `katana-markdown-engine` owns the document model, metadata schema, and target resolution
- **THEN** `katana-language-editor` owns metadata updates on save
- **THEN** `katana-document-viewer` owns Floem viewer rendering and HTML/PDF/PNG/JPG export of KME models
- **THEN** `katana-canvas-forge` owns external rendering for Mermaid, Draw.io, PlantUML, and math
- **THEN** `katana` owns integration and fixture authority
- **THEN** `katana` owns editor-viewer synchronization control and commands viewer or editor

### Requirement: AST lint is separated before KME implementation

The system SHALL prioritize shared AST lint separation before KME implementation work.

#### Scenario: Plan repository separation

- **WHEN** KatanA ecosystem repository separation is planned
- **THEN** P0 is `katana-ast-lint`
- **THEN** P1 is `katana-markdown-engine`
- **THEN** P2 is `katana-ui-widget`
- **THEN** P3 is remaining integration work
- **THEN** each separated repository can use the same AST lint quality gate

### Requirement: KME v0 follows current KatanA behavior

The system SHALL treat current KatanA Markdown behavior as the KME v0 compatibility line.

#### Scenario: Canonical fixtures are used

- **WHEN** KME v0 fixture coverage is defined
- **THEN** `assets/fixtures/sample.md` is the primary fixture
- **THEN** the README badge header is a required operational fixture
- **THEN** alert syntax from `sample_basic.md` is included
- **THEN** description list coverage is added as a missing fixture

### Requirement: Metadata stays external to Markdown

The system MUST NOT require KatanA-specific pagination or LLM annotation syntax inside Markdown files.

#### Scenario: Metadata file is used

- **WHEN** a Markdown document has pagination or LLM annotation metadata
- **THEN** the metadata is stored outside the Markdown body
- **THEN** `README.md.metadata.json` is the standard naming pattern
- **THEN** target identity includes path, node id, ranges, fingerprint, and surrounding context

### Requirement: Floem is the viewer UI baseline

The system SHALL treat Floem as the target UI framework for editor, viewer, and shared widgets.

#### Scenario: KME model is displayed

- **WHEN** the KME model is displayed in KatanA viewer
- **THEN** the target implementation is Floem native rendering
- **THEN** egui-specific parser or widget internals are not exposed as the contract

### Requirement: KatanA owns editor-viewer synchronization

The system SHALL keep editor-viewer synchronization control in KatanA.

#### Scenario: Editor and viewer are synchronized

- **WHEN** KatanA synchronizes editor and viewer positions
- **THEN** KatanA uses KME node id, source range, line-column, raw snippet, or fingerprint as matching material
- **THEN** KatanA sends scroll, selection, or highlight commands to viewer or editor
- **THEN** KME, KLE, and KDV do not know each other's state or issue synchronization commands

### Requirement: Downstream work waits for KME contracts

The system SHALL prevent downstream repositories from defining their own Markdown document model or metadata schema before KME contracts are available.

#### Scenario: A downstream repository starts work before KME is complete

- **WHEN** KDV, KLE, KCF, KUW, or KatanA integration work starts before KME public DTOs are stable
- **THEN** the work may prepare adapters, OpenSpec, repository baseline, or quality gates
- **THEN** the work MUST NOT define a replacement Markdown document model
- **THEN** the work MUST NOT define a repository-local metadata schema as a substitute for KME

### Requirement: KCF export moves to KDV after viewer export is ready

The system SHALL keep existing KCF export only until KDV provides equivalent viewer/export capability.

#### Scenario: KDV export becomes available

- **WHEN** KDV can export HTML/PDF/PNG/JPG from the same render pipeline as viewer display
- **THEN** KCF export-related plans are moved to KDV
- **THEN** KCF keeps Mermaid, Draw.io, PlantUML, and math external rendering
- **THEN** KCF export implementation can be removed after migration

### Requirement: KUW absence is tracked as a blocking planning risk

The system SHALL track missing `katana-ui-widget` repository creation as a P2 planning risk.

#### Scenario: Shared UI behavior is planned

- **WHEN** metadata badges, unresolved metadata, tabs, toolbar, copy, or edit affordance behavior is planned
- **THEN** the plan identifies whether the behavior belongs to KUW
- **THEN** KDV or KatanA does not silently absorb shared widget responsibilities
