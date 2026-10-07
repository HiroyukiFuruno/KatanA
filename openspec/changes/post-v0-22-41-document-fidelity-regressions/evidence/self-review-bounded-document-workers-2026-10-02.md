# Self-review: bounded document intake and filter candidate queue

## Scope

Two independent fixes address PR #346 comments 4155543091 and 4156681273.
The public dependency graph remains unchanged and uses registry releases.

## Verified

- Intake has one singleton, two workers and 64 weak queued requests. Dropping
  a handle cancels delivery; obsolete queued requests release paths and senders.
- No filesystem read is performed while holding the pool queue lock. Worker
  startup failure and unwind leave a terminal failed pool without replacements.
- Successful delivery and pool state use the same lock; UI polling rejects a
  terminal failure even when another worker remains blocked in an OS read.
- The command queue retains 16 ordinary commands plus the latest candidate
  request. Overflow does not evict that candidate or reorder surviving commands.
- Actual FIFO regression failed with 15 threads from baseline 3, then passed
  with 5. Queue saturation, cancellation, weak release, UI painting and terminal
  failure are tested. Existing intake assertions and ignored tests are intact.
- Filter queue five tests, actual XLSX saturated worker two tests and existing
  bounded queue two tests passed. Strict test-inclusive Clippy and AST 23 passed.
- No lint suppression, coverage exclusion, deadline relaxation, synchronous
  fallback, extra workers or new worktrees were introduced.

## Remaining verification

Full coverage completed with exit0 against the frozen changes: UI 978 passed
(2 existing ignored), core 215, platform 113, export 13, UI parallel 143
(2 existing ignored), serial 18. Meaningful uncovered lines were zero and the
existing strict document surface contract remained 100%, without exclusions
or threshold changes. Current Windows CI found
a separate real-font worker timeout; its cause is still under investigation.
Normal commit/push, fresh review reply/resolve, all-platform gates and packaged
acceptance remain necessary. This review does not certify public release.

## Limits

OS reads cannot be cancelled; the fix bounds worker accumulation, not syscall
duration. Real partial OS thread-spawn failure has not been induced. The small
in-process resource-cycle evidence predates these changes and must be repeated
using rebuilt binaries.

## Conclusion

Scoped source review passes. Final release gates remain incomplete.
