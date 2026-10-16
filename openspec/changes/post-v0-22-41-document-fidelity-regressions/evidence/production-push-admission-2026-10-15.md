# Production push admission candidate — 2026-10-15

## Scope and status

Issue #335 remains open. This candidate connects real pushed commits and branch-scoped Issue references to whole-tree Cargo input, native version-requirement, registry publication and fresh owner-check contracts. It is not evidence of a completed release or an installed production dispatcher.

The existing editable worktree remains on `release/v0.22.42`. Master is clean, stash count is zero and the two existing worktrees are retained. No application, user settings, Terms acceptance, sibling source or monitoring schedule was changed.

## Fixed admission defects

- A successful retained hook does not imply that its quality command ran. The dispatcher now creates a private per-push session; only the fixed `just check` owner command issues a receipt matching that session, source revision and root after actual success. Missing receipts remain unverified. Issue-authored command/exit fields are not executed or trusted as execution receipts.
- The pre-push selector includes root and nested Cargo inputs, standalone graph inputs and owner configuration. Actual Lefthook/Just miniature fixtures verify selection, execution order, missing receipts, exclusion and source binding; those fixtures are not production quality-gate results.
- The installer rejects hooks-directory symlinks that leave the canonical common Git directory, preserves foreign hook bytes/mode/config, closes the temporary-wrapper descriptor and rejects incompatible registered worktree runtimes before altering any hook.
- Publication verification checks public repository identity and visibility before stable Release identity, then registry checksum and the actual crate's metadata. Compressed response, gzip expansion, internal PAX/GNU metadata reads and member metadata have explicit size/deadline boundaries. No archive is extracted into the filesystem.

## Observed results

| Check | Actual result | Scope |
| --- | --- | --- |
| Skipped owner was formerly labelled verified | Assertion failed before repair | Dispatcher receipt binding |
| Foreign hooks-directory symlink | Returned 0 before repair; installer suite then passed 10 tests | Real temporary Git, no foreign mutation after repair |
| Root/standalone hook input selection | Six cases failed before repair; three contract tests passed afterward | Actual Lefthook 2.2.0 and fixed Just fixture wiring |
| Focused candidate suite | Session 67344: 53 passed, 18.095 seconds | Candidate Python contracts |
| Release Python discovery | Session 9207: 223 passed, 74.658 seconds | Intermediate `test_*.py` candidate |
| Broader release discovery | Session 95494: 226 passed, 74.217 seconds | Intermediate `test*.py` candidate |
| Final release discovery | Session 94203: 228 passed, 75.449 seconds | Final candidate, `test*.py`, warnings treated as errors |
| Final archive/publication focused repair | 15 passed | Includes bounded PAX parse; not HTTP mock E2E |
| Release ownership and Draft CI contracts | 10 and 5 passed respectively | Fixed owner entrypoint and unchanged Ready-only policy |
| Public KDV 0.5.12 | Session 70942: exit 0 | Public repo/Release/index/crate checksum/identity |
| Public KRR runtime/assets 0.4.23 | Session 34686: exit 0 | Both public registry packages; no CLI claim |
| Final bounded public-artifact batch | Session 43410: exit 0 | KDV, KRR runtime and assets after internal-header read repair |

Final independent read-only review found no remaining P0/P1 in this diff. It did not execute tests or approve production activation, all of Issue #335, or release completion. All 24 changed/new Python files were parsed; new/changed functions and candidate files meet the 30/200-line constraints. The pre-existing `main` function in the release-flow test was not rewritten; its assertion now checks the actual fixed owner entrypoint.

The native macOS readiness and release preflight workflows install their required Lefthook/Just tools before running these contracts. Draft runner conditions and original quality thresholds are unchanged. There is one fixed owner command, not a second manual `just check` before push.

## Ordinary source integration

- Publication and bounded-artifact concern: `0d56d696137acac9c5e3ca09b7b83aea7467ecfa`; commit session 49598 actual exit0, signature G.
- Branch-scoped dependency-admission concern: `00624b789c8106b9e1b049965a41c0e82baa51b9`; commit session 91860 actual exit0, signature G.
- Owner receipt / safe installer / native contract-tool connection: `23222c201bbbb7af2911fa97c8bbb61eb4b2f3b6`; commit session 26132 actual exit0, signature G.

These ordinary commits integrate source only. The one normal push result and exact remote/current review still need to be collected.

## Remaining completion requirements

- Collect the final candidate's complete Python results, independent review, ordinary signed concern commits and one normal push result. Intermediate tests above must not be reused as final-source proof.
- Verify a complete genuine downstream-admission positive path and native hook connection. Wiring fixtures alone do not prove that path. Shared hook activation remains prohibited until every registered worktree has a compatible runtime; fixed-worktree fallback is not used.
- Implement and verify Issue #335's publication-gated safe cleanup, including retain/failure conditions and release workflow connection.
- Preserve the full current-HEAD review, Ready CI/coverage, original-resource and packaged-host acceptance, public assets/checksums, Issue updates, OpenSpec archive and cleanup requirements of v0.22.42.
