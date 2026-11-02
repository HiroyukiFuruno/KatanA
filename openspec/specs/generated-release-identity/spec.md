# generated-release-identity

## Purpose

Preserve generated v0.22.45 payload provenance and actual public download identity. The current PR-closed and same-source dispatch bindings are implemented; dispatch that creates a new version-bump commit remains an explicit follow-up in https://github.com/HiroyukiFuruno/KatanA/issues/403 and is not claimed supported here.

## Requirements

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

### Requirement: Manual bump provenance binds generated source to its trigger

The verifier SHALL bind a manual bump's generated commit chain to the same repository, workflow, run and attempt using verifiable metadata and allowed version-file changes.

#### Scenario: Accept a genuine version-only generated chain

- **WHEN** real Git/process and metadata contracts establish the valid trigger and generated source
- **THEN** the verifier accepts that chain without requiring false equality between trigger and generated commits

#### Scenario: Reject an unrelated or modified generated source

- **WHEN** repository, run, attempt, source, intermediate chain, allowed files or version differ
- **THEN** verification fails and existing same-source dispatch, PRclosed and fixed v42 policy remain enforced

### Requirement: Public receipt claims match the actual publication path

The release SHALL collect a receipt from actual public metadata and record acceptance and rejection results without relabeling no-bump publication as generated-bump publication.

#### Scenario: Collect the single batch's actual receipt

- **WHEN** v0.22.47 is published once
- **THEN** source, run, attempt, manifests and asset bytes are checked and the actual event path is identified
