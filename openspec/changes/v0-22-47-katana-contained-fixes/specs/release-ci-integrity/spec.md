## ADDED Requirements

### Requirement: Remote branch cleanup requires explicit opt-in

Release cleanup SHALL preserve remote refs by default and when its option is missing; explicit opt-in SHALL retain every existing exact-target, SHA, provenance, owner and normal-hook guard.

#### Scenario: Default, missing and explicit remote cleanup

- **WHEN** remote cleanup is false or unspecified
- **THEN** remote deletion is not executed and the receipt truthfully records that fact
- **WHEN** explicit opt-in is provided with invalid target evidence
- **THEN** deletion is refused

### Requirement: CI result reuse preserves complete quality semantics

CI SHALL reuse or skip work only with verified complete input equivalence and a trusted successful run, and SHALL retain build, test, lint, coverage, CodeQL and supply-chain requirements.

#### Scenario: Reuse eligibility or mismatch

- **WHEN** source, toolchain, lock, features, fixtures, environment or evidence differ or are incomplete
- **THEN** reuse is refused and required verification runs normally

### Requirement: Cache and inventory claims are measured

Windows cold/warm cache measurements and macOS normal/coverage inventory comparisons SHALL record comparable inputs, complete results, elapsed times and explained differences.

#### Scenario: Measure actual runner outcomes

- **WHEN** candidate CI efficiency is assessed
- **THEN** actual run receipts establish eligibility and measured effects without suppressing unproven cases
