## ADDED Requirements

### Requirement: Release assets declare and satisfy startup compatibility

Every KatanA release asset MUST declare its supported CPU architecture and minimum OS. The packaged main executable and `kdv-office-worker` MUST support the same declared target, and the release gate MUST launch the packaged application on a clean machine for each declared target.

#### Scenario: Release artifact architecture is inspected

- **WHEN** a release candidate is packaged for macOS, Windows, or Linux
- **THEN** the gate verifies the main executable and Office sidecar architectures against the declared asset target
- **AND THEN** a generic asset name MUST NOT imply support for an architecture absent from either executable

#### Scenario: Clean machine starts the packaged application

- **WHEN** the release candidate is installed or extracted on each declared target
- **THEN** the application reaches an idle empty workspace without panic, loader error, or blocked UI heartbeat
- **AND THEN** the Office sidecar is present and executable without preventing base application startup

### Requirement: macOS packaging does not require paid Apple credentials

The macOS release workflow MUST remain executable without Apple Developer Program credentials. It MAY use ad-hoc signing, and the distribution documentation MUST accurately retain any Gatekeeper or quarantine step required by that choice.

#### Scenario: macOS release is built in CI

- **WHEN** the macOS app and DMG are packaged
- **THEN** the workflow does not require Developer ID certificate, Apple ID, team ID, or notarization secrets
- **AND THEN** the app remains ad-hoc signed and its documented Gatekeeper constraints remain explicit
