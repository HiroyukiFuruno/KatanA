## ADDED Requirements

### Requirement: High-granularity piecewise-linear scroll synchronization

The system SHALL synchronize the scroll position of the editor and preview panes using a piecewise-linear mapping derived from structural anchors (headings, paragraphs, and blocks) found in both views.

#### Scenario: Sync position within a long paragraph

- **WHEN** the user scrolls the editor to the middle of a long paragraph
- **THEN** the system calculates the progress within that paragraph's segment in the ScrollMapper
- **THEN** the preview pane scrolls to the corresponding relative position within the rendered paragraph

### Requirement: End-of-file (EOF) boundary synchronization

The system SHALL ensure that both panes reach their respective bottom positions simultaneously, regardless of the difference in rendered height between the editor and preview.

#### Scenario: Reaching the end of a document

- **WHEN** either the editor or the preview is scrolled to the absolute bottom of its content
- **THEN** the other pane SHALL also be scrolled to its absolute bottom
- **THEN** any remaining space is compensated by dynamic virtual padding at the bottom of the viewport

### Requirement: Multi-level anchor discovery for mapping

The system SHALL discover synchronization anchors at multiple levels (H1-H6, paragraph, list, blockquote) to build a mapping table that minimizes visual drift during scrolling.

#### Scenario: Mapping with sparse headings

- **WHEN** a document has long sections between headings
- **THEN** the system identifies paragraph-level elements as secondary anchors
- **THEN** the scroll mapping remains stable and precise even in areas without explicit heading markers
