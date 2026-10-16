# Immutable Git dependency input discovery

This slice discovers every Cargo.toml/Cargo.lock in the actual Git tree and
compares immutable regular-blob identities and contents. It does not yet infer
trusted workspace ownership, match semantic changes to Issue records, verify
publication, activate production admission, or complete Issue #335.

## Actual evidence

- Final root warning-as-error unittest discovery under all four Git hook
  environment variables: session 49532, exit 0, 136 tests, 20.731 seconds.
- Root AST inspection: all four candidate Python files are at most 200 lines;
  all functions are at most 30 lines. git diff --check exited 0.
- Worktree HEAD/tree before and after hook-context tests remained
  e43cefc29316d6379c24c0b7eba90a67fe6cf4ef /
  1000018d984e3785c711c7cb04551ef113ae050d; core.bare remained false.
  The primary master checkout remained clean.
- Live read of that public HEAD: session 91686, exit 0, 18 actual dependency
  inputs totaling 728393 bytes, including nested tool and vendor inputs.
- Actual temporary SHA-256 repositories cover dirty-worktree substitution,
  added/removed/changed nested inputs, ordinary-source-only changes, symlinks,
  oversized blobs, aggregate size and file count limits, and full commit OIDs.
- Actual malformed trees demonstrated tag peeling (697222, failed assertion)
  and duplicate-path ambiguity (1bbf4a, failed assertion). Explicit object-type
  checks and duplicate-path rejection now pass those same regressions.

## Bounded process lifecycle

Reader-thread candidates were rejected after reproducing double-counted limits,
premature timeout and EOF-before-process-exit failures. The final implementation
uses nonblocking pipe reads without reader threads, individual stdout/stderr
bounds, a shared deadline, and finally kill/wait/close for the owned process.
The real finite descendant fixture retains a pipe for four seconds; the timeout
regression requires bounded elapsed time and no additional reader threads.
Only fixture-owned descendant PIDs are cleaned up by the test.

The implementation relies on nonblocking pipe capability, available on Windows
with Python 3.12+, according to the [Python documentation](https://docs.python.org/3.14/library/os.html#os.set_blocking).
Unsupported capability is rejected; Windows execution has not been verified.
Tree collection shares one 30-second deadline across before/after snapshots,
with 4096 dependency files, 1 MiB per blob and 16 MiB per-tree content limits.
No user application, saved workspace JSON, or Terms state was changed.
