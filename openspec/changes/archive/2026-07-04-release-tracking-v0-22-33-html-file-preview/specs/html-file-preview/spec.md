> **Archived delta:** This file is retained for traceability and synchronized
> with the canonical Rust/V8 requirements in
> `openspec/specs/html-file-preview/spec.md`. The rejected Chromium route is
> not a release requirement.

## ADDED Requirements

### Requirement: HTML documents can be opened as browser tabs

The system SHALL allow `.html` and `.htm` files to be opened from the workspace tree, file open dialog, and drag-and-drop flow. The system SHALL also allow a user-entered `http` or `https` URL to be opened as an HTML browser tab.

#### Scenario: Open a workspace HTML file

- **WHEN** the user opens a workspace file named `sample.html`
- **THEN** KatanA opens it as the active document
- **THEN** KatanA reads the main document and starts the HTML browser-session path

#### Scenario: Open an HTML URL

- **WHEN** the user enters an allowed `http` or `https` URL
- **THEN** KatanA fetches the main document
- **THEN** KatanA starts the HTML browser-session path with the final complete document URL

### Requirement: KatanA must preserve the browser document source contract

KatanA MUST provide the unmodified raw HTML and complete `file`, `http`, or `https` document URL to KDV. KDV MUST forward those values to KRR without parsing, normalizing, or fetching the main document. KatanA and KDV MUST NOT inject a doctype, `<base>`, navigation script, or rewritten HTML wrapper.

#### Scenario: Local relative resources use the original file origin

- **WHEN** a workspace HTML document references a relative stylesheet, script, or image
- **THEN** KatanA passes the raw HTML and complete file document URL to KDV
- **THEN** KRR resolves allowed subresources against that document URL

#### Scenario: Redirected URL preserves its final origin

- **WHEN** an allowed URL fetch redirects to another allowed URL
- **THEN** KatanA supplies the response body and final complete document URL
- **THEN** KRR evaluates the document and relative resources against the final origin

### Requirement: Browser-equivalent HTML session is the only interactive preview path

The system MUST use a persistent KRR browser page as the source of truth for HTML5 parsing, CSS cascade/layout/paint, JavaScript, Web APIs, event loop behavior, form controls, hit-testing, and page lifecycle. The system MUST NOT fall back to static HTML rendering when the browser session cannot start. KatanA MUST display a typed viewer error instead of using a static parser, text normalizer, Markdown renderer, or HTML export image.

#### Scenario: JavaScript action updates the document

- **WHEN** a script, timer, microtask, button, or form action changes DOM or style state
- **THEN** KRR evaluates the action and repaints the page
- **THEN** KatanA displays the latest KRR frame without interpreting HTML, CSS, or JavaScript

#### Scenario: Browser runtime is unavailable

- **WHEN** the KRR in-process runtime cannot start
- **THEN** the HTML tab displays a typed browser-runtime error
- **THEN** no static HTML renderer or export surface is shown as an interactive preview

### Requirement: KatanA must host a complete interactive browser surface

KatanA MUST display the latest complete viewport frame from the KDV adapter and forward pointer, keyboard, text/IME, focus, scroll, and resize events in order to the KRR browser session. KatanA MUST NOT perform browser hit-testing or compose partial damage images.

#### Scenario: Accordion, button, and text input remain interactive

- **WHEN** the user expands an accordion, activates a JavaScript button, and edits a form field
- **THEN** KatanA forwards raw input events to KDV
- **THEN** KRR performs hit-testing and event dispatch
- **THEN** each returned frame contains both unchanged viewport content and the updated control state

#### Scenario: Resize changes browser layout

- **WHEN** the native preview viewport changes size
- **THEN** KatanA forwards the new viewport to KDV
- **THEN** KRR reflows and repaints the page at that exact viewport size

### Requirement: Browser navigation must use KatanA main-document acquisition

KRR MUST report runtime-confirmed top-level navigation events after it evaluates event listeners, `preventDefault()`, same-document behavior, and default actions. KDV MUST forward navigation events without interpreting links. KatanA MUST apply its history and resource policy, fetch the next main document when allowed, and reload the browser session with the new raw HTML and complete document URL.

