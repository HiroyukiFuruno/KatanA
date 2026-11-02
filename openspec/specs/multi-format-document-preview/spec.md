# Multi-format Document Preview

## Purpose

KatanA hosts published KDV document frames for PDF, DOCX, XLSX, and PPTX. KatanA remains a thin consumer: document parsing, conversion, layout, fonts, and session policy stay below the KatanA -> KDV -> KUC/KRR boundary.

## Requirements

### Requirement: v0.22.40 must consume the published external-hyperlink fix

KatanA v0.22.40 MUST resolve exact published `katana-document-viewer 0.5.4` from crates.io. Its lockfiles MUST resolve registry `office2pdf 0.6.7`, MUST NOT resolve `office2pdf-katana`, and MUST NOT use path or git sources for KDV or the Office conversion chain. KatanA MUST NOT add a direct Office/PDF engine, archive parser, layout engine, font resolver, hyperlink handler, or direct KUC dependency.

#### Scenario: Published KDV chain is present

- **WHEN** v0.22.40 release readiness is evaluated
- **THEN** the exact KDV and official office2pdf registry versions are resolved
- **AND THEN** KatanA remains a thin KDV host

#### Scenario: Retired or non-registry chain is present

- **WHEN** Cargo.toml or Cargo.lock resolves `office2pdf-katana`, a path source, or a git source
- **THEN** the release gate fails

### Requirement: Passive OOXML external hyperlinks render through the native host

KatanA MUST pass a PPTX containing a standard OOXML external hyperlink relationship to the published KDV session unchanged. The native document surface MUST receive a PPTX `Page` frame and MUST NOT show a preflight rejection. KatanA MUST NOT request the external target.

#### Scenario: Local external-hyperlink PPTX is opened

- **WHEN** the existing headless KatanA document scenario opens the generated local fixture
- **THEN** the first complete frame reports `pptx` and `Page`
- **AND THEN** the document surface does not expose a KDV failure diagnostic

### Requirement: v0.22.40 is the only allowed adjacent release target

The SemVer guard MUST accept only the adjacent update from published v0.22.39 to v0.22.40. It MUST reject v0.22.39, v0.22.41, withdrawn v0.29.0, and all minor or major jumps.

#### Scenario: Adjacent patch is evaluated

- **WHEN** v0.22.40 is evaluated after v0.22.39
- **THEN** the guard accepts the target

#### Scenario: Non-adjacent version is evaluated

- **WHEN** any other target is evaluated after v0.22.39
- **THEN** the guard rejects the target

### Requirement: focus中のPDF/PPTX surfaceは範囲内の左右ページ移動を受け付ける

KatanA MUST PDF/PPTX document surfaceがkeyboard focusを所有している間に限り、modifierのないleft/right arrow keyを既存のprevious/next page commandへ転送する。commandは先頭pageの前や最終pageの後へ進めてはならない。他format、modifier、または別widgetが所有するfocusを横取りしてはならない。

#### Scenario: focus中のPDF/PPTXが範囲内arrow keyを受け付ける

- **WHEN** focus中のPDF/PPTX surfaceがpage範囲内でmodifierのないleft/right arrow keyを受け取る
- **THEN** KatanAが対応する既存previous/next page commandをdispatchする

#### Scenario: 別focus・他format・modifier・境界では移動しない

- **WHEN** 別widgetがfocusを持つ、active formatがPDF/PPTXでない、modifierが押されている、または現在pageが該当する境界にある
- **THEN** KatanAはpage navigation commandをdispatchしない

### Requirement: All document controls share format and runtime policy

KatanA SHALL derive every vertical sidebar operation, code/split mode, shortcut, command palette entry and direct action guard from one document operation settings policy. PPTX slideshow readiness SHALL come from the current live surface; binary PDF/Office documents SHALL reject unsupported code/split modes.

#### Scenario: Ready PPTX and unsupported binary mode

- **WHEN** a current PPTX surface becomes ready or a binary document receives a mode command
- **THEN** slideshow controls reflect the live readiness at every entry, while unsupported code/split controls and actions remain unavailable

#### Scenario: Same path reopened or active tab changes

- **WHEN** the active document identity, revision or preview generation changes
- **THEN** stale runtime readiness is not reused and palette/sidebar/shortcut/direct-dispatch decisions remain consistent

