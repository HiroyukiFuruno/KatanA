## ADDED Requirements

### Requirement: Workspace open flow supports workspace tabs

The system SHALL route workspace open requests through a single workspace open policy before loading the selected workspace.

#### Scenario: Open workspace through menu

- **WHEN** the user opens a workspace from the application menu
- **THEN** the system applies the workspace tab open policy
- **THEN** the system loads the selected workspace as the active workspace

#### Scenario: Open workspace through workspace list or history

- **WHEN** the user opens a workspace from the saved workspace list or workspace history
- **THEN** the system applies the same workspace tab open policy as the application menu
- **THEN** the system loads the selected workspace as the active workspace

#### Scenario: Open workspace through workspace tab plus button

- **WHEN** the user opens a workspace from the workspace tab `+` button
- **THEN** the system applies the same workspace tab open policy as the application menu
- **THEN** the system loads the selected workspace as the active workspace
