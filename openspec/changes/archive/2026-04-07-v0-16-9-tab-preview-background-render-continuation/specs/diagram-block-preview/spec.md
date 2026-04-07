# Delta Spec: Diagram Block Preview — Background Continuity Across Tab Switches

## ADDED Requirements

### Requirement: Diagram rendering continues while the tab is inactive

The system SHALL allow Mermaid, PlantUML, and Draw.io rendering jobs to continue while their owning tab is inactive, provided the source generation remains valid.

#### Scenario: Mermaid render finishes after the user switched tabs

- **WHEN** a Mermaid block is still rendering and the user switches to another tab before completion
- **THEN** the Mermaid render continues in the background for the original tab
- **THEN** returning to the tab shows the finished result or its current valid lifecycle state without restarting from the initial loading state

#### Scenario: PlantUML and Draw.io behave the same way

- **WHEN** a PlantUML or Draw.io block is still rendering and the user switches to another tab before completion
- **THEN** the render continues in the background for the original tab
- **THEN** the completed result remains associated with that tab until it is drawn or invalidated

### Requirement: Completed diagram results hydrate into the next visible activation

The system SHALL hydrate completed diagram render results into the preview when the owning tab becomes visible again.

#### Scenario: Diagram render completed while the tab was hidden

- **WHEN** a diagram result finished in the background while its tab was inactive
- **THEN** the next activation of that tab attaches the completed diagram result into the visible preview
- **THEN** the tab does not start a duplicate render for the same valid diagram source

#### Scenario: Older diagram result completes after source invalidation

- **WHEN** a diagram job completes after the document source has already changed or the preview was explicitly invalidated
- **THEN** the stale result is ignored
- **THEN** only the current generation may be hydrated into the preview
