## MODIFIED Requirements

### Requirement: v0.22.35 release must prove the published browser chain

KatanA v0.22.35 MUST consume published KDV `0.3.x` and KRR `0.4.x` crates from crates.io, with a minimum resolved version of KDV `0.3.3` and KRR `0.4.5`. KatanA MUST NOT use a local path or git dependency for KDV/KRR, or package a Chromium/browser runtime archive. The withdrawn `v0.29.0` MUST NOT be accepted as a release target, and the SemVer guard MUST accept only the adjacent update from published v0.22.34 to v0.22.35.

#### Scenario: Document lifecycle initializes the interactive fixture

- **WHEN** external JavaScript registers its controls through `document.addEventListener("DOMContentLoaded", ...)`
- **THEN** KRR advances `document.readyState` through `loading`, `interactive`, and `complete` in browser order
- **THEN** the initial complete frame proves the listener ran before accordion, click, input, and navigation interactions are exercised

#### Scenario: Runtime failure remains traceable

- **WHEN** a lifecycle listener throws a JavaScript exception
- **THEN** the visible KatanA error preserves the KRR/KDV operation context and V8 stack or script/line/column/source location
- **THEN** a later generic worker status does not replace the primary cause

#### Scenario: Static or unpublished chain is detected

- **WHEN** KatanA resolves KDV below `0.3.3`, KRR below `0.4.5`, a path/git source, or includes a Chromium/browser runtime archive
- **THEN** the v0.22.35 release gate fails
- **THEN** a static screenshot or parser test cannot satisfy the release gate

#### Scenario: Non-adjacent release target is requested

- **WHEN** the latest published KatanA release is v0.22.34
- **THEN** the release guard accepts v0.22.35
- **THEN** the release guard rejects v0.22.34, v0.22.36, withdrawn v0.29.0, minor jumps, and major jumps

