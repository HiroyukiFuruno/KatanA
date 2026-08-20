## ADDED Requirements

### Requirement: v0.22.39 must consume the official Office conversion chain

KatanA v0.22.39 MUST resolve exact published `katana-document-viewer 0.5.3` from crates.io.
Its lockfile MUST resolve registry `office2pdf 0.6.7`, MUST NOT resolve
`office2pdf-katana`, and MUST NOT use path or git sources for KDV or the Office conversion chain.
KatanA MUST NOT add a direct Office/PDF engine dependency, parser, layout engine, font resolver,
or direct KUC dependency.

#### Scenario: Official chain is present

- **WHEN** v0.22.39 release readiness is evaluated
- **THEN** the exact KDV and office2pdf registry versions are resolved
- **AND THEN** KatanA remains a thin KDV host

#### Scenario: Retired or non-registry chain is present

- **WHEN** Cargo.toml or Cargo.lock resolves `office2pdf-katana`, a path source, or a git source
- **THEN** the release gate fails

### Requirement: v0.22.39 is the only allowed adjacent release target

The SemVer guard MUST accept only the adjacent update from published v0.22.38 to v0.22.39.
It MUST reject v0.22.38, v0.22.40, withdrawn v0.29.0, and all minor or major jumps.

#### Scenario: Adjacent patch is evaluated

- **WHEN** v0.22.39 is evaluated after v0.22.38
- **THEN** the guard accepts the target

#### Scenario: Non-adjacent version is evaluated

- **WHEN** any other target is evaluated after v0.22.38
- **THEN** the guard rejects the target
