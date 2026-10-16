# Locked cleanup transaction validation — Issue335

Normal signed source integration: `819a52ef` (session47903 exit0). This
component is local-only until the next normal push; the previously published
`170d56af` does not include it.

## Scope

`release_cleanup_transaction.py` validates an already-admitted native branch
deletion during Git's `prepared` reference-transaction callback. It is not a
publication admission command, hook installer, or complete cleanup workflow.
No real repository hook, branch or worktree was changed by this work.

Git's [reference-transaction contract](https://git-scm.com/docs/githooks)
states that references are locked during `prepared`. The old value can be zero
even for an existing reference; the current reference value must then be read.
The earlier native transaction tests observed that case for `git branch -d`.

The validator requires one exact deletion record, an explicit release branch,
a full OID of the repository's actual width, and a nonsymbolic branch whose
current tip equals the audited OID. The CLI also binds the supplied root to the
hook CWD's common Git directory before evaluating the prepared record. It does
not install itself or override any existing hook.

## Actual verification

- session6429: native notification/rejection and prototype locked-tip checks,
  four tests passed, 2.698s. These were exploratory checks, not product RED.
- session61587: first runtime component checks, six passed, 4.436s.
- session97907: malformed/foreign/multiple/non-ASCII/oversized input and
  nonrelease/abbreviated OID negatives added; eight passed, 5.371s.
- session65639: moved-but-merged branch and symbolic-ref coverage; nine passed,
  5.819s.
- session44800: all cleanup tests passed35, 27.274s, before repository binding.
- Independent review identified a foreign-root acceptance path. An actual
  second temporary Git repository used the same release branch name but a
  different tip. Its hook was supplied the first repository's root/OID.
  session84213 exited1 with `CleanupRefused not raised`, demonstrating that
  deletion was incorrectly admitted before the fix. No mock transport or
  synthetic Git success was used.
- After the prepared CLI common-directory check, session43856 passed all10
  transaction tests, 7.258s. Both repositories' branches and the actual
  repository HEAD remain intact in the negative regression.
- Independent read-only re-review found no remaining P0/P1 for this limited
  component. Final session43563 passed all36 cleanup tests in 28.672s, exit0;
  raw output is `tmp/cleanup-transaction-final-tests-2026-10-15.log`.

## Remaining integration

Existing-hook-first delegation, private cleanup intent, public Release/tag
admission, default synchronization, merged/clean/unused retention, remote
deletion, scoped worktree cleanup/prune, and the mandatory release workflow
invocation are separate unfinished integration requirements. This component
alone does not establish a complete concurrency-safe cleanup transaction or
authorize real deletion. No new worktree, stash, force operation, hook bypass,
human app operation or sibling repository edit was used.
