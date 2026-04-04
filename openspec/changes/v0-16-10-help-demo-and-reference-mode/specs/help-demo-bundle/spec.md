## ADDED Requirements

### Requirement: Help demo opens the localized feature bundle inside the existing tab surface

The system MUST resolve demo assets from `assets/feature` under the active workspace and open them in the existing tab strip instead of a separate window or modal.

#### Scenario: Open demo bundle from Help
- **WHEN** the user executes the Help menu's Demo command while the active workspace contains `assets/feature`
- **THEN** the resolved demo files are opened as tabs in the current workspace session
- **THEN** existing non-demo tabs remain open

#### Scenario: Missing demo bundle fails recoverably
- **WHEN** the user executes the Help menu's Demo command but no active workspace is open, or `assets/feature` is missing
- **THEN** the system shows a recoverable error/status message
- **THEN** no existing tabs or groups are removed or replaced

### Requirement: Help demo groups the resolved files under `demo`

The system MUST cluster the resolved demo files into a tab group named `demo`.

#### Scenario: First demo launch creates the group
- **WHEN** the resolved demo bundle is opened for the first time in the current workspace session
- **THEN** the system creates a tab group whose visible name is `demo`
- **THEN** every resolved demo tab belongs to that group

#### Scenario: Reopening demo reuses the existing group
- **WHEN** the user launches the Help demo again while a `demo` tab group already exists
- **THEN** the system reuses that group instead of creating a duplicate demo group
- **THEN** the group's membership is reconciled with the newly resolved demo file set

### Requirement: Help demo opens Markdown docs and reference code with different presentation policies

The system MUST distinguish Markdown demo documents from non-Markdown textual code/reference assets during demo expansion.

#### Scenario: Markdown demo document opens as a normal document tab
- **WHEN** a resolved demo file is a Markdown document selected by the locale resolver
- **THEN** the file opens through the normal document pipeline
- **THEN** it is not forced into reference mode solely because it belongs to the demo bundle

#### Scenario: Textual code asset opens in reference mode
- **WHEN** a resolved demo file is a non-Markdown textual asset under `assets/feature`
- **THEN** the file opens in reference mode
- **THEN** the system uses the existing code-pane surface instead of treating it as an editable document
