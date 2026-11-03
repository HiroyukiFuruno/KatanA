## ADDED Requirements

### Requirement: Host document closure isolates UI state

KatanA SHALL detach closed or loading document tabs, previews and host-owned watchers without blocking unrelated documents or claiming unavailable upstream worker cancellation.

#### Scenario: Close during loading and reopen

- **WHEN** representative Office or HTML input is closed during loading or after rendering
- **THEN** UI progress, unrelated tab state, switching and reopen are verified through the real host path

### Requirement: Published dependency acceptance is source bound

The release SHALL verify the current registry-resolved KDV/KRR chain with required original inputs and ten mixed Office/HTML cycles on the packaged application.

#### Scenario: Complete packaged document acceptance

- **WHEN** the final source and public package are available
- **THEN** input hashes, rendering/operation/close results, final idle and dependency identity are recorded
