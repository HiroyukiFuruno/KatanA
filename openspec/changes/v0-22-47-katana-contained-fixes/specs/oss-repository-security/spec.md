## ADDED Requirements

### Requirement: KatanA-owned execution environments are maintained

The project SHALL identify image and virtual-environment ownership before updating KatanA-owned packages and CA bundles, and SHALL verify trusted TLS acceptance and untrusted certificate rejection.

#### Scenario: Audit an image or retained environment

- **WHEN** a listed environment is assessed
- **THEN** provenance, owner, update procedure and real positive/negative verification are recorded; unknown ownership is investigated rather than assumed external

### Requirement: Dependency exceptions retain their approval boundaries

The release SHALL reassess direct/transitive dependencies, actual API use and the existing ZIP exception's origin and expiry without silently extending it or lowering any quality gate.

#### Scenario: Select a compatible update

- **WHEN** a registry update is evaluated
- **THEN** exact versions/checksums, compatibility, affected APIs and complete quality results determine adoption, and true upstream residuals remain explicitly tracked
