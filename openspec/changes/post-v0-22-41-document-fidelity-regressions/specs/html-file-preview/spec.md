## ADDED Requirements

### Requirement: Real document sticky navigation matches browser scrolling

The HTML viewer MUST honor `position: sticky` with viewport inset and containing-block bounds. For the supplied requirements document, the left table of contents MUST remain visible at the viewport top while the main content scrolls to `#s15`.

#### Scenario: Navigate the supplied document to section 15

- **WHEN** `要件定義書_v1.0.html#s15` is rendered and the page scroll position reaches section 15
- **THEN** the left table of contents remains at the same viewport-top position as Chrome
- **AND THEN** the main content moves independently beneath the current scroll position

#### Scenario: Sticky element reaches its containing block end

- **WHEN** a sticky element would move beyond the bottom of its containing block
- **THEN** KRR clamps the element inside that containing block instead of treating it as permanently fixed

### Requirement: HTML-inapplicable preview menus remain inactive

KatanA MUST disable table-of-contents, export/download, story/slideshow, and tools/view preview panels that have no meaningful HTML operation. Disabled panels MUST reject click, hover-open, shortcut, and direct action dispatch.

#### Scenario: HTML tab is active

- **WHEN** the active document is a local or remote HTML document
- **THEN** every unsupported preview panel button is visibly disabled
- **AND THEN** no unsupported panel state can be opened through another input path

### Requirement: Supplied HTML compatibility is differential-tested

The complete KatanA to KDV to KRR path MUST compare the supplied HTML at the same logical viewport against Chrome for CSS layout, JavaScript-visible state, scrolling, and anchor navigation. A static initial screenshot alone MUST NOT satisfy this requirement.

Chrome rendering and observable behavior of the same original document MUST be the reference, not the current KatanA output. The comparison MUST record and align viewport, display scale, fonts, and external-resource/network conditions. It MUST cover typography, colors, image placement, and layout as well as applicable links, scrolling, forms, and JavaScript-driven interactions. Existing independent quality thresholds MUST remain unchanged.

#### Scenario: A native report identifies a browser compatibility gap

- **WHEN** the original HTML differs in style or behavior between Chrome and KatanA
- **THEN** the expected result is captured from Chrome under the recorded comparison conditions
- **AND THEN** the mismatch remains an unresolved acceptance item until its owner-layer fix is independently reverified through KatanA
- **AND THEN** the reference MUST NOT be regenerated from KatanA to hide the mismatch

#### Scenario: Browser differential reaches the same anchor and state

- **WHEN** Chrome and KatanA open the supplied HTML with the same viewport and navigate to `#s15`
- **THEN** the harness compares the sticky table of contents, main-column geometry, active TOC class, and visible section
- **AND THEN** any unsupported CSS or Web API is reported as a typed owner-layer gap
