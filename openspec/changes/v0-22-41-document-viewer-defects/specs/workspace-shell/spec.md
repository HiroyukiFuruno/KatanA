## ADDED Requirements

### Requirement: Opening a binary document reveals its explorer entry
Opening a PDF, DOCX, XLSX, or PPTX document MUST select the corresponding file in the workspace explorer and MUST expand its parent directories so the active entry is visible.

#### Scenario: Binary document is opened outside an explorer click
- **WHEN** a PDF or Office document becomes the active document through any supported open path
- **THEN** the explorer MUST select and reveal the matching workspace-relative file
