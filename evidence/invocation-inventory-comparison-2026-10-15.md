# Observed invocation comparison diagnostics

Issue [#343](https://github.com/HiroyukiFuruno/KatanA/issues/343) requires a
machine comparison before removing duplicate checks. The existing collector and
per-list parser did not connect normal workspace execution to the four coverage
phase records. This change adds that diagnostic connection, not skip authority.

## Comparison boundary

- Raw all/ignored listings are reparsed with the existing strict parser. Their
  SHA256, serialized cases/footer, ignored subset and listing argv must agree.
- Each listing must match its emitted Cargo artifact, including target source,
  package, manifest and target kind. Executable paths and listing-time digests
  are observations, not interchangeable target identities or execution proof.
- Requested Cargo arguments reconstruct the recorded executed arguments through
  the existing recorder contract. Source HEAD/tree, clean status and both locks
  must remain stable within each run. Conditions/features/digests are compared
  explicitly between runs rather than normalized away.
- Supported native selection is deliberately restricted to the current whole-run
  forms, ignored/include-ignored, substring skip and thread/output flags. Exact
  or positional filtering, benchmark selection and unknown options are
  unavailable, not approximated.
- The CLI loads the normal record and four fixed coverage phase files, verifies
  each phase's production selectors, and records input file digests. Missing or
  invalid observations never become equal empty candidate sets.
- Candidate intersection and one-sided differences are not executed/pass counts.
  Every report fixes `execution_equivalent=false` and `reuse_authorized=false`.
  Doctest, hermetic environment, execution-time binary identity and trusted CI
  provenance remain unproved. Dirty/failing runs cannot authorize reuse.

## CI integration

The existing macOS test job compares the records after coverage and before its
existing diagnostic upload, including after failure. Missing phase files produce
an explicit unavailable diagnostic; malformed input fails the comparator.
Original failed test steps remain failed. The comparison never invokes Cargo,
test binaries or an additional coverage run. The original job-level Draft guard,
test commands, coverage exclusions/thresholds and platform gates are unchanged.
Normal, coverage, comparison and upload share a directory scoped to the GitHub
run ID and attempt. Restored target caches cannot supply an older run's records.

## Verification

- New-feature RED: comparator absent, seven tests/fourteen failing subcases,
  actual exit1. This is missing-feature evidence, not a historical product bug.
  Raw `tmp/invocation-comparison-feature-red-2026-10-15.log`.
- Initial fixture arguments did not follow the recorder's Cargo contract and
  produced four assertions. Correcting the fixture to use the real
  `cargo_command()` transformation yielded seven passing comparison tests.
- Additional malformed identity/argv/raw/artifact/status/empty inventory and
  selection boundaries: eight tests passed. Pure tests do not prove native
  execution equivalence.
- Intermediate existing native inventory discovery: 53 passed in11.308seconds,
  actual exit0; `tmp/invocation-comparison-targets-2026-10-15.log`. Later additions
  require a final targeted collection.
- Wiring RED on the missing comparison step was recovered by adding the step.
  Default actionlint v1.7.12 including ShellCheck exited0, session51247;
  `tmp/invocation-comparison-actionlint-2026-10-15.log`.

- Final discovery: 68 tests passed in27.704seconds, session69205 actual exit0;
  `tmp/invocation-comparison-final-targets-v2-2026-10-15.log`. This includes a real
  temporary native Cargo/Git workspace, the recorder and four real llvm-cov
  selections. The CLI reports the deliberately skipped fixture as normal-only,
  while retaining false execution-equivalence and reuse authority.
- Malformed JSON, duplicate keys, oversized manifests, incomplete phases,
  fresh run scope beside populated old outputs and temporary summary output
  isolation are tested. Unsupported native collection remains unavailable.
- Independent review identified unknown environment keys, lost raw listing
  differences and stale cached observations. Their regression failures were
  repaired. A separate source-identity regression was repaired so different
  normal/coverage source identities cannot yield an available comparison.
- Final default actionlint including ShellCheck: session22996 actual exit0;
  `tmp/invocation-comparison-final-actionlint-2026-10-15.log`.
- Final independent read-only review found no P0/P1 or mandatory in-scope P2.
  Self-review: existing test/coverage/Draft gates remain unchanged; no product
  API, dependencies, bypass, test ignore or skip authority was added. Diff check
  passed. Source and workflow changes form one concern.

Normal integration and new-HEAD review are still pending. Windows
cold/warm measurements, complete test-set equivalence, protected result reuse,
measured total-cycle reduction and full Issue #343 completion remain separate.
The user application/settings/Terms and sibling repositories are not operated.
