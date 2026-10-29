## ADDED Requirements

### Requirement: All document controls share format and runtime policy

KatanA SHALL derive every vertical sidebar operation, code/split mode, shortcut, command palette entry and direct action guard from one document operation settings policy. PPTX slideshow readiness SHALL come from the current live surface; binary PDF/Office documents SHALL reject unsupported code/split modes.

#### Scenario: Ready PPTX and unsupported binary mode

- **WHEN** a current PPTX surface becomes ready or a binary document receives a mode command
- **THEN** slideshow controls reflect the live readiness at every entry, while unsupported code/split controls and actions remain unavailable

#### Scenario: Same path reopened or active tab changes

- **WHEN** the active document identity, revision or preview generation changes
- **THEN** stale runtime readiness is not reused and palette/sidebar/shortcut/direct-dispatch decisions remain consistent

### Requirement: Document loading uses one localized presentation

KatanA SHALL use one reusable loading component for images, HTML, Office/PDF, Canvas and section loading, passing the local state and resolved localized text without exposing template placeholders.

#### Scenario: Pending document frame

- **WHEN** any supported document is waiting for its first frame
- **THEN** the shared loading presentation is shown and no literal `{kind}` placeholder appears

### Requirement: HTML and Office close preserve UI progress and resource cleanup

KatanA SHALL separate close-action response from background teardown, preserve eventual zero-resource cleanup and the existing memory budget, and record HTML and Office phase timings independently before claiming a common root cause.

#### Scenario: Close a loaded HTML or Office document

- **WHEN** a loaded HTML, DOCX, XLSX or PPTX tab is closed
- **THEN** the remaining interface can progress and teardown reaches the existing idle/resource and memory conditions without weakening them

### Requirement: Office embedded font names are checked before conversion

KatanA SHALL apply the same bounded DOCX/PPTX embedded-font validation at local and remote source intake before handing bytes to the document worker. Embedded font names containing path separators, colon or NUL, ambiguous archive names and malformed XML SHALL be rejected without writing extracted fonts. Normal Unicode names SHALL remain accepted. Existing source and metadata size limits SHALL be preserved.

#### Scenario: Unsafe embedded font name from local or remote input

- **WHEN** a DOCX or PPTX contains a known embedded-font marker and an unsafe font name, including encoded XML attribute forms
- **THEN** the shared source intake reports a validation error before conversion and the document worker does not receive those bytes

#### Scenario: Normal input and other document formats

- **WHEN** a normal Unicode font name or an ordinary Office document is opened
- **THEN** the shared validation accepts it within the existing limits
- **AND** PDF and XLSX do not execute the DOCX/PPTX font parser, while their existing format checks remain in effect
