# v0.22.45 Build Identity — Issue362

## Definition of Ready

The selector fix is already published in v44. Raw three-platform identities were not emitted there. Current authorization covers reusable future release identity in the existing v45 checkout only.

## Branch Rule

Use `release/v0.22.45` in the existing checkout. No new branch/worktree, stash, force operation, sibling edits, or hook bypass. The parent integrates concern-specific commits and the release sequence.

## 1. Implementation and Targeted Validation

- [x] 1.1 Extract reusable payload/record mechanisms behind unchanged fixed-v42 acceptance adapters.
- [x] 1.2 Bind producer and verifier to canonical version, actual clean source, exact run/attempt, three manifests, and five assets.
- [x] 1.3 Wire all build producers and prepublication checks without changing existing approval or required job conditions.
- [x] 1.4 Add read-only automatic postpublication admission/download/identity verification with a distinct non-QA receipt.
- [x] 1.5 Pass real Git/archive/process regressions and preserved legacy approval/source/scope contracts.

Evidence: `tmp/update-check-failure/issue362-identity-tests.log` and `issue362-future-tests.log` record 14 tests passing under `python3 -S` with optional site packages disabled. Nine `issue362-test-*.log` suites record 180 preserved v42 acceptance/source/admission tests passing; strict PNG QA uses the existing `tmp/release-evidence-python` with the declared Pillow13 dependency. Synthetic binary-header/archive fixtures never establish public installer acceptance. YAML parsing, scoped diff checks, and OpenSpec strict validation pass. Actual CI/publication evidence remains incomplete in 3.3 and 3.4.

Final local summary: `tmp/update-check-failure/issue362-verification-summary.json` records 13 suites / 231 tests passing, including release-flow27 and CI-resource10. The strict acceptance result is in `issue362-test-document-fidelity-acceptance-evidence-correct-python.log`; the initial system-Python attempt lacked Pillow and is not used as a pass. Worker self-review found no remaining P0/P1; parent High review of the identity implementation also reported zero P0/P1/P2. Integration and the formal release sequence remain pending below.

## 2. Self Review and Feedback

### Definition of Ready

Implementation and focused checks exist. Parent review covers the distinct concern before normal integration.

- [x] 2.1 Review exact source/version/run/attempt binding, legacy policy preservation, workflow trust boundary, and fail-closed missing evidence.
- [x] 2.2 Resolve executable review findings without adding user-only acceptance conditions.

Parent High review reported zero P0/P1/P2 findings before the bounded v45 phase-plumbing correction. That correction passed an independent High peer review with no P0/P1/P2 findings; future CI/publication/merge records below remain incomplete until their actual results exist.

Bounded phase evidence: `tmp/update-check-failure/v45-phase-tests.log` records 14 existing/v45 tests passing, including the actual preflight task-dispatch fragment and same-version cross-change path refusal. Release-flow27, mode-binding6, and preflight-contract35 also pass. The two `v45-*-actual-bootstrap.log` files record successful checks of the physical storage/identity task files with only their specified future tasks pending. No CI/publication checkbox was marked complete by this phase correction.

The bounded v45 postpublication records route also passed independent High review with no P0/P1/P2 findings and 10 real Git/archive/process/API-contract regressions. It consumes the successful canonical Readiness artifact and validates live tag/build/attempt/asset identity; it does not claim actual v45 publication before that event.

## 3. Final Verification & Release Work

### Definition of Ready

Focused regressions and parent review pass. Formal release work remains owned by the parent session.

- [x] 3.1 Self-review and formatting; normal concern-specific commit and normal push hook.
- [ ] 3.2 Draft PR → current review → repair/reply/resolve → P0/P1 zero → Ready → required checks → normal merge.
- [ ] 3.3 Obtain actual v45 raw macOS/Linux/Windows identity manifests with the same source/run/attempt.
- [ ] 3.4 Confirm actual v45 public five-asset payload verification and preserve its identity-only receipt; do not relabel synthetic tests or v44 missing artifacts as success.
- [ ] 3.5 Archive the completed change after public evidence and safely clean local release state.

## Definition of Done

Agent-executable implementation, automated regression gates, normal review/publication, real CI manifests, and real public payload matching are evidenced. Document QA/95-score approval and user-only interaction remain outside this identity claim.

## KatanA CLI Entry Point

Use `./scripts/openspec` for status, instructions, and validation.

Actual normal push at 4b19a96a exited0; Draft PR402 Codex completed with no findings and full-page collection found zero unresolved threads. The remaining composite3.2 still includes Ready/required CI/merge and stays incomplete. Actual v45 raw/public identity evidence is still pending.
