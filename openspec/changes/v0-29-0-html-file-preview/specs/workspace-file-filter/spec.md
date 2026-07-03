## ADDED Requirements

### Requirement: HTML files are standard visible and openable workspace files

The system SHALL treat `.html` and `.htm` as standard visible and openable workspace file extensions.

#### Scenario: HTML appears in the workspace tree

- **WHEN** a workspace directory contains `index.html`
- **THEN** the workspace tree includes `index.html` in the visible file list

#### Scenario: HTM appears in the workspace tree

- **WHEN** a workspace directory contains `legacy.htm`
- **THEN** the workspace tree includes `legacy.htm` in the visible file list

#### Scenario: Existing workspace filtering still applies

- **WHEN** the workspace filter is active with a pattern that does not match `index.html`
- **THEN** the workspace tree hides `index.html` according to the existing filtering rules
- **THEN** HTML visibility does not bypass the user's workspace filter
