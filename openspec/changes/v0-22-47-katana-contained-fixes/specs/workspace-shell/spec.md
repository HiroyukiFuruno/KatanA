## ADDED Requirements

### Requirement: Split backgrounds preserve surrounding panels

Split backgrounds SHALL be restricted to the remaining content rectangle after top, side, and status panels have reserved their regions. They SHALL NOT paint over surrounding panels in either split orientation or in light and dark themes.

#### Scenario: Render split content after surrounding panels

- **WHEN** a shared root UI reserves surrounding panels before rendering split editor and preview panes
- **THEN** the background fill stays within the available content rectangle and previously drawn panels remain visible

#### Scenario: Capture the full workspace after changing view mode

- **WHEN** the real application changes from preview-only to split mode
- **THEN** the file tree, upper controls, editor, preview, and status remain visible without synthetic image composition or an added fixed wait
