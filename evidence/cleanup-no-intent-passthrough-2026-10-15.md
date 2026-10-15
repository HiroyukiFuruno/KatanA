# Cleanup reference passthrough before production wiring

## Implemented scope

No-intent reference transactions delegate only to the existing owner. Cleanup
input bounds and additional Lefthook policy do not affect ordinary Git operations.
Both explicit CLI input and scoped environment input are supported; a partial
path/digest pair is refused after preserving an earlier owner failure.

Only `KATANA_CLEANUP_INTENT` and `KATANA_CLEANUP_INTENT_SHA256` are removed from
the owner's environment. Other Git hook environment is preserved. A valid
environment-bound operation is still checked by the private record and locked
expected-tip guard.

## Evidence

- Source normally committed as `6c586811352cb7b8e787390594a0277c6a235830`,
  session 62648 exit 0, signature G. This concern is not yet public.
- Before the production change, corrected-fixture session 64689 exited 1 with
  seven assertion failures: mandatory CLI arguments prevented owner delegation
  and a native commit. No optional-input exception was assumed without a RED.
- Final changed-component discovery 41707 exited 0: 30 tests, 19.177 seconds.
  Tests use real Git repositories, real executables, and real private files.
- Final cleanup discovery 94024 exited 0: 66 tests, 48.221 seconds. Raw log:
  `tmp/cleanup-passthrough-final-tests-2026-10-15.log`. Command:
  `python3 -B -W error -m unittest discover -s scripts/release -p 'test*cleanup*.py'`.
- Native commit and fast-forward pull succeed without an intent. A native
  `branch -d` with a valid scoped environment intent succeeds while the owner
  observes both dedicated environment values as absent.
- Native Git may also emit `AUTO_MERGE` transactions after committing a branch.
  Sessions 32116 and 49223 exposed an invalid fixture assumption that the final
  callback always names master. The fixture now records every transaction and
  verifies the actual master update; this was not a product RED.
- Fresh independent read-only review found no component P0/P1 or mandatory P2.
- Three changed Python files parse and satisfy file <=200/function <=30 limits.
  Lengths are 63/68/153 lines, maximum function span 17.

## Not completed by this change

The shared hook is not installed. Public admission/caller lifetime, operation
serialization, safe local and remote deletion/retention, scoped prune, and release
workflow invocation remain separate unfinished Issue #335 requirements. No user
application, settings JSON, default worktree, or sibling repository was modified.
Source publication and current-HEAD formal review must be checked separately.
