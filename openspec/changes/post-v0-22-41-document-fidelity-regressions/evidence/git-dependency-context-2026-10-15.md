# Same-commit dependency path context

## Scope

Issue #335 remains open. `GitDependencyContext` derives declarations and their
provider/target records from one selected commit's private immutable file map.
Inherited paths resolve from the provider manifest; direct paths resolve from the
declaring manifest. Repository escape, absolute, drive and backslash paths remain
external candidates rather than receiving an internal exemption. Local missing,
non-package and mismatched-name targets fail explicitly.

The private package-name cache avoids repeated target TOML parsing for aliases.
The real 128-alias fixture checks returned targets; no timing improvement is
claimed. Workspace membership, package-version ownership, lock reachability,
public-registry evidence, trusted gate records and production-hook admission are
still unfinished. A matching path/name alone is not proof of internal ownership.

## Object authenticity regression

All mutations below affect isolated temporary repositories only, with inherited
Git environment removed and local hooks/signing disabled.

- Same-size SHA-256 loose blob mutation: native `cat-file -s` and `cat-file blob`
  succeeded, while the old reader incorrectly accepted the changed bytes.
  Session 29634 exited 1 with the actual missing-rejection assertion.
- The reader now recomputes the Git blob hash from the object header and exact
  bytes, accepting full SHA-1/SHA-256 identities and rejecting a mismatch.
  Session 58687 exited 0: 13 reader tests in 10.397 seconds.
- A same-size commit mutation redirecting to a different valid tree was rejected
  by native Git 2.56.0 with exit 128 and `hash mismatch`; the reader also returned
  `GitBindingError`. This was not an old-reader failure.
- A same-size tree mutation hiding `Cargo.toml` was also rejected by the reader.
  These native-Git boundaries remain explicit regression tests, not assumptions
  that every `cat-file` operation verifies content.

## Verification and review

- Initial same-commit context: session 57155, exit 0, 7 tests/9.840 seconds.
- Expanded suite session 70672 exposed an incorrectly escaped TOML test fixture;
  the fixture was corrected using a literal TOML string. This was not a product
  defect and the failing result is retained rather than counted as success.
- Release Python suite session 81275: exit 0, 153 tests/38.558 seconds before the
  final native commit/tree regression additions.
- Final release Python suite session 80502: exit 0, 155 tests/39.277 seconds,
  with warnings treated as errors and all final source changes included.
- Independent read-only review found no scoped P0/P1 after the native Git
  regression evidence; the earlier hypothetical commit/tree finding was
  withdrawn. The reviewer did not execute tests.

This evidence does not prove full Issue #335, all release gates, packaged
acceptance, startup recovery, or public release completion.

## Formal integration

Normal signed commits (signature status `G`):

- `36f8a08e43d0cd430ae5d092432f5fff5a4e9bca`: blob content identity and real regressions.
- `0279f3df7c01d54a37ec32042504cbe3bdcc0492`: same-commit path context and regressions.

Both normal commit commands exited 0. Upload/current review are separate steps.
