# Delta Spec: Diagram Block Preview — Stable Source Anchors

## ADDED Requirements

### Requirement: Rendered diagram blocks preserve a stable source anchor for preview interaction

The system SHALL preserve a stable source anchor, including source-line span and block identity, for rendered Mermaid, PlantUML, and Draw.io preview blocks even after pending-to-rendered replacement.

#### Scenario: Mermaid block is replaced after asynchronous render completion

- **WHEN** a Mermaid block transitions from pending state to its rendered image/state
- **THEN** the preview interaction layer continues to reference the same source anchor
- **THEN** the source range does not drift into the preceding or following block

#### Scenario: PlantUML and Draw.io blocks use the same anchor contract

- **WHEN** a PlantUML or Draw.io block is shown as rendered preview output
- **THEN** preview hover highlight and split sync can use the stable source anchor rather than a renderer-specific rect heuristic
- **THEN** the source-anchoring contract does not vary by renderer kind
