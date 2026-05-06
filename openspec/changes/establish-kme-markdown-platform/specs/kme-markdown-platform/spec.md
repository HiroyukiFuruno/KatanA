## ADDED Requirements

### Requirement: KME platform responsibilities are explicit

The system SHALL define KME, editor, preview, export, KatanA integration, and UI widget responsibilities before implementation begins.

#### Scenario: Repository responsibilities are split

- **WHEN** KME platform work is planned
- **THEN** `katana-markdown-engine` owns the document model, metadata schema, and target resolution
- **THEN** `katana-language-editor` owns metadata updates on save
- **THEN** `katana-document-preview` owns Floem preview rendering of KME models
- **THEN** `katana-canvas-forge` owns export quality gates and later high-fidelity export
- **THEN** `katana` owns integration and fixture authority

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

### Requirement: Floem is the UI baseline

The system SHALL treat Floem as the target UI framework for editor, preview, and shared widgets.

#### Scenario: KME model is displayed

- **WHEN** the KME model is displayed in KatanA preview
- **THEN** the target implementation is Floem native rendering
- **THEN** egui-specific parser or widget internals are not exposed as the contract
