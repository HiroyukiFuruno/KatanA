## ADDED Requirements

### Requirement: HTML files can be opened as previewable documents

The system SHALL allow `.html` and `.htm` files to be opened from the workspace tree, file open dialog, and drag-and-drop flow as previewable documents.

#### Scenario: Open HTML from workspace tree

- **WHEN** a workspace contains a file named `sample.html`
- **THEN** the workspace tree exposes the file as selectable
- **WHEN** the user selects the file
- **THEN** the system opens it as the active document
- **THEN** the preview pane renders the file through the HTML file preview path

#### Scenario: Open HTML from file dialog

- **WHEN** the user opens the file dialog
- **THEN** `.html` and `.htm` are accepted file extensions
- **WHEN** the user selects an existing `.htm` file
- **THEN** the system opens it as the active document
- **THEN** the preview pane renders the file through the HTML file preview path

#### Scenario: Open HTML by drag and drop

- **WHEN** the user drops an existing `.html` file onto KatanA
- **THEN** the system treats it as an openable document
- **THEN** the preview pane renders the file through the HTML file preview path

### Requirement: HTML files use direct HTML preview instead of Markdown preview

The system MUST route `.html` and `.htm` active documents through a direct HTML preview path rather than treating the file contents as Markdown.

#### Scenario: HTML document renders static HTML elements

- **WHEN** the active document path ends with `.html`
- **AND** the document content includes headings, paragraphs, links, images, details, or tables
- **THEN** the preview pane renders those supported static HTML elements
- **THEN** the system does not require the user to wrap the file contents in Markdown fences or Markdown syntax

#### Scenario: HTML document is not wrapped as Markdown

- **WHEN** the active document path ends with `.htm`
- **THEN** the system does not run the content through Markdown-only diagram fence wrapping
- **THEN** raw HTML document wrappers that are supported by the direct HTML preview path do not appear as plain source text solely because the file is not Markdown

### Requirement: HTML file preview does not introduce a browser runtime

The system MUST NOT introduce WebView, React, DOM runtime, bundled web app, or JavaScript execution to support the MVP HTML file preview.

#### Scenario: Static preview remains native

- **WHEN** HTML file preview support is built
- **THEN** the implementation uses the existing native preview surface, existing HTML renderer, or KDV direct HTML contract
- **THEN** the dependency graph does not gain a WebView, React, DOM runtime, or bundled web app for this feature

### Requirement: Markdown-specific tools do not process HTML files

The system MUST keep Markdown diagnostics, Markdown formatting, and Markdown export scoped to Markdown documents even after HTML files become openable.

#### Scenario: Diagnostics skip HTML files

- **WHEN** an active or open document path ends with `.html`
- **THEN** Markdown diagnostics are not evaluated for that file
- **THEN** the Problems panel does not report Markdown lint diagnostics for that HTML file

#### Scenario: Markdown formatting rejects HTML files

- **WHEN** the user invokes Markdown formatting for a path ending with `.htm`
- **THEN** the system does not pass the file content to the Markdown formatter
- **THEN** the system reports that the path is not a Markdown document

#### Scenario: Markdown export is not used for HTML file preview

- **WHEN** the active document path ends with `.html`
- **THEN** HTML file preview does not call the Markdown export adapter as part of rendering the preview
- **THEN** the file remains a previewed source document, not a Markdown document exported to HTML

### Requirement: External repository work is gated by confirmed need

The system SHALL keep KDV and KRR changes out of scope until Katana-side integration proves that an external public API, source contract, or rendering capability is missing.

#### Scenario: KDV change is created only after API gap is confirmed

- **WHEN** Katana-side implementation verifies that the existing KDV HTML source contract is insufficient
- **THEN** the implementer records the specific missing API or contract
- **THEN** KDV work is split into a KDV issue or OpenSpec change before editing the KDV repository

#### Scenario: KRR change is not required for MVP

- **WHEN** the MVP goal is safe static HTML file preview
- **THEN** KRR is not modified
- **THEN** CSS layout fidelity, JavaScript execution, iframe behavior, or pixel faithful rendering are deferred to a separate responsibility decision
