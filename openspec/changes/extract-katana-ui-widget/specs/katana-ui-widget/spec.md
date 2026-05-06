## ADDED Requirements

### Requirement: Shared UI widgets are separated for Floem

The system SHALL separate reusable Floem UI widgets from KatanA application-specific shell code.

#### Scenario: Metadata UI is reused

- **WHEN** KME metadata is displayed in preview or editor flows
- **THEN** shared metadata UI can be provided by `katana-ui-widget`
- **THEN** KatanA shell code does not own reusable metadata rendering logic

### Requirement: UI widget extraction waits for P0 and P1 contracts

The system SHALL treat `katana-ui-widget` as P2 after shared AST lint and KME contracts.

#### Scenario: Start UI widget extraction

- **WHEN** `katana-ui-widget` extraction begins
- **THEN** P0 `katana-ast-lint` governance is available
- **THEN** P1 KME metadata/display DTO direction is available
- **THEN** widget display types are not fixed ahead of KME contracts

### Requirement: UI widgets do not own KME internals

Shared UI widgets MUST NOT depend on KME parser internals or metadata storage internals.

#### Scenario: Render unresolved metadata

- **WHEN** an unresolved metadata target is rendered
- **THEN** the widget receives a public display DTO
- **THEN** the widget does not receive parser AST nodes or KME private types
