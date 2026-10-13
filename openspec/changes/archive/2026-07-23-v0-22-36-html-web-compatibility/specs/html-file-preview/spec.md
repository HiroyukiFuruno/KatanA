## ADDED Requirements

### Requirement: KRR host events must follow DOM Event dispatch semantics

KRR MUST use its installed V8 `Event` implementation for pointer, keyboard, form, lifecycle, and synthetic events. Dispatch MUST preserve `type`, `target`, `currentTarget`, `eventPhase`, `bubbles`, `cancelable`, and `defaultPrevented`; MUST run capture, target, and bubble listeners in tree order; and MUST honor `stopPropagation()`, `stopImmediatePropagation()`, `preventDefault()`, listener removal, `capture`, and `once` before default actions.

#### Scenario: Child action stops parent click handling

- **WHEN** a child button listener calls `event.stopPropagation()` during a bubbling click
- **THEN** the child listener and its target-phase handlers complete without a JavaScript exception
- **THEN** ancestor bubble listeners do not run
- **THEN** capture listeners that ran before the target retain their recorded order

#### Scenario: Cancelled link does not navigate

- **WHEN** a cancelable link click listener calls `event.preventDefault()`
- **THEN** `event.defaultPrevented` becomes true
- **THEN** KRR emits no top-level navigation intent

### Requirement: Interactive stylesheets must use structured CSS parsing and cascade

KRR MUST parse interactive stylesheets and style attributes with a structured CSS tokenizer/parser rather than splitting on braces, semicolons, or colons. The cascade MUST preserve source order, selector specificity, inline priority, `!important`, inheritance, and custom properties. The interactive computed-style/layout path MUST support the declared compatibility matrix for normal document flow, flex, grid, box sizing, overflow, responsive media rules, typography, color/background/border, tables, and viewport-relative or font-relative lengths.

#### Scenario: Realistic external stylesheet controls layout

- **WHEN** an allowed URL document loads an external stylesheet containing comments, quoted delimiters, custom properties, descendant/child/attribute selectors, `!important`, flex, grid, box sizing, overflow, and a matching media query
- **THEN** KRR applies the winning declarations without discarding adjacent valid rules
- **THEN** the complete frame satisfies the expected layout and color assertions at the current viewport

#### Scenario: JavaScript recascades mutated state

- **WHEN** JavaScript changes an element class, attribute, inline style, or custom property used by matching declarations
- **THEN** KRR recomputes style and layout from the current DOM
- **THEN** the next complete frame reflects the mutated state

### Requirement: URL HTML must be proven through principal fetch, subresources, and navigation

KatanA MUST accept an allowed `http` or `https` URL, fetch the principal document, retain the final response URL after redirects, and pass unmodified HTML plus that URL through KDV to KRR. KRR MUST resolve and request policy-approved relative stylesheet, script, and image resources against the final URL. Runtime-confirmed same-origin link navigation MUST return to KatanA for principal-document acquisition and replace the active page without losing tab history.

#### Scenario: Redirected page loads relative resources

- **WHEN** a user-entered URL redirects to `/app/index.html` and that document references `assets/page.css`, `assets/page.js`, and `assets/mark.svg`
- **THEN** KatanA passes the final `/app/index.html` URL as the document origin
- **THEN** KRR requests each resource below `/app/assets/`
- **THEN** the complete frame proves the stylesheet, script mutation, and image were applied

#### Scenario: Page link loads another URL document

- **WHEN** the user activates a same-origin relative link and no handler cancels the action
- **THEN** KRR emits the normalized target URL after event dispatch
- **THEN** KatanA fetches the target as the next principal document
- **THEN** the active tab displays the linked page and records both document URLs in history

### Requirement: HTML page frame must fill the native preview viewport

KatanA MUST allocate the exact HTML preview content bounds to the complete KRR frame. KatanA MUST NOT add page padding, a decorative border, a rounded frame, or a host background between the page frame and those bounds. The page's own `html` and `body` styles remain KRR-owned.

#### Scenario: Page background reaches every viewport edge

- **WHEN** an HTML document sets a visible body background and KatanA displays its browser surface
- **THEN** the KRR frame bounds equal the available HTML preview content bounds
- **THEN** no KatanA preview padding, border, or background is visible around the page

### Requirement: v0.22.36 release must prove realistic web compatibility

KatanA v0.22.36 MUST consume the required KDV `0.3.x` and KRR `0.4.x` patch releases from crates.io for final release gates, MUST NOT land path/git dependencies, and MUST NOT include Chromium, WebView, external browser helpers, or browser archives. The SemVer guard MUST allow only the adjacent update from published v0.22.35 to v0.22.36 and MUST continue to reject withdrawn v0.29.0.

#### Scenario: Headless release evidence exercises a realistic URL page

- **WHEN** v0.22.36 release readiness is evaluated
- **THEN** a loopback HTTP fixture proves redirect final origin, relative CSS/JS/image requests, structured CSS cascade/layout, event propagation and cancellation, accordion, JavaScript mutation, text input, fragment navigation, linked-document navigation, resize, scroll, and exact viewport bounds
- **THEN** machine assertions and before/after screenshots independently prove the DOM, resource, navigation, layout, and visual results
- **THEN** strict coverage remains 100% with zero uncovered lines and no exclusion or threshold relaxation

#### Scenario: Narrow diagnostic fixture is presented as release evidence

- **WHEN** evidence omits realistic external resources, event propagation, URL navigation, or exact viewport assertions
- **THEN** the v0.22.36 release gate fails even if a static screenshot exists
