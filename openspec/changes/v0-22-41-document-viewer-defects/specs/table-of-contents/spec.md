## ADDED Requirements

### Requirement: Table of contents availability follows document semantics
KatanA MUST disable the table-of-contents control and reject its toggle action for DOCX, XLSX, and PPTX documents. PDF and Markdown documents MUST retain the control.

#### Scenario: Office document is active
- **WHEN** the active document is DOCX, XLSX, or PPTX
- **THEN** the table-of-contents control MUST be inactive and the TOC panel state MUST remain closed

### Requirement: PDF outline hierarchy navigates parsed destinations
When KDV provides PDF outline items, KatanA MUST display their titles with hierarchy and MUST navigate to the parsed page destination when an item is selected.

#### Scenario: PDF with an embedded outline is active
- **WHEN** the user selects a nested PDF outline item with a page destination
- **THEN** KatanA MUST send one index navigation command for that destination and mark the destination page active