#### Scenario: Link navigation changes the active document

- **WHEN** a link action produces an allowed top-level document navigation
- **THEN** KRR reports the normalized navigation event through KDV
- **THEN** KatanA fetches the target main document and updates tab history
- **THEN** the browser surface displays the target document

#### Scenario: Page script cancels navigation

- **WHEN** a page event handler calls `preventDefault()`
- **THEN** KRR does not report a host navigation event
- **THEN** KatanA does not infer navigation from link geometry or markup

### Requirement: HTML rendering ownership must remain outside KatanA and KDV

KatanA and KDV MUST NOT implement an HTML parser, CSS cascade/layout, JavaScript interpreter, browser hit-test, or platform WebView for the interactive viewer. KDV SHALL own only the worker-backed adapter for KRR session lifecycle, latest-frame delivery, input forwarding, navigation events, and typed errors. KRR/KDV public APIs MUST NOT expose KatanA UI framework types.

#### Scenario: Browser-session adapter remains reusable

- **WHEN** KatanA opens an HTML browser tab
- **THEN** KDV creates and owns the KRR session on its worker boundary
- **THEN** KatanA consumes only adapter frames, events, state, and typed errors
- **THEN** KDV does not expose DOM nodes, CSS properties, or clickable regions

### Requirement: HTML sessions refresh only on meaningful source changes

KatanA MUST reload the active browser session when its source document or an allowed renderer-reported local dependency changes. KatanA SHALL coalesce rapid save and watcher events and MUST NOT parse HTML or CSS to discover dependencies.

#### Scenario: Source save reloads the browser session

- **WHEN** a changed HTML document is saved
- **THEN** KatanA reads the latest source once after the save succeeds
- **THEN** KatanA requests one coalesced browser-session reload

#### Scenario: Disallowed dependency is requested

- **WHEN** the document requests a workspace escape, unsupported scheme, disallowed origin, subprocess, or remote iframe
- **THEN** KRR rejects the request through its browser resource policy
- **THEN** KatanA and KDV do not grant the requested host capability

### Requirement: Markdown-specific tools must not process HTML documents

The system MUST keep Markdown diagnostics, Markdown formatting, Markdown export, and Markdown diagram wrapping scoped to Markdown documents.

#### Scenario: Markdown tools skip an HTML tab

- **WHEN** the active document is an HTML file or URL
- **THEN** Markdown diagnostics and formatting do not process its source
- **THEN** Markdown export and diagram wrapping are not used to render its interactive surface

### Requirement: v0.22.33 release must prove the published browser chain

KatanA v0.22.33 MUST consume published KDV `0.3.x` and KRR `0.4.x` crates from crates.io, with a minimum resolved version of KDV `0.3.1` and KRR `0.4.3`. KatanA MUST NOT use a local path or git dependency for KDV/KRR, or package a Chromium/browser runtime archive. The initial release order remains KRR `0.4.0` before KDV `0.3.0`, while the final KatanA integration MUST include the required patch fixes. The withdrawn `v0.29.0` MUST NOT be accepted as a release target.

#### Scenario: Release evidence exercises browser behavior

- **WHEN** v0.22.33 release readiness is evaluated
- **THEN** packaged headless-process evidence covers external CSS/JavaScript, accordion, JavaScript action, text input, link navigation, reload, resize, and complete action frames
- **THEN** same-document and external fragment states prove the complete document origin, raw KRR frame pixels, and composed KatanA screenshot pixels independently
- **THEN** the evidence shows no overlapping or clipped text in unchanged or updated regions
- **THEN** release remains blocked until the user explicitly approves that evidence

#### Scenario: Static or unpublished chain is detected

- **WHEN** KatanA resolves KDV below `0.3.1`, KRR below `0.4.3`, a path/git source, or includes a Chromium/browser runtime archive
- **THEN** the v0.22.33 release gate fails
- **THEN** a static screenshot or parser test cannot satisfy the release gate
