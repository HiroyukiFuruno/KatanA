# Narrow spreadsheet filter control — 2026-10-04

## Review contract

Current a91469ff review5405638326 raised P2 comment4177218504, threadPRRT_kwDORm09y86oxgyI. Header cells of22px or less returned no filter control, preventing users from changing or clearing existing criteria. This is a KatanA-owned interaction bug, not an upstream wait condition.

## Repair and verification

- Added a real egui input regression for column widths1,12,22,23px. Each click must dispatch exactly the correct Candidates command; the real menu then receives candidates and dispatches Clear for the same sheet/column.
- Before the production repair, the new test failed on the1px case with the missing-hit-target assertion (exit101). Log: `tmp/narrow-filter-old-red-2026-10-04.log`.
- Removed the width-based omission and bounded the button allocation by the actual full cell width. The original column-right anchor, clipped cell bounds and viewport intersection remain unchanged. Zero-area/offscreen controls still fail the existing visible-area checks.
- With the declared real Office-worker prerequisite supplied, all16 spreadsheet-filter tests pass (including real worker queue behavior and the offscreen-anchor regression), exit0/2.37s. Log: `tmp/narrow-filter-green-2026-10-04.log`.
- The initial test draft had a Clear-column type error and the first broad focused invocation omitted the worker environment; these failed invocations are not behavioral RED or successful verification. The corrected real entry above retains every test, without skip or mock.
- The first AST run rejected a305-line UI test file. Narrow-column interaction regressions were separated into their own cohesive module, retaining the shared real-menu harness; no lint allowance was added. Format, AST23 and impacted Clippy then passed. Added a zero-width no-hit-target regression; the resulting full17 filter tests passed at the final module layout, exit0/0.12s (`tmp/narrow-filter-final-green-2026-10-04.log`).

Self-review: the only production change is the allocation-width bound and removal of the arbitrary narrow-column omission. The original right-edge anchor and clipping guards remain; all existing filter state/queue/scroll tests are retained. The new interaction test uses actual RawInput and the real menu, verifies exact sheet/column command payloads, and introduces no snapshot, mock, additional delay, fallback or baseline change.

The prior full coverage success at7548542e predates this product repair. Changed-source full coverage, ordinary push hooks, current review reply/resolve/retrieval and packaged acceptance remain separate checkpoints. No release baseline, timeout, ignored test, coverage exclusion or upstream dependency changed.
