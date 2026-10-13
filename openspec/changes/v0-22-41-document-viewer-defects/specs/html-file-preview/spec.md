## ADDED Requirements

### Requirement: HTML preview input remains attached to the browser surface
KatanA MUST route pointer, keyboard, wheel, and focus input to the live KRR browser surface so JavaScript actions and both-axis scrolling remain usable after the document is drawn.

#### Scenario: Interactive supplied HTML is opened
- **WHEN** the supplied HTML fixture is opened through the real KatanA host path
- **THEN** clicking its controls MUST change the document and wheel input MUST scroll the browser document
