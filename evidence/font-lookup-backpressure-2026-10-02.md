# Bounded font lookup and backpressure

Current PR #346 review comment 4163002793 identified per-request thread creation
after cancellation: an old lookup can still own its payload during filesystem
I/O while a replacement starts. Cancellation alone does not interrupt OS I/O.

The candidate uses one active resolver and a bounded eight-entry pending queue.
Only cancelled pending work is discarded; independent document requests remain
FIFO. A worker unwind releases pending work and permits a later request to run;
unexpected failures continue through the existing typed disconnected/spawn path.

Independent review found a P1 in the initial candidate: queue-full `WouldBlock`
was permanently latched as a font failure. The revised lookup retains its lease
and retries transient backpressure through the existing UI poll, reserving a
25 ms repaint while capacity is unavailable. It retains no additional Context.
Changing requests, cancelling, and closing clear the retry state.

## Executed regressions before responsibility splitting

- Restoring the previous fail-on-`WouldBlock` branch made the actual bounded
  queue / DocumentFontLookup / installed Bold-face child regression fail
  (`tmp/font-backpressure-red.log`, exit 101).
- A negative control that started a new resolver while one was already active
  failed the lifecycle regression with three live workers instead of one
  (`tmp/font-worker-unbounded-negative-control.log`, exit 101). This is a
  concurrency negative control, not a claimed measurement of the old binary.
- Both temporary changes were restored with exact matching SHA-256 values:
  lookup `9e00c1d6d5a7dc2118d4cd40f22e36b551098968030ac1ec30af8d74cee53c97`,
  worker `fb42f30e414ccb8b22fa951f63fcc54bccb20647c363ea7a0de12aff7f84011f`.
- Ten focused tests passed, including real thread/channel barriers, cancelled
  payload drop, queue saturation, panic cleanup/recovery and installed font
  resolution. Child tests require a fresh receipt, exact test selection, bounded
  waiting and owned kill/wait cleanup (`tmp/font-shared-worker-final-green.log`).
- Strict all-target Clippy and format passed. The AST gate then rejected the
  scheduler/child helper file sizes and a test literal. The scheduler and retry
  child were split by responsibility, and leftover unused imports were removed.
- The final split source passes ten focused tests, AST 23 tests, strict
  all-target Clippy, format and diff checks. Full coverage completed with exit 0
  (`tmp/font-scheduler-full-coverage.log`): UI 1005, parallel integration 143 and
  serial integration 18 passed; strict document coverage is 100% with zero
  uncovered lines. Existing ignored tests and coverage exclusions are unchanged.
  Normal push and current review reply/resolve remain required. Frozen commercial source
  identities are recorded in `tmp/font-scheduler-final-source.sha256`.

This does not prove original Office RSS compliance, clean-machine packaging,
source-renderer fidelity or interruption of a blocked OS filesystem call.
Those product acceptance conditions and public release remain outstanding.

## Self-review

### No issues

- The changed lookup poll call passes the existing UI context; no public API,
  persisted settings, registry dependency or release acceptance limit changes.
- Queue saturation is transient, independent requests remain FIFO, cancelled
  payloads are dropped, and OS I/O retains its lifecycle until real completion.
- Tests assert worker counts, payload lifetime, receipts and actual font
  registration. No visual snapshots, new ignored tests or gate exclusions.
- Final focused tests, AST, strict all-target Clippy and format pass. The
  commercial source remains frozen while the full coverage gate runs.

### Conclusion

Targeted verification and full coverage: PASS. Normal push and current review
completion are pending; this is not a product acceptance or release PASS.
