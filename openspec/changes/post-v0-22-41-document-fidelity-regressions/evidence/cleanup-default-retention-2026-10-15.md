# Issue 335: publication-guarded default synchronization and retention

This is the first cleanup implementation slice, not completed release cleanup.
No production switch, pull, branch deletion or worktree deletion was executed.

## Implemented contract

- Admit only an exact public stable GitHub Release and repository identity before
  synchronization. Inspect effective fetch and push destinations of both the
  source and explicit existing default controller, rejecting multiple URLs,
  mismatched repositories and URL rewrite configuration.
- Require the explicit controller to share the actual common Git directory, be
  on default, have no tracked/untracked changes, and have no repository stash.
  Do not force default out of another worktree or switch the release worktree.
- Use native switch and fast-forward-only pull without autostash. Confirm clean
  state and exact current remote default identity after synchronization.
- Re-audit only an explicit release branch and full expected commit. Retain
  checked-out or unmerged branches; reject protected/default and changed targets.
  An eligible audit result does not delete the branch or establish unused status
  for any worktree removal.
- Reuse bounded native Git output and deadlines. No hook bypass, force push,
  new worktree, or production stash was used. Existing-stash rejection is tested
  only in disposable native Git fixtures.

## Recovered evidence

- Prior ordinary push 31165 exited 0; local/origin/PR 346 match dc416d66 with
  ahead/behind 0/0. Complete review retrieval 37270 exited 0 and archived 148
  reviews; exact-HEAD formal reviews remain zero and the three unresolved
  manual-target policy threads remain unresolved.
- Native Git initial audit tests: 95037, eight tests, exit 0. After destination
  binding and bounded output: 36955, twelve tests, exit 0.
- Release Python discovery 2353: 240 tests in 84.617 seconds, exit 0, BEFORE
  the final rewrite rejection. This result is not final-source full acceptance.
- Two-stage insteadOf rewrite was demonstrated without network transport.
  Added regressions 38374 returned exit 1 with two expected assertion failures.
  Final targeted suite 46173: fourteen tests in 9.781 seconds, exit 0.
  Raw logs: tmp/cleanup335-native-tests-2026-10-15.log and
  tmp/cleanup335-url-rewrite-red-2026-10-15.log.
- Native read-only validation of the two existing production worktrees confirmed
  common Git identity, default controller, clean controller and stash zero.
- Real v0.22.42 lookup reports release not found. The publication entry refused;
  both worktree HEAD/status/all reflogs were unchanged (31623, exit 0).
  This is a negative admission check, not a published cleanup positive case.
- Final-source live negative check 24968 exited 0: effective fetch identity
  validated, unpublished Release refused, both HEAD/status/all reflogs unchanged.
- Independent read-only final review found no P0/P1 in these four files. It did
  not execute tests or approve deletion, production activation or full cleanup.
- All four added Python files parse, are below 200 lines, and every function is
  below 30 lines. Existing preflight discovery includes the new native tests.

## Remaining original DoD

Source concern was normally committed at
56cc53ca3334f3e53555e8cffa6425f0a3ee927e (95422, exit 0).
Documentation integration and normal upload remain separately verified steps.

Local/remote conditional deletion, active worktree/evidence preservation,
scoped prune, full genuine cleanup admission, ordinary-owner integration,
production dispatcher activation and the release workflow call remain open.
Do not invoke this slice as completed automated cleanup or relax release gates.
