## ADDED Requirements

### Requirement: Help menu exposes a Demo command

The system MUST expose a Demo command from the Help menu so users can launch the bundled feature walkthrough from inside the application.

#### Scenario: Demo command appears in Help

- **WHEN** the native application menu is built for Katana
- **THEN** the Help menu contains a Demo entry
- **THEN** the Demo entry is localized through the existing menu i18n path

#### Scenario: Demo command dispatches the help-demo workflow

- **WHEN** the user selects the Help menu's Demo entry
- **THEN** the application dispatches the help-demo opening action
- **THEN** the demo-bundle resolver is invoked without requiring the user to browse the filesystem manually
