## ADDED Requirements

### Requirement: AST lint separation is the first repository split

The system SHALL separate shared AST lint governance before KME implementation work begins.

#### Scenario: Prioritize repository separation

- **WHEN** KatanA ecosystem repository separation is planned
- **THEN** `katana-ast-lint` is treated as P0
- **THEN** `katana-markdown-engine` is treated as P1
- **THEN** `katana-ui-widget` is treated as P2
- **THEN** other integrations are treated as P3

### Requirement: AST lint exposes a shared violation contract

The system SHALL define a shared violation format for separated repositories.

#### Scenario: Report a violation

- **WHEN** AST lint finds a repository violation
- **THEN** the result includes rule id, severity, file, range, message, and remediation guidance
- **THEN** downstream repositories can consume the result without custom parsers

### Requirement: Repository adapters keep local concerns outside shared rules

Shared AST lint rules MUST NOT hard-code KatanA repository-local paths.

#### Scenario: Run lint in a separated repository

- **WHEN** AST lint runs in `katana-markdown-engine`
- **THEN** repository-specific file discovery is handled by an adapter
- **THEN** shared rules remain reusable across KME, kdp, kle, kcf, and kuw

### Requirement: KatanA consumes the shared AST lint crate

KatanA SHALL consume `katana-ast-lint` through workspace dependency and test/CI entrypoints instead of preserving duplicated shared rules in `crates/katana-linter`.

#### Scenario: Run KatanA AST lint after extraction

- **WHEN** a developer runs KatanA `just ast-lint`
- **THEN** the gate invokes `katana_ast_lint` rule APIs through KatanA's repository adapter or test runner
- **THEN** shared rules are not reimplemented in KatanA-local linter code
- **THEN** any remaining KatanA-local linter code is limited to repository-specific adapter or runner responsibilities
