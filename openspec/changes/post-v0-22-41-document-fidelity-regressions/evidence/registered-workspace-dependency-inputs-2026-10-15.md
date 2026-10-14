# Registered workspace dependency inputs

## Scope

Issue #335 remains open. This candidate binds committed Cargo inputs to registered
workspace package paths, names, and resolved versions. It compares all committed
Cargo manifests and lockfiles, not only author-listed changed paths.

The production dependency admission checker, publication verification, activation
of the dispatcher, and release cleanup integration remain incomplete. No actual
repository hook or user application settings have been changed by this slice.

## Actual checks

- Native Rust SemVer binary: `cargo test --locked -p katana-linter --bin
  dependency-contract`, actual exit 0, four tests. Exact, caret, compound, and
  prerelease requirements use the existing locked `semver` crate, not a Python
  approximation. Only the linter's existing lock record gained the dependency.
- Native test-inclusive strict Clippy: actual exit 0, session 65919.
- Official `just fmt-check`: actual exit 0, session 36237.
- Official `just ast-lint`: actual exit 0, session 57409, 23 tests. This does not
  prove the separately tracked defective function-length rule is fixed.
- Workspace and initial snapshot checks: actual exit 0, session 18388, 20 tests.
- Dispatcher/installer/workspace checks: actual exit 0, session 22159, 25 tests.
  Dispatcher tests prove sequencing and HEAD-change rejection, not complete
  production admission. Actual hooks were not installed.

## Reproduced and repaired boundaries

1. Native Cargo accepts an omitted package version as `0.0.0`; the candidate
   rejected it. The real Cargo comparison failed before the fix (actual exit 1),
   then passed with the direct package default. Missing workspace-inherited
   version remains invalid rather than silently becoming the default.
2. A workspace member's adjacent stray lockfile borrowed the root workspace's
   internal package exemptions. A real committed dependency-edge update was
   incorrectly omitted: session 20526, actual exit 1, expected one change but
   observed zero. Exemption now requires the manifest itself to own that lock.
   Snapshot suite session 2593: actual exit 0, seven tests, 6.654 seconds.
3. A root-only external dependency update must reach an unchanged member blob.
   Exact owned lock identity is exempt; registry and different-version identities
   remain candidates. Independent workspace locks cannot borrow other roots'
   identities. Nested checksum-only changes and committed input versus dirty
   working-tree isolation are covered by real temporary Git repositories.

The first snapshot suite's invalid bootstrap lock fixture produced parser errors,
not product regression assertions. It was replaced with a valid package record.
The earlier 186/188-test runs preceded the last exclusion repairs and are not
final-source proof. Fresh frozen-source discovery is being collected separately.

4. Cargo's excluded directory also excludes its descendants, while explicitly
   listed members take precedence. Both candidate mismatches were confirmed
   against actual Cargo metadata (50879 and the explicit-member run, exit 1).
   Exclusion now uses normalized component-prefix matching with literal member
   priority instead of glob exclusion. Workspace 16 and snapshot seven checks:
   session 29246, actual exit 0, 23 tests, 20.345 seconds.

Source concern was integrated with normal signed commit
`2ee119579393edd1732a482f50c5baa732e2c7f8` (session 40834, exit 0, signature G).
Independent read-only re-review found no additional P0/P1 in this limited slice.
Python AST function-span inspection is empty after the installer's separate
responsibility split. Those dispatcher files remain separate uncommitted work.
Final release Python discovery: session 60663, actual exit 0, 190 tests,
62.552 seconds. This includes the unactivated dispatcher sequencing tests and
does not make those uncommitted files part of the uploaded source concern.

## Remaining acceptance

Current-head GitHub review and normal push are not yet proven.
No previous HEAD's CI, packaged application acceptance, or public release is
substituted by these targeted checks. Full Issue #335 and v0.22.42 release DoD
remain active; upstream defects and conditional safe startup remain in their
previously agreed scope.
