## MODIFIED Requirements

### Requirement: Mermaid blocks render inline in the standard preview

The system SHALL render fenced Markdown blocks labeled `mermaid` as inline diagrams in the default preview experience, using a Rust-native backend when the selected backend has passed the diagram renderer parity gate.

#### Scenario: Render a Mermaid flowchart

- **WHEN** the active Markdown document contains a valid fenced `mermaid` block
- **THEN** the preview shows the rendered diagram instead of the raw fenced source
- **THEN** changes to the block are reflected when the preview refreshes

#### Scenario: Render Mermaid without Node.js

- **WHEN** the selected Rust-native Mermaid backend is enabled and a supported Mermaid block is rendered
- **THEN** the preview does not require Node.js or Mermaid CLI to be installed
- **THEN** the rendered result uses the same diagram result and fallback contract as the previous Mermaid path

### Requirement: PlantUML blocks render inline in the standard preview

The system SHALL render fenced Markdown blocks labeled `plantuml` as inline diagrams in the default preview experience, using a Rust-native backend when the selected backend has passed the diagram renderer parity gate.

#### Scenario: Render a PlantUML sequence diagram

- **WHEN** the active Markdown document contains a valid fenced `plantuml` block
- **THEN** the preview shows the rendered diagram instead of the raw fenced source
- **THEN** the rendered result is produced through a fully local bundled rendering path compatible with the desktop application

#### Scenario: Render PlantUML without Java

- **WHEN** the selected Rust-native PlantUML backend is enabled and a supported PlantUML block is rendered
- **THEN** the preview does not require Java or `plantuml.jar` to be installed
- **THEN** the rendered result uses the same diagram result and fallback contract as the previous PlantUML path
