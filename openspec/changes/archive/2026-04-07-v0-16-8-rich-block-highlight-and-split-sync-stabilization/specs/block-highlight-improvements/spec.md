## ADDED Requirements

### Requirement: Hovering a structured preview block highlights the matching source range

The system MUST highlight the matching source range in the editor when the user hovers a rendered Mermaid, PlantUML, or Draw.io block, or a GitHub Flavored Markdown alert/admonition block, in preview.

#### Scenario: Hover a Mermaid block

- **WHEN** the user places the pointer over a rendered Mermaid block
- **THEN** only the source range corresponding to that Mermaid fence is highlighted in the editor

#### Scenario: Hover a PlantUML block

- **WHEN** the user places the pointer over a rendered PlantUML block
- **THEN** only the source range corresponding to that PlantUML fence is highlighted in the editor

#### Scenario: Hover a Draw.io block

- **WHEN** the user places the pointer over a rendered Draw.io block
- **THEN** only the source range corresponding to that Draw.io fence is highlighted in the editor

#### Scenario: Hover a GitHub Flavored Markdown alert block

- **WHEN** the user places the pointer over a rendered preview block produced from alert/admonition syntax such as `[!NOTE]`
- **THEN** only the source range corresponding to that alert/admonition block is highlighted in the editor

### Requirement: Hover-highlight ranges stay aligned around structured block boundaries

The system MUST keep hover-highlight source ranges aligned even when paragraphs or headings exist immediately before or after a rendered diagram or alert/admonition block.

#### Scenario: Paragraph immediately precedes a structured block

- **WHEN** the user hovers the diagram or alert/admonition block
- **THEN** the preceding paragraph lines are not highlighted by mistake

#### Scenario: Paragraph immediately follows a structured block

- **WHEN** the user hovers the diagram or alert/admonition block
- **THEN** the following paragraph lines are not highlighted by mistake
