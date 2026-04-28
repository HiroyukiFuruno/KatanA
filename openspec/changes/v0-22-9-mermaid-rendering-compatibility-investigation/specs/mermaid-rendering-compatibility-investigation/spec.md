## ADDED Requirements

### Requirement: Mermaid rendering compatibility is investigated before broad fixes

システムは、Mermaid.js 描画の追加修正を行う前に、旧 `mmdc` 描画との差分を図形種類ごとに調査しなければならない（MUST）。

#### Scenario: Identify diagram-specific rendering differences

- **WHEN** a Mermaid diagram type is selected for compatibility review
- **THEN** the investigation records the Mermaid source, `mmdc` output characteristics, Mermaid.js output characteristics, and visible differences
- **THEN** the investigation does not treat a single diagram type fix as proof that all diagram types are compatible

#### Scenario: Keep gantt current-date behavior as one known pattern

- **WHEN** a Mermaid gantt chart has a current-date marker outside the chart date range
- **THEN** the investigation records that the marker can affect output size or perceived alignment
- **THEN** the investigation treats that pattern as one fixture, not as the only compatibility issue

### Requirement: Compatibility fixtures cover common Mermaid diagram types

システムは、Mermaid.js と `mmdc` の描画差分を確認するため、代表的な Mermaid 図形 fixture を保持しなければならない（SHALL）。

#### Scenario: Cover common diagram families

- **WHEN** compatibility fixtures are prepared
- **THEN** they include at least flowchart, sequence, class, state, entity relationship, gantt, pie, journey, mindmap, and timeline examples
- **THEN** each fixture includes enough labels, edges, and theme-sensitive elements to reveal visible regressions

#### Scenario: Capture output evidence for user review

- **WHEN** a compatibility investigation result is reported
- **THEN** it includes screenshot or image evidence generated from reproducible commands or `scripts/screenshot`
- **THEN** the evidence distinguishes `mmdc` baseline output from Mermaid.js output

### Requirement: Fix decisions are prioritized from evidence

システムは、Mermaid.js 描画差分の修正優先度を、再現証跡とユーザー影響に基づいて決めなければならない（MUST）。

#### Scenario: Classify compatibility findings

- **WHEN** a rendering difference is found
- **THEN** it is classified as layout, size, theme, typography, marker, interaction, error handling, or cache behavior
- **THEN** the finding records whether it should be fixed immediately, deferred to a versioned change, or accepted as a documented Mermaid.js difference

#### Scenario: Avoid uncontrolled SVG post-processing

- **WHEN** a proposed fix requires SVG post-processing
- **THEN** the design records why Mermaid initialization settings or container sizing cannot solve it
- **THEN** the post-processing scope is limited to the affected diagram type or SVG element pattern
