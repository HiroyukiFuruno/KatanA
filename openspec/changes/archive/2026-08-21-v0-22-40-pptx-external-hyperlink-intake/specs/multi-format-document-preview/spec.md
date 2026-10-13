## ADDED Requirements

### Requirement: v0.22.40 must consume the published external-hyperlink fix

KatanA v0.22.40 MUST resolve exact published `katana-document-viewer 0.5.4` from crates.io.
Its lockfiles MUST resolve registry `office2pdf 0.6.7`, MUST NOT resolve
`office2pdf-katana`, and MUST NOT use path or git sources for KDV or the Office conversion
chain. KatanA MUST NOT add a direct Office/PDF engine, archive parser, layout engine, font
resolver, hyperlink handler, or direct KUC dependency.

#### Scenario: Published KDV chain is present

- **WHEN** v0.22.40 release readiness is evaluated
- **THEN** the exact KDV and official office2pdf registry versions are resolved
- **AND THEN** KatanA remains a thin KDV host

#### Scenario: Retired or non-registry chain is present

- **WHEN** Cargo.toml or Cargo.lock resolves `office2pdf-katana`, a path source, or a git source
- **THEN** the release gate fails

### Requirement: Passive OOXML external hyperlinks render through the native host

KatanA MUST pass a PPTX containing a standard OOXML external hyperlink relationship to the
published KDV session unchanged. The native document surface MUST receive a PPTX `Page` frame
and MUST NOT show a preflight rejection. KatanA MUST NOT request the external target.

#### Scenario: Local external-hyperlink PPTX is opened

- **WHEN** the existing headless KatanA document scenario opens the generated local fixture
- **THEN** the first complete frame reports `pptx` and `Page`
- **AND THEN** the document surface does not expose a KDV failure diagnostic

### Requirement: v0.22.40 is the only allowed adjacent release target

The SemVer guard MUST accept only the adjacent update from published v0.22.39 to v0.22.40.
It MUST reject v0.22.39, v0.22.41, withdrawn v0.29.0, and all minor or major jumps.

#### Scenario: Adjacent patch is evaluated

- **WHEN** v0.22.40 is evaluated after v0.22.39
- **THEN** the guard accepts the target

#### Scenario: Non-adjacent version is evaluated

- **WHEN** any other target is evaluated after v0.22.39
- **THEN** the guard rejects the target
