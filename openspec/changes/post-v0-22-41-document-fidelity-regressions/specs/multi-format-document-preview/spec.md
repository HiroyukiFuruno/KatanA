## ADDED Requirements

### Requirement: Office-inapplicable preview menus remain inactive

KatanA MUST disable table-of-contents, export/download, story/slideshow, and tools/view preview panels whenever the active DOCX, XLSX, or PPTX viewer does not support the corresponding operation. Disabled panels MUST reject click, hover-open, shortcut, and direct action dispatch.

#### Scenario: Office tab is active

- **WHEN** an Office document is the active tab
- **THEN** each unsupported preview panel button is visibly disabled
- **AND THEN** no unsupported panel state can be opened through another input path

### Requirement: XLSX sheet tabs stay on the bottom edge

KatanA MUST reserve a persistent bottom tab rail before allocating the spreadsheet grid. The rail MUST use KDV worksheet labels and MUST switch sheets through the typed KDV `JumpTo` command.

#### Scenario: Multi-sheet workbook fills the preview

- **WHEN** an XLSX grid consumes all remaining document viewport space
- **THEN** the worksheet tab rail remains visible at the bottom edge
- **WHEN** a worksheet tab is activated
- **THEN** the corresponding KDV sheet becomes active without page-style previous/next navigation

### Requirement: XLSX filters use viewer-owned spreadsheet state

KDV MUST expose parsed AutoFilter or table-filter metadata, candidate values, active criteria, and filtered row visibility through typed spreadsheet surface state and commands. KatanA MUST project that state without reparsing displayed cell strings.

#### Scenario: Filter a workbook column

- **WHEN** the active worksheet contains a supported filter range and the user selects criteria
- **THEN** KDV applies the filter and publishes the resulting visible rows and active criteria
- **AND THEN** KatanA displays the filter state and updated grid

#### Scenario: Worksheet has no filter metadata

- **WHEN** the active worksheet has no supported filter range
- **THEN** KatanA does not display a nonfunctional filter affordance

### Requirement: Standards-compliant OOXML ZIP variants open safely

The Office intake path MUST accept standards-compliant DOCX, XLSX, and PPTX archives that use data descriptors or valid central-directory sizes even when the local header omits the final size. It MUST retain entry-count, expanded-byte, ratio, timeout, encryption, and path-safety protections.

#### Scenario: Valid data-descriptor OOXML is opened

- **WHEN** a supplied DOCX, XLSX, or PPTX contains a valid ZIP data descriptor and central directory
- **THEN** the document reaches the expected Page or Grid frame
- **AND THEN** no generic `zip archive` or `file length is not available in local header` error is shown

#### Scenario: Unsafe archive is opened

- **WHEN** an archive exceeds a safety limit, is encrypted, or contains an unsafe path
- **THEN** the viewer rejects it with the exact entry, limit, and owner layer

### Requirement: Office fidelity is measured against the source renderer

DOCX and XLSX rendering MUST preserve supported text, fonts, borders, fills, merged cells, row and column geometry, and pagination or worksheet structure. PPTX MUST preserve supported text, shapes, tables, images, charts, and slide geometry. Fixture acceptance MUST use objective geometry and missing-element comparisons in addition to screenshots.

#### Scenario: Compare Office fixtures

- **WHEN** each supplied Office fixture is rendered at the agreed reference viewport
- **THEN** the harness records missing elements and geometry deltas against the source-renderer reference
- **AND THEN** a regression outside the recorded tolerance fails acceptance

### Requirement: PPTX first display exposes and controls stage latency

The Office path MUST report and bound source intake, ZIP preflight, conversion, PDF/session open, first-page render, and host texture-upload time. It MUST NOT repeat conversion or main-thread work when an unchanged document is reopened within the supported cache lifetime.

#### Scenario: Open the supplied PPTX from a cold session

- **WHEN** a supplied PPTX is opened from a cold packaged-app session with `DEBUG=true`
- **THEN** every stage emits one elapsed-time record and the first complete frame time is measured
- **AND THEN** the dominant avoidable stage is covered by a regression gate
