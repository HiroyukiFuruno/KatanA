# Issue lookup payload and actual commit binding

Issue: https://github.com/HiroyukiFuruno/KatanA/issues/335

Source: `3022686cc0b4acdd4a67507b3b4708f53db5b551`, normal signed commit.
Final independent HIGH diff review found no remaining P0/P1 in this slice.

This helper binds lookup payload identity, the expected real full commit, and
the Issue's declared manifest/lockfile blobs. It does not authenticate an
arbitrary supplied payload, orchestrate live transport, prove publication,
verify Cargo semantics, execute commands from Issue text, certify strict logs,
or activate production admission.

## Actual checks

- Real temporary SHA-256 Git commits provide actual regular manifest/lock
  blobs. The expected source must be a full available commit, and a declared
  record must include it. Historical evidence cannot substitute for the next
  expected commit, even if its historical tree remains valid.
- PR metadata, wrong repository/number, missing or invalid body, missing or
  abbreviated expected objects, and altered blob identity are rejected.
- Null, empty, or normal non-evidence bodies remain `None`, meaning
  unclassified. That is not an exemption or admission success.
- The initial new test run failed with eight errors due to a fixture attribute
  shadowing its helper method. That test-construction error was corrected; it
  is not recorded as a pre-existing product regression.
- Independent HIGH review identified a standalone nonpositive-number boundary.
  Actual zero/negative metadata with real Git commits was accepted by the old
  helper: six tests, two assertion failures, exit 1. Reusing the existing
  positive-integer candidate validator repaired that boundary.
- Single final discovery: 68 tests, 6.094 seconds, actual exit 0.
- Real read-only GitHub lookup of Issue 335 and current public full commit
  `2144131a59e540a5d321fc97de525f36c6d35f5f` returned unclassified evidence,
  actual exit 0. A real PR 346 lookup was rejected as an Issue binding,
  actual exit 0. Neither case claims published downstream evidence acceptance.

Raw local logs: `tmp/issue335-issue-commit-binding-first-2026-10-14.log`,
`tmp/issue335-issue-number-binding-red-2026-10-14.log`,
`tmp/issue335-issue-commit-binding-final-2026-10-14.log`,
`tmp/issue335-actual-issue-commit-unclassified-2026-10-14.log`, and
`tmp/issue335-actual-pr-commit-rejection-2026-10-14.log`.

The caller must obtain the genuine lookup response and select actual commit
Issue references. Payload validation is not transport provenance. The future
admission owner still needs downstream classification, source-bound Cargo
content checks, registry/Release verification, trusted strict execution proof,
existing-hook-first activation, and public-release-only safe cleanup. The
complete Issue, Ready gates, packaged acceptance, and release remain open.
