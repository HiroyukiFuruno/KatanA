## ADDED Requirements

### Requirement: External distribution records target-specific failures

KatanA SHALL preserve the actual Homebrew and WinGet sync outcome separately as structured results and visible workflow diagnostics. Unconfigured, failed, and succeeded targets SHALL not be conflated.

#### Scenario: One registry update fails

- **WHEN** one external registry command exits unsuccessfully
- **THEN** its failed outcome and diagnostic receipt are retained without falsely reporting registration success

### Requirement: Homebrew generation is valid and test-isolated

The generator SHALL use supported Cask DSL and reject invalid or zero digests before authentication or network access. Its automated tests SHALL verify fixture command/credential isolation before invoking the generator.

#### Scenario: Invalid digest or unisolated test process

- **WHEN** a zero digest is supplied or a test fixture cannot prove its curl and token isolation
- **THEN** no external metadata write occurs
