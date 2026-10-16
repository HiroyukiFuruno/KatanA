# Windows registry-cache and test timing observations

## Original requirement and remaining scope

Issue #343 requires measured Windows cache behavior and less redundant release verification without reducing quality. Registry-source cache restoration already exists; the Windows target cache remains disabled. The prior job did not expose its restore outcome together with the elapsed time and exit of the actual workspace test command.

This change adds diagnostic observations, not cache authorization or test elimination. It does not finish the Windows cold/warm comparison or all of Issue #343.

## Implementation

- The existing Windows registry-cache step receives an output identity. Its paths, pinned action, exact key and lack of target caching remain unchanged.
- Only Windows wraps the existing `bash scripts/ci/with-office-test-worker.sh 'cargo test --workspace'` invocation. The command runs once with inherited arguments, environment and output streams. Other platforms retain the original invocation.
- The observer records elapsed time, raw/normalized exit, actual cwd, declared CI source/run/OS/architecture and individual lockfile hashes. Lock hashes are explicitly labeled `after_command`, not frozen execution inputs. Missing metadata is explicit. It does not collect the full environment or grant trust to self-reported CI identifiers.
- A failed child remains failed even if observation recording also fails. A successful child with a recording failure returns 2. Spawn failures are not reported as successful executions. POSIX signal exits retain the raw value and use the conventional shell exit code.
- JSON is atomically saved under the run/attempt-specific inventory directory, already uploaded by the existing `always()` artifact step. The controlled step summary also includes source/run/lock metadata and its observation phase as HTML-escaped JSON; it excludes raw cache text.
- `diagnostic_only=true`, `reuse_authorized=false`, `cache_scope=cargo_registry_sources_only`, `build_cache_state=not_measured` and `cold_warm_equivalence=not_evaluated` prevent registry hits from being mislabeled as warm-build or execution-equivalence evidence.
- No test, coverage threshold, exclusion, worker quality check, cache target, Draft guard or required CI condition is removed or relaxed.

## Targeted verification

- New workflow contract: old workflow returned exit 1 with three failures; connected workflow returned exit 0 with three passes. Logs: `tmp/windows-ci-observation-contract-red-2026-10-16.log` and `tmp/windows-ci-observation-contract-green-2026-10-16.log`.
- Independent review identified missing summary identity, ambiguous post-command hashes and an overly broad atomicity-test description. Added regressions first failed in session 94513 (exit 1, two failures and one error); the corrected final related CI-resource, observer subprocess and workflow contracts passed in session 94406 (exit 0, 23 tests in 2.336 seconds). Logs: `tmp/windows-ci-observation-review-red-2026-10-16.log` and `tmp/windows-ci-observation-reviewed-targets-2026-10-16.log`.
- Actual temporary subprocess/file tests cover single execution, argv/environment/stdout/stderr passthrough, failure status, missing executables, signal status, missing/invalid lockfiles, the successful save path, failed-replace temporary-file cleanup, summary failures, unknown cache values and summary injection rejection. They do not prove concurrent-reader atomicity. No command-execution mocks are used.
- Default actionlint including ShellCheck: session 22470 returned exit 0. Log: `tmp/windows-ci-observation-actionlint-2026-10-16.log`.
- Python AST inspection: all three new Python files are below 200 lines and every function is at most 24 lines.

## Self-review and non-claims

The diff preserves the original command and quality gates; source metadata is diagnostic, not an admission receipt. The observer's own regression test isolates `GITHUB_STEP_SUMMARY` instead of appending fixture observations to a real CI summary. Hash assertions compare exact file contents, not only digest shape.

Real Windows runner execution, cold/warm matched runs, runner/total time comparisons, reuse counts and final release acceptance remain unverified. Those results must be collected after Ready without starting Draft CI or substituting macOS subprocess tests for Windows results. Post-command lock observations do not establish stable execution inputs. Independent-review corrections and their targeted results are recorded above; final re-review and normal integration/push are recorded separately when completed.

Final independent re-review found no remaining mandatory P0/P1/P2. The source/workflow and regression files were integrated by normal signed commit `ef29a17d5d3da49ea865063e079023cc52a477f5` (session 15586, exit 0, signature status G). Normal push and current-HEAD public review results are tracked in `tmp/document-fidelity-release-current.md` and the active task ledger; this record does not assert that those later steps or real Windows execution already succeeded.
