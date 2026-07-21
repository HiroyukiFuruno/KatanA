## ADDED Requirements

### Requirement: Browser resources and embedded SVG remain KRR-owned

KRR MUST load policy-approved HTTP/HTTPS stylesheet, script, and image subresources and MUST render embedded SVG through the same in-process browser frame. An individual blocked or failed subresource MUST NOT replace the main document with a fatal viewer error. KRR MUST continue to reject HTTPS mixed content, credential-bearing network URLs, local file escapes, unsupported schemes, and iframe fetching.

#### Scenario: Render allowed cross-origin resources

- **WHEN** an HTTP or HTTPS document references an allowed cross-origin stylesheet, script, or image
- **THEN** KRR resolves and loads the resource without refetching the main document
- **THEN** the resulting CSS, JavaScript mutation, or image is visible in the complete browser frame

#### Scenario: Continue after a blocked subresource

- **WHEN** one stylesheet, script, or image is rejected by resource policy or fails transport
- **THEN** KRR logs the document origin, resource kind, reference, and cause
- **THEN** KRR renders the rest of the main document without the failed resource

#### Scenario: Render embedded Mermaid SVG

- **WHEN** the HTML document contains an embedded SVG produced by Mermaid
- **THEN** KRR preserves SVG namespace, case-sensitive attributes, viewBox, vector geometry, and CSS dimensions
- **THEN** the complete browser frame contains the diagram shapes, edges, and labels

### Requirement: Browser worker failures remain recoverable and traceable

KDV MUST keep its worker command loop alive when KRR session startup fails. A subsequent valid resize and navigation MUST be able to create a new session. KDV and KatanA MUST preserve layer, operation, complete document URL, and root cause in logs and the visible error. A later generic worker stop or panic MUST NOT overwrite an existing specific browser operation error.

#### Scenario: Recover after a startup failure

- **WHEN** the KRR session fails to start and KDV publishes the typed startup error
- **THEN** the KDV worker remains available for commands
- **WHEN** KatanA sends a valid viewport and navigation to a valid document
- **THEN** KDV starts a replacement KRR session and publishes its complete frame

#### Scenario: Display the primary failure context

- **WHEN** a browser operation fails inside KRR
- **THEN** the visible error and tracing log identify the KRR/KDV/KatanA layer, operation, complete document URL, and root cause
- **THEN** `browser worker has stopped` is shown only when no more specific primary error exists

#### Scenario: Reject stale recovery frames

- **WHEN** recovery produces an initial frame whose dimensions do not match the current visible viewport
- **THEN** KatanA discards that stale frame
- **THEN** the surface waits for an exact-viewport frame instead of displaying a resized bootstrap frame

## MODIFIED Requirements

### Requirement: v0.22.34 release must prove the published browser chain

KatanA v0.22.34 MUST consume published KDV `0.3.x` and KRR `0.4.x` crates from crates.io, with a minimum resolved version of KDV `0.3.2` and KRR `0.4.4`. KatanA MUST NOT use a local path or git dependency for KDV/KRR, or package a Chromium/browser runtime archive. The withdrawn `v0.29.0` MUST NOT be accepted as a release target, and the SemVer guard MUST accept only the adjacent update from published v0.22.33 to v0.22.34.

#### Scenario: Release evidence exercises browser behavior and recovery

- **WHEN** v0.22.34 release readiness is evaluated
- **THEN** packaged headless-process evidence covers external CSS/JavaScript/image, embedded Mermaid SVG, accordion, JavaScript action, text input, link navigation, reload, resize, worker error recovery, and complete action frames
- **THEN** same-document and external fragment states prove the complete document origin, raw KRR frame pixels, and composed KatanA screenshot pixels independently
- **THEN** image evidence proves fixed-background bordered controls on a light theme and preserves fullscreen dimensions through horizontal and vertical scrolling
- **THEN** the evidence shows no overlapping or clipped text in unchanged or updated regions

#### Scenario: Static or unpublished chain is detected

- **WHEN** KatanA resolves KDV below `0.3.2`, KRR below `0.4.4`, a path/git source, or includes a Chromium/browser runtime archive
- **THEN** the v0.22.34 release gate fails
- **THEN** a static screenshot or parser test cannot satisfy the release gate

#### Scenario: Non-adjacent release target is requested

- **WHEN** the latest published KatanA release is v0.22.33
- **THEN** the release guard accepts v0.22.34
- **THEN** the release guard rejects v0.22.33, v0.22.35, withdrawn v0.29.0, minor jumps, and major jumps
