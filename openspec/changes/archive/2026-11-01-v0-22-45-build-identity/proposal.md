# v0.22.45 Build Identity — Issue362

## Why

Release run selection is version-aware, but generated identity artifacts and their payload verifier are bound to v0.22.42. Later releases cannot prove the exact source/version/run/attempt for their five assets. The missing v44 raw manifests remain missing; this change must not manufacture them.

## What Changes

- Reuse the existing archive, architecture, Git-source, and run-admission mechanisms in a version-bound identity core.
- Emit three build manifests and verify all five payloads before release publication for every canonical release version.
- After the release workflow finishes, independently admit its run/attempt and download the actual public assets for the same strict verification.
- Preserve v42 document approval, host evidence, scores, frozen producer hashes, and controller scope. Identity success makes no document QA claim.

## Capabilities

### New Capabilities

- `generated-release-identity`: exact generated/public payload provenance without document acceptance.

### Modified Capabilities

None.

## Impact

Release helper modules, build/readiness identity steps, and focused Python/Git/archive contracts only. No app/runtime, native latency budgets, dependency waiver, or required job changes.
