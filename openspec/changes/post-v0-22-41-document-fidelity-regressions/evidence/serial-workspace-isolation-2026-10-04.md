# Serial integration workspace isolation

## Failure and causal control

- Public KDV 0.5.11 dependency graph at `f79b9e5d` reached the unchanged full coverage serial suite, then failed: 17 passed, one Explorer alignment failure, exit 101. The same binary's isolated alignment test passed.
- Adding an explicit load-completion assertion showed that the workspace was still loading, rather than an incorrect 2 px alignment or translated label.
- The three preceding i18n tests use a persistent settings repository, but their app initialization previously retained the default global workspace repository. `KatanaApp::new` can therefore restore real user workspace histories before the test's own workspace action. Background scans use the process-global Rayon pool.
- Pairwise diagnostic runs reproduced failure after language roundtrip or all-languages alone; the multiple-changes test, which explicitly opens a small workspace and cancels the restored scan, passed with the alignment test. These filtered runs are diagnostic only, not the acceptance gate.
- Isolating only the tree fixture did not solve the complete suite: `tmp/kdv0511-serial-initial-state-2026-10-04.log`, 17 passed / one failed, exit 101.
- With a real JSON global workspace repository beside each i18n test's settings file, all 18 serial tests passed in 3.22 s: `tmp/kdv0511-serial-global-workspace-isolation-2026-10-04.log`, exit 0.
- Negative control removed only that i18n repository assignment while retaining the tree isolation and unique settings directory. All 18 ran again and returned 17 passed / one failed in 3.91 s, exit 101: `tmp/kdv0511-serial-global-workspace-negative-control-2026-10-04.log`. The isolation was then restored.

## Scope and verification integrity

Only integration fixture setup changes. Production code, real filesystem scanning, all 18 test registrations, the existing 100-step polling limit, 2 ms scheduling pause, and 2 px alignment criterion are unchanged. No mock, skip, ignore, timeout increase, snapshot replacement, coverage exemption, or assertion weakening was introduced. The tree test additionally checks successful load, exact workspace root and absence of an unrelated release-note popup.

The full unchanged `just check-full` rerun with two jobs is logged to `tmp/kdv0511-full-local-gate-isolated-2026-10-04.log`. It exited101 at the source-comment AST check described below. Neither unsuccessful full command is relabelled as a pass. Final repaired-source targeted checks succeed: all18 serial integrations in3.41s (`tmp/kdv0511-serial-isolated-final-2026-10-04.log`), strict test-inclusive Clippy (`tmp/kdv0511-isolation-test-clippy-2026-10-04.log`), AST23 (`tmp/kdv0511-isolation-ast-2026-10-04.log`) and diff whitespace validation, all exit0. Full coverage/platform validation, normal-hook commit/push, current review, packaged acceptance, independent fidelity and publication remain separate pending checkpoints.

## Self-review

PASS for the scoped fixture repair: real JSON repositories and filesystem input are retained, and test isolation prevents restoring user workspace history during app initialization. The first full rerun passed export13 in343.77s, then the unchanged AST suite rejected a newly added Japanese source comment. The redundant comment was removed; the reason is documented here, without altering the linter. All targeted final-source checks above pass. No public API or application behavior changes. Full-gate results and normal-hook commit/push remain pending.
