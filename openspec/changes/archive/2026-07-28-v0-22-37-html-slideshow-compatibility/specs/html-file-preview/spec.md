## ADDED Requirements

### Requirement: Real HTML slide decks preserve layout and interaction

The browser-equivalent viewer MUST render a self-contained slide deck through the KRR Rust/V8 frame path. It MUST preserve viewport-relative dimensions, CSS math, gradients, positioned and layered controls, flex/grid composition, nested inline formatting, explicit line breaks, multilingual wrapping, and class-driven visibility. Pointer and keyboard input MUST execute the page's JavaScript and repaint the selected slide without KatanA or KDV interpreting the document.

#### Scenario: Pointer navigation advances the deck

- **WHEN** the user activates a rendered next-slide control
- **THEN** KRR dispatches the pointer event to the document's JavaScript handler
- **THEN** the next complete frame displays the newly active slide and updated progress state

#### Scenario: Keyboard navigation advances the deck

- **WHEN** the active HTML surface receives a supported next-slide key
- **THEN** the document's keyboard handler runs without a JavaScript exception
- **THEN** the next complete frame displays the corresponding slide

### Requirement: Slideshow evidence waits for the resulting HTML frame

The headless acceptance harness MUST observe the active HTML frame generation before pointer or keyboard input and MUST wait for a newer generation before capturing or asserting the resulting state. A fixed sleep alone MUST NOT satisfy the slideshow transition contract.

#### Scenario: Input does not produce a new frame

- **WHEN** a slideshow action does not produce a newer HTML frame within its bounded wait
- **THEN** the acceptance run fails with the previous generation and timeout
- **THEN** no stale screenshot is accepted as the action result

### Requirement: v0.22.37 proves the published slideshow chain

KatanA `v0.22.37` MUST resolve published KDV `0.3.5` and KRR `0.4.14` crates from crates.io in the application and headless harness, MUST reject path/git overrides, and MUST keep browser executables and archives out of the product. The SemVer guard MUST allow only published `v0.22.36` to `v0.22.37` and MUST reject withdrawn `v0.29.0`.

#### Scenario: Real slide-deck release evidence is evaluated

- **WHEN** `v0.22.37` release readiness is evaluated
- **THEN** the actual Google Drive `slides.html` completes 14 slide states and 43 scripted actions through the native KatanA UI
- **THEN** click and keyboard navigation each produce newer complete frames without a worker stop or JavaScript exception
- **THEN** fresh screenshots are available for user review before any KatanA commit, push, PR, or release
