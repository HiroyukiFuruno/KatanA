## ADDED Requirements

### Requirement: Exact generated release provenance

Every canonical release version SHALL emit macOS, Linux, and Windows build-job identity manifests and require their exact product commit, source tree, lockfile, registry dependencies, workflow run, attempt, version, targets, and all five asset bytes to match before publication.

#### Scenario: A manifest or identity differs

- **WHEN** a manifest is missing or its source, version, run, attempt, target, or asset bytes differ
- **THEN** publication identity verification fails without fallback or synthetic replacement

### Requirement: Independent public payload identity

After the actual release workflow completes, a read-only verifier SHALL admit its published tag and successful required jobs, freshly download the actual public assets, and verify them against all three build manifests. Its receipt SHALL make no document QA or fidelity approval claim.

#### Scenario: A historical release has no raw manifests

- **WHEN** the admitted run lacks any generated platform identity artifact
- **THEN** the verifier reports failure and never reconstructs a historical build manifest

### Requirement: Preserve historical acceptance policy

The v0.22.42 document approval, host acceptance, score gates, historical download producer hash, and frozen controller scope SHALL remain unchanged by future version identity support.

#### Scenario: Future metadata is offered to fixed acceptance

- **WHEN** a future-version manifest is supplied to the fixed-v42 acceptance adapter
- **THEN** that adapter rejects it even if generic identity-only verification would accept its version
