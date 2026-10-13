## ADDED Requirements

### Requirement: Diagram rendering is selected through backend adapters

The system SHALL route Mermaid and PlantUML rendering through KatanA-owned backend adapters rather than calling one concrete CLI, jar, or crate implementation directly from preview code.

#### Scenario: Render Mermaid through a selected backend

- **WHEN** a Mermaid diagram block is rendered
- **THEN** the system selects a Mermaid backend through the diagram backend adapter
- **THEN** preview code receives a renderer-neutral `DiagramResult`
- **THEN** preview code does not directly call `mmdc` or a Rust Mermaid crate

#### Scenario: Render PlantUML through a selected backend

- **WHEN** a PlantUML diagram block is rendered
- **THEN** the system selects a PlantUML backend through the diagram backend adapter
- **THEN** preview code receives a renderer-neutral `DiagramResult`
- **THEN** preview code does not directly call `java`, `plantuml.jar`, or a Rust PlantUML crate

### Requirement: Rust-native backends have parity gates before becoming default

The system MUST verify Rust-native diagram backends against KatanA fixtures and platform packaging gates before making them the default backend.

#### Scenario: Evaluate a Mermaid Rust backend

- **WHEN** a Rust Mermaid backend is added
- **THEN** it is compared against KatanA Mermaid fixtures for supported syntax, rendered output shape, theme propagation, error handling, and export compatibility
- **THEN** it remains opt-in or fallback-only until the parity gate passes

#### Scenario: Evaluate a PlantUML Rust backend

- **WHEN** a Rust PlantUML backend is added
- **THEN** it is compared against KatanA PlantUML fixtures for supported syntax, rendered output shape, theme propagation, error handling, and export compatibility
- **THEN** it remains opt-in or fallback-only until the parity gate passes

### Requirement: External runtime setup is no longer required for default successful rendering

The system SHALL make supported Mermaid and PlantUML diagrams render successfully on a clean desktop installation without requiring the user to install Node.js, Mermaid CLI, Java, or `plantuml.jar`, once the Rust-native backend is selected as default.

#### Scenario: Render on a clean Windows installation

- **WHEN** KatanA starts on Windows without Node.js, Mermaid CLI, Java, or `plantuml.jar`
- **THEN** supported Mermaid and PlantUML diagrams render through the selected Rust-native backends
- **THEN** missing external runtime setup is not shown as the default successful path

#### Scenario: Use an external backend fallback

- **WHEN** a Rust-native backend cannot render a diagram that an enabled external backend can render
- **THEN** the system may fall back to the external backend
- **THEN** the fallback is explicit in diagnostics or debug logging

### Requirement: Backend failures preserve Markdown preview

The system MUST preserve Markdown preview when any diagram backend fails.

#### Scenario: Rust backend fails

- **WHEN** a selected Rust-native backend returns an error for a diagram block
- **THEN** the rest of the Markdown preview remains available
- **THEN** the failing block displays a recoverable diagram failure state with the source and error context

#### Scenario: Fallback backend is unavailable

- **WHEN** no configured backend can render the diagram
- **THEN** the system displays the existing command-not-found or not-installed fallback state
- **THEN** the application does not require a restart to retry after configuration changes
