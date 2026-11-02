## ADDED Requirements

### Requirement: Canonical diagram references bind to exact released source

Canonical sample diagram references SHALL use the declared input and 1280 by 2400 geometry, and bind output bytes to exact source, version, input hash and reproduction command.

#### Scenario: Adopt a released source's reference

- **WHEN** a candidate capture is generated for the final release source
- **THEN** its identity and geometry are verified before canonical adoption and a public handoff record is prepared

### Requirement: Reference publication does not claim downstream acceptance

KatanA SHALL publish its own capture evidence without treating KDV's future adoption as completed KatanA work or a blocker on unrelated fixes.

#### Scenario: Hand off the canonical artifact

- **WHEN** the verified reference is available
- **THEN** its public location and reproduction details are recorded and downstream responsibilities remain separate
