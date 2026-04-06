# Delta Spec: Markdown Authoring — Alert Rhythm Polish

## ADDED Requirements

### Requirement: GitHub Flavored Markdown alert blocks use compact vertical rhythm

The system SHALL render GitHub Flavored Markdown alert/admonition blocks with asymmetric vertical padding on the title row and restrained vertical margins for the whole block.

#### Scenario: Alert title row uses asymmetric padding

- **WHEN** the user previews an alert block such as `[!NOTE]`
- **THEN** the title row is not cramped against the body and uses slightly more bottom space than top space
- **THEN** the icon and title text read as a single header row

#### Scenario: Alert block margins stay restrained

- **WHEN** an alert block is surrounded by paragraphs or lists
- **THEN** the alert block does not create excessive empty space above or below itself
- **THEN** surrounding blocks retain readable separation without being visually crushed
