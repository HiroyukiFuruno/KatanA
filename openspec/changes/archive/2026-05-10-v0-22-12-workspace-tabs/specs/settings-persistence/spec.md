## ADDED Requirements

### Requirement: Workspace tab preference is persisted

The system SHALL persist the user's preference for opening workspaces in tabs.

#### Scenario: Save workspace tab preference

- **WHEN** the user changes the setting for opening workspaces in tabs
- **THEN** the system saves the setting to the application settings JSON

#### Scenario: Restore workspace tab preference

- **WHEN** the application starts
- **THEN** the system restores the saved setting for opening workspaces in tabs
- **THEN** the system uses `true` when the setting is absent

### Requirement: Workspace tab state is persisted in workspace state

The system SHALL keep opened workspace tabs in `workspace.json` rather than in per-workspace document session JSON.

#### Scenario: Save opened workspace tabs to workspace state

- **WHEN** opened workspace tabs change
- **THEN** the system saves the tab list and active workspace to `workspace.json`

#### Scenario: Keep document tabs per workspace

- **WHEN** the active workspace changes
- **THEN** the system saves document tabs through the existing per-workspace session state
- **THEN** the system does not store document tabs in `workspace.json`
