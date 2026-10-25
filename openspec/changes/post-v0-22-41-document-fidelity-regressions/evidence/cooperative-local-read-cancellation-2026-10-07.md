# Cooperative local read cancellation

## Review and scope

- Baseline: public `502933b9e2daeca1fac54f420069562d4db68490`.
- Current review `5435480274`: image comment `4201258867` / thread `PRRT_kwDORm09y86psGt3`; document comment `4201258873` / thread `PRRT_kwDORm09y86psGt-`.
- Both are KatanA-owned cancellation issues, not deferred KDV/KRR rendering issues.
- Preserve the one image worker, two document workers, bounded queues, document size cap, rendering implementation, and original acceptance thresholds.
- An OS syscall that has not returned, or an internal codec/rasterizer call, is not immediately interruptible. Cooperative cancellation stops further work at read/phase boundaries; it is not a hard-interruption guarantee.

## Baseline runtime failures

- Document session `20048`, actual exit `101`: the real FIFO child asserted `cancelled FIFO reads kept both workers until writer EOF`. Log: `tmp/cancelled-document-read-runtime-red.log`.
- Image session `24556`, actual exit `101`: the real loader/FIFO child asserted `follow-up PNG remained blocked behind stale FIFO decode`. Log: `tmp/cancelled-image-read-actual-runtime-red.log`.
- Both use private real-file fixtures and actual pool/loader paths, keep a persistent writer open, and retain the original five-second assertion window. Child isolation prevents contamination of the parent process's global pools.
- Image session `66488` was a compilation failure from wiring an unused helper before the production callers existed. It is not runtime RED. The earlier unsafe image fixture was rejected before execution because its writer handshake and cleanup could deadlock.

## Candidate status

The shared private reader checks cancellation before and after bounded reads and uses a non-retrying I/O error. Document intake retains the original bounded reader, size sentinel, allocation/error handling, and memory lease. Image intake checks Weak owner/generation without retaining the owner through codec work and closes the source file before decoding.

- Focused document GREEN `13897`: actual exit `0`.
- Image suite `13976`: actual exit `0`, 40 tests passed.
- Final aggregate `83369`: real-Office-worker document suite passed 146 tests, image suite passed 40, shared real-file reader passed 3. AST stopped the aggregate at 22 PASS / 1 FAIL: three comment-style violations and two unnamed fixture color components. Those violations were corrected without disabling rules; corrected static aggregate `38438` is running.
- The initial broad document run `20698` failed two worker-dependent tests because no real Office worker path was set. The followup `92698` stopped at compilation because obsolete synchronous wrappers were unused in a normal build. Neither is accepted as GREEN. The final run uses the existing `with-office-test-worker.sh` contract. Existing synchronous convenience functions are now private test-only wrappers; tests still execute the actual production cooperative source path.

- Corrected aggregate `38438`: actual exit `0`; formal AST 23 PASS, strict all-targets UI Clippy and format check passed. Full AST/token target `34530`: actual exit `0`, 31 PASS including the eight Japanese-comment/token regressions.
- Signed normal concern-separated commits: document/shared reader `1ca93980d13af724c486a4b2c182a4c3310bb187` (`61556`, actual exit `0`); image `699ee58fc792e47091a6164acf8c56b538cd35fc` (`21434`, actual exit `0`). Both have local signature status `G`.

Formal integration, current full gates, review-thread handling, and release remain pending. No packaged-host acceptance, independent quality score, human app operation, or Terms approval is claimed by these fixtures.
