## ADDED Requirements

### Requirement: Split-mode scroll sync includes geometry contributed by structured preview blocks

The system SHALL incorporate preview geometry contributed by rendered Mermaid, PlantUML, and Draw.io blocks, and by GitHub Flavored Markdown alert/admonition blocks, into editor-side source mapping whenever split-mode scroll sync is enabled, without introducing drift before or after the block boundary.

#### Scenario: Scroll from the editor across a diagram fence

- **WHEN** the user scrolls on the editor side from immediately before a diagram fence to immediately after it
- **THEN** the preview follows continuously to the rendered diagram position
- **THEN** no discontinuous jump occurs around the diagram boundary

#### Scenario: Scroll from the preview across a rendered diagram block

- **WHEN** the user scrolls on the preview side across a rendered diagram block
- **THEN** the editor follows the matching source lines before and after the diagram fence
- **THEN** line mapping does not break because of rendered-height differences

#### Scenario: Scroll from the preview across an alert/admonition block

- **WHEN** the user scrolls on the preview side across a GitHub Flavored Markdown alert/admonition block
- **THEN** the editor follows the matching source lines before and after that block
- **THEN** line mapping does not drift because of alert block height and internal title/body layout

### Requirement: Split sync re-converges after asynchronous diagram render completion

The system SHALL re-evaluate split sync after diagram render completion changes preview geometry and converge back to a stable state.

#### Scenario: Mermaid render completes after the initial preview is already visible

- **WHEN** the preview appears first and the Mermaid image finishes rendering later, updating preview geometry
- **THEN** split sync converges without oscillating in the opposite direction
- **THEN** the logical position around the diagram block is preserved
