## ADDED Requirements

### Requirement: Preview Diagnostic Fixes before applying
The system SHALL provide a mechanism to preview the modifications that will be applied by a DiagnosticFix before the user actually applies the fix.

#### Scenario: User hovers over the Fix button in the Problems panel
- **WHEN** the user hovers the cursor over a "Fix" button associated with a Diagnostic in the Problems panel
- **THEN** the system displays a Tooltip containing a preview of the text changes
- **AND** the preview clearly distinguishes between the original text being removed and the new text being inserted

#### Scenario: Diagnostic has no associated fix
- **WHEN** the user views a Diagnostic that does not contain any `DiagnosticFix`
- **THEN** no Fix button is displayed, and therefore no preview is available
