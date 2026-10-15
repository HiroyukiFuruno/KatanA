# Cleanup owner delegation and private intent

## Scope

Issue #335 component only. No production hook installation, repository branch
deletion, default synchronization, remote deletion, or prune was performed.
The existing release worktree and registry dependency versions remain unchanged.

## Implemented contract

- A private directory (0700) and regular record (0600) must belong to the current
  user. Directory-relative native opens reject final symlinks; unsupported native
  protection fails closed. Creation is exclusive and cannot overwrite a record.
- The bounded record binds an exact schema, random nonce, canonical repository,
  common Git directory, explicit release branch, full OID, and caller-held SHA256.
  Duplicate fields and unknown fields are rejected. This is not public-release
  authorization and does not establish a replay-resistant lifecycle by itself.
- The reference dispatcher executes the existing owner first with its original
  phase and complete stdin. Only the additional guard's copy is size-bounded.
  Relative owner paths are made absolute before both validation and execution.
  Ordinary exit codes and signal termination are propagated.
- After successful owner execution, disabled Lefthook ownership is refused.
  Prepared transactions validate the private record, actual common repository,
  and existing locked expected-tip guard. Native tests retain mismatching refs.

## Verification

- Source normally committed as `bbe3cebe0718b9cb92a1578a90652f1aa0a17085`,
  commit session 52072 exit 0, signature G. Not yet public at this record.
- Private intent tests: session 38688, exit 0, 7 tests, 2.874 seconds.
- Independent review found two owner-delegation defects. Native executable
  regressions failed before repair: oversized input bypassed the owner and a
  relative basename selected a different PATH executable (two assertion failures).
- Streaming/path repair: session 58113, exit 0, 18 tests, 11.177 seconds.
  Additional full-input/early-exit tests: 61374, exit 0, 4 tests, 0.958 seconds.
- Whole release Python discovery: 6538, exit 0, 284 tests, 115.712 seconds.
  Raw log: `tmp/cleanup-owner-intent-final-tests-2026-10-15.log`.
  This preceded the final signal-propagation repair; it is not final-HEAD proof.
- Signal regression: 44138, exit 1; actual SIGTERM became exit 241 instead of
  signal -15. Native signal propagation was then added.
- Final changed-component discovery: 77050, exit 0, 21 tests, 12.209 seconds.
  Command: `python3 -B -W error -m unittest discover -s scripts/release -p 'test_cleanup_*.py' -v`.
- Main AST validation: seven Python files parse, every file <=200 lines and
  every function <=30 lines (maximum 21). `git diff --check` succeeded.
- Fresh independent read-only review found no remaining component P0/P1 or
  mandatory P2. Final target result 77050 was separately recovered by main.
- Test fixture corrections (independent invalid-field mutations, explicit
  signing-off configuration, and a real target branch) are not product REDs.

## Remaining integration

Public-release admission must create and retire the private operation record.
Production existing-hook preservation/activation, audited local and remote
deletion/retention, scoped worktree pruning, and mandatory release workflow
invocation remain unfinished. Existing-worktree runtime parity is required
before shared hook activation; no master edit or fixed-release-path fallback is
permitted. No Issue closure, release publication, full native OS acceptance,
coverage equivalence, or packaged acceptance follows from these component tests.
