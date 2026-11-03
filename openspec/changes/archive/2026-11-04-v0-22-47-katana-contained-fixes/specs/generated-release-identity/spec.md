## ADDED Requirements

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
