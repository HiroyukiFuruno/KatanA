# UI Architecture and Testability

## MODIFIED Requirements

### Requirement: Separation of UI and Logic

UI components (`*_ui.rs`) and pure logic (`*_logic.rs` or state structures) MUST be physically separated to ensure testability.

#### Scenario: Verify component file structure

- **Given** I am developing a UI component (e.g., in `views/panels/workspace/`)
- **When** the component is implemented
- **Then** its egui rendering logic is decoupled into a separate file from its state mutation and calculation logic.

#### Scenario: Verify 100% test coverage for pure logic

- **Given** a pure logic module that has been separated from UI rendering
- **When** tests are executed via `make check`
- **Then** the module is no longer marked with `COVERAGE_IGNORE` and has 100% branch test coverage.
