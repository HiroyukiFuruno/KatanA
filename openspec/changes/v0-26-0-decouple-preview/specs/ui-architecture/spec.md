## MODIFIED Requirements

### Requirement: Delegated Component Architecture

The KatanA UI application (`katana-ui`) MUST NOT contain deep implementation logic for text editing or markdown rendering. It SHALL delegate these responsibilities to the dedicated `katana-editor` and `katana-markdown-preview` crates.

#### Scenario: UI Assembly

- **WHEN** the main application layout is constructed
- **THEN** `katana-ui` acts solely as the glue code, instantiating the editor and preview components and wiring them together using standard event channels or callbacks.
