## ADDED Requirements

### Requirement: XLSX navigation uses named bottom sheet tabs
KatanA MUST render XLSX navigation as horizontally scrollable tabs below the worksheet and MUST label them with KDV-provided worksheet names rather than page numbers.

#### Scenario: Workbook with multiple named sheets is opened
- **WHEN** KDV returns multiple worksheet labels
- **THEN** KatanA MUST display those labels as bottom tabs and selecting a tab MUST navigate to its worksheet index

### Requirement: v0.22.41 consumes the complete published document fix chain
KatanA v0.22.41 MUST consume exact published KDV 0.5.5 and KRR 0.4.17 from crates.io. The resolved KDV chain MUST use published `office2pdf-katana 0.6.10` and MUST NOT use path or git overrides.

#### Scenario: Release dependency provenance is checked
- **WHEN** the v0.22.41 lockfiles and package graph are validated
- **THEN** KDV, KRR, and the Office conversion fork MUST resolve from the public registry at the required versions

### Requirement: Every supplied Office defect is accepted through the real viewer path
The release MUST open all six supplied Office fixtures, including frozen merged cells, blocked active content, ZIP data descriptors, the large worksheet, PPTX colspan, and malformed PPTX PNG cases, without weakening bounded safety policy.

#### Scenario: Supplied Office fixture suite runs
- **WHEN** all supplied Office fixtures are exercised through KDV's worker-backed acceptance path
- **THEN** every fixture MUST open successfully and safe static display MUST NOT execute active content
