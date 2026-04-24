## ADDED Requirements

### Requirement: Independent Editor Widget
The system SHALL provide an independent editor crate (`katana-editor`) that handles all text editing functions, syntax highlighting, line numbering, and cursor management without relying on KatanA's global application state.

#### Scenario: Editor Initialization
- **WHEN** the editor widget is initialized with a given text buffer
- **THEN** it renders the text, applies appropriate syntax highlighting, and provides standard text editing capabilities independently of the parent UI.

### Requirement: Editor State Decoupling
The editor component MUST be decoupled from file management systems. It SHALL accept raw text and emit events or state changes for the parent UI to handle saving or loading.

#### Scenario: Text Modification Event
- **WHEN** a user modifies text inside the editor
- **THEN** the editor component updates its internal buffer and notifies the parent UI via callback or event queue, without performing direct file I/O.