### Requirement: Document loading uses one localized presentation

KatanA SHALL use one reusable loading component for images, HTML, Office/PDF, Canvas and section loading, passing the local state and resolved localized text without exposing template placeholders.

#### Scenario: Pending document frame

- **WHEN** any supported document is waiting for its first frame
- **THEN** the shared loading presentation is shown and no literal `{kind}` placeholder appears

### Requirement: HTML and Office close preserve UI progress and resource cleanup

KatanA SHALL separate close-action response from background teardown, preserve eventual zero-resource cleanup and the existing memory budget, and record HTML and Office phase timings independently before claiming a common root cause.

#### Scenario: Close a loaded HTML or Office document

- **WHEN** a loaded HTML, DOCX, XLSX or PPTX tab is closed
- **THEN** the remaining interface can progress and teardown reaches the existing idle/resource and memory conditions without weakening them

### Requirement: Office embedded font names are checked before conversion

KatanA SHALL apply the same bounded DOCX/PPTX embedded-font validation at local and remote source intake before handing bytes to the document worker. Embedded font names containing path separators, colon or NUL, ambiguous archive names and malformed XML SHALL be rejected without writing extracted fonts. Normal Unicode names SHALL remain accepted. Existing source and metadata size limits SHALL be preserved.

#### Scenario: Unsafe embedded font name from local or remote input

- **WHEN** a DOCX or PPTX contains a known embedded-font marker and an unsafe font name, including encoded XML attribute forms
- **THEN** the shared source intake reports a validation error before conversion and the document worker does not receive those bytes

#### Scenario: Normal input and other document formats

- **WHEN** a normal Unicode font name or an ordinary Office document is opened
- **THEN** the shared validation accepts it within the existing limits
- **AND** PDF and XLSX do not execute the DOCX/PPTX font parser, while their existing format checks remain in effect

### Requirement: Loading presentation SHALL be centered in the document viewport

The host SHALL use one shared loading presentation for supported document types and place its visible indicator and label at the horizontal and vertical center of the available document viewport, including nested centered layouts and bounded inline surfaces.

#### Scenario: PPTX is loading in a centered preview surface

- **WHEN** the document preview has no rendered frame yet
- **THEN** the shared indicator rectangle is centered on both axes of the preview area
- **AND** its localized label contains no unresolved placeholder

### Requirement: Active document paging SHALL respect input ownership

The host SHALL allow unmodified left and right arrows to page an activated PDF or PPTX without an undocumented extra click, while respecting text entry, search, modal dialogs, modified shortcuts, and page boundaries.

#### Scenario: User selects a paginated document before pressing an arrow

- **WHEN** a ready PDF or PPTX becomes the active preview and no later text or modal interaction owns the input
- **THEN** an unmodified right arrow requests exactly one next page when a next page exists
- **AND** tab shortcuts and text editing are not triggered by that same input

### Requirement: Pending document conversion SHALL NOT block independent image presentation

The host SHALL keep Explorer selection and local image read/decode/paint responsive while Office or HTML work is pending. The host SHALL NOT require an unrelated document worker to finish before a local image can be painted.

#### Scenario: Image is selected while Office or HTML remains pending

- **WHEN** a small valid local PNG is selected while other document work has not produced a frame
- **THEN** its production request path can progress to a painted texture without waiting for that unrelated work to finish
- **AND** unchanged repeated frames do not reset the image decode or texture

### Requirement: Host document closure isolates UI state

KatanA SHALL detach closed or loading document tabs, previews and host-owned watchers without blocking unrelated documents or claiming unavailable upstream worker cancellation.

#### Scenario: Close during loading and reopen

- **WHEN** representative Office or HTML input is closed during loading or after rendering
- **THEN** UI progress, unrelated tab state, switching and reopen are verified through the real host path

### Requirement: Published dependency acceptance is source bound

The release SHALL verify the current registry-resolved KDV/KRR chain with required original inputs and ten mixed Office/HTML cycles on the packaged application.

#### Scenario: Complete packaged document acceptance

- **WHEN** the final source and public package are available
- **THEN** input hashes, rendering/operation/close results, final idle and dependency identity are recorded
