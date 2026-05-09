## Purpose

Workspace tabs allow multiple workspace roots to stay open in one window while keeping each workspace's document tabs and search context separate.

## Requirements

### Requirement: Workspace tabs are persisted and restored

The system SHALL persist opened workspace tabs and the active workspace in `workspace.json`.

#### Scenario: Persist opened workspace tabs

- **WHEN** the user has multiple workspace tabs open
- **THEN** the system saves their root paths to `workspace.json`
- **THEN** the system saves the active workspace root path to `workspace.json`

#### Scenario: Restore opened workspace tabs on launch

- **WHEN** the application starts and `workspace.json` contains opened workspace tabs
- **THEN** the system restores the workspace tab list
- **THEN** the system opens the saved active workspace when it still exists

#### Scenario: Ignore missing workspace tabs during restore

- **WHEN** `workspace.json` contains a workspace tab whose path no longer exists
- **THEN** the system excludes that workspace tab from the restored tab list
- **THEN** the system selects an existing workspace tab if the saved active workspace is missing

### Requirement: Workspace tabs can be selected and closed

The system SHALL allow users to switch and close workspace tabs without mixing them into file tabs.

#### Scenario: Select a workspace tab

- **WHEN** the user clicks a workspace tab
- **THEN** the system saves the current workspace session
- **THEN** the system loads the selected workspace as the active workspace
- **THEN** the system restores that workspace's file tabs through the existing workspace session restore behavior

#### Scenario: Close an inactive workspace tab

- **WHEN** the user clicks the close button on an inactive workspace tab
- **THEN** the system removes that workspace tab from the opened workspace tabs
- **THEN** the active workspace remains unchanged

#### Scenario: Close the active workspace tab

- **WHEN** the user clicks the close button on the active workspace tab
- **THEN** the system removes that workspace tab from the opened workspace tabs
- **THEN** the system activates a remaining workspace tab when one exists
- **THEN** the system closes the workspace when no workspace tab remains

### Requirement: Workspace tab strip provides workspace opening controls

The system SHALL render workspace tabs in a horizontally scrollable workspace tab strip.

#### Scenario: Show workspace tab close button

- **WHEN** a workspace tab is rendered
- **THEN** the system shows the workspace name
- **THEN** the system shows a close button to the right of the workspace name

#### Scenario: Show workspace tab boundaries

- **WHEN** workspace tabs are rendered
- **THEN** the system draws a subtle border for each workspace tab
- **THEN** the border color remains visible even when the theme's default widget border is transparent
- **THEN** the border uses the accent color while the workspace tab is hovered
- **THEN** the border is drawn inside the tab bounds so tab width and layout do not expand
- **THEN** the border uses a 4px corner radius
- **THEN** the system does not add an active background fill for the border treatment

#### Scenario: Show workspace tab close button on hover

- **WHEN** a workspace tab is not hovered
- **THEN** the system keeps the close button area reserved
- **THEN** the system does not show the close icon
- **WHEN** the workspace tab is hovered
- **THEN** the system shows the close icon in the reserved close button area

#### Scenario: Show document tab boundaries

- **WHEN** document tabs are rendered
- **THEN** the system draws a subtle border for each document tab
- **THEN** the border uses the accent color while the document tab is hovered
- **THEN** the hover region covers the document tab body, not only the close button
- **THEN** the border is drawn on the document tab parent bounds
- **THEN** the border is drawn inside the tab bounds so tab width and layout do not expand
- **THEN** the system does not add an active background fill for the border treatment

#### Scenario: Open workspace from plus button

- **WHEN** the user clicks the `+` button at the right end of the workspace tab strip
- **THEN** the system runs the same workspace open flow as the menu item for opening a workspace

#### Scenario: Scroll workspace tabs horizontally

- **WHEN** workspace tabs exceed the available width
- **THEN** the system allows horizontal scrolling of the workspace tab strip
- **THEN** the system does not show a horizontal scrollbar
- **THEN** the system does not show left or right navigation buttons for workspace tabs

#### Scenario: Scroll to newly opened workspace tab

- **WHEN** a new workspace tab is opened
- **THEN** the system scrolls the workspace tab strip to the newly opened tab

#### Scenario: Use full top-level window width

- **WHEN** workspace tabs are rendered
- **THEN** the system renders the workspace tab strip before the workspace sidebar is allocated
- **THEN** the workspace tab strip uses the full top-level window width

#### Scenario: Distribute workspace tabs evenly

- **WHEN** workspace tabs are rendered within the workspace tab strip
- **THEN** the system reserves the width needed for the `+` button
- **THEN** the system divides the remaining width equally across workspace tabs
- **AND** two workspace tabs each receive half of the remaining width
- **AND** three workspace tabs each receive one third of the remaining width

### Requirement: Temporary workspaces are excluded from workspace tabs

The system SHALL NOT expose temporary file-open workspaces as persisted workspace tabs.

#### Scenario: Open standalone file as temporary workspace

- **WHEN** the user opens a standalone file without a workspace
- **THEN** the system creates a temporary workspace for the current session
- **THEN** the system does not add that temporary workspace to opened workspace tabs
- **THEN** the system does not save that temporary workspace as the active workspace

#### Scenario: Clear legacy temporary workspace tabs

- **WHEN** `workspace.json` contains a temporary workspace path in opened workspace tabs
- **THEN** the system removes that temporary workspace tab during startup cleanup
- **THEN** the system clears active workspace when it points to the temporary workspace

### Requirement: Workspace-scoped search results follow the active workspace

The system SHALL keep global search results scoped to the active workspace.

#### Scenario: Clear global search results on workspace switch

- **WHEN** the active workspace changes
- **THEN** the system clears file-name search results for the previous workspace
- **THEN** the system clears Markdown content search results for the previous workspace
- **THEN** the system keeps the user's search query and Markdown search history

#### Scenario: Re-run global search for current workspace

- **WHEN** the global search panel is open with an existing query
- **AND** the active workspace changes
- **THEN** the next global search render uses the new active workspace as the search root

### Requirement: Workspace open policy is configurable

The system SHALL provide a setting that controls whether opening a different workspace creates a workspace tab.

#### Scenario: Default to opening workspace in tabs

- **WHEN** settings are created with defaults
- **THEN** `open_workspace_in_tabs` is enabled

#### Scenario: Add different workspace as tab when enabled

- **WHEN** `open_workspace_in_tabs` is enabled
- **AND** the user opens a workspace that differs from the active workspace
- **THEN** the system adds the workspace as a workspace tab when it is not already open
- **THEN** the system activates that workspace tab

#### Scenario: Avoid duplicate workspace tabs

- **WHEN** the user opens a workspace that is already present as a workspace tab
- **THEN** the system activates the existing workspace tab
- **THEN** the system does not add a duplicate workspace tab

#### Scenario: Replace current workspace when disabled

- **WHEN** `open_workspace_in_tabs` is disabled
- **AND** only one workspace tab is open
- **AND** the user opens a different workspace
- **THEN** the system replaces the existing workspace tab with the opened workspace
- **THEN** the number of workspace tabs remains one

#### Scenario: Do not grow existing multiple tabs when disabled

- **WHEN** `open_workspace_in_tabs` is disabled
- **AND** multiple workspace tabs are already open
- **AND** the user opens a workspace that is not already present
- **THEN** the system replaces the active workspace tab with the opened workspace
- **THEN** the number of workspace tabs does not increase
