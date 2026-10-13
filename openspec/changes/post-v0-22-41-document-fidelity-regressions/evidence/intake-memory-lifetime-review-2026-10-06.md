# Intake read/result ownership review

## Current finding

PR346 review5425916975 at published HEAD
`3c3485e12961462bc62f382921422138c6e7cd38` adds P2 thread
`PRRT_kwDORm09y86pYrwb`: the final Office/PDF close can request allocator
relief while the independent local-intake pool still owns a bounded read/result.
The pool's two persistent threads are not render/font workers; counting their
entire lifetime would prevent relief indefinitely and alter existing live-count
contracts.

Cancellation does not interrupt an OS file read. Removing an active-request
entry also does not prove that a successfully sent, buffered source was dropped.
The receiver can own that source after the worker returns.

## Scoped repair

- A separate intake lease uses the existing memory-state mutex and an intake
  ownership counter. Render/font worker counts remain unchanged.
- Local-source intake acquires its lease before canonicalization/read and shares
  it with cloned successful sources through `Arc`.
- `bytes` precedes the lease field: Rust drops struct fields in declaration
  order, so source-owned bytes are dropped before the lease. Error-path locals
  are dropped in reverse declaration order. See the official
  [Rust destructor contract](https://doc.rust-lang.org/reference/destructors.html).
- Empty UI descriptors retain neither bytes nor a lease. Moving bytes into a
  viewer occurs inside the existing tracked render-worker lifetime.
- Final-close relief requires both counters to be zero. Acquiring an intake
  lease does not cancel an already pending close request. Existing new-worker
  and nonempty-host reopening cancellation remain unchanged.
- No UI join, new allocator, public API, commercial test port, timeout change,
  or acceptance-threshold adjustment is introduced.

## Verification status

- `tmp/intake-memory-source-red.log`: actual exit0 but zero matching tests;
  rejected as reproduction evidence.
- `tmp/intake-memory-source-actual-red.log`: corrected official module filter,
  actual exit101; two tests executed and both failed at the intended
  `intake owns memory` assertion, not compilation.
- Candidate official GREEN: source ownership2, memory policy7, local intake11,
  worker lifecycle6, final-preview cleanup4. Actual exits0; logs are
  `tmp/intake-memory-{source,worker-memory,local-intake,worker-lifecycle-actual,action-cleanup}-green.log`.
  The mistaken lifecycle filename filter matched zero tests and is excluded.
- AST23 and strict impacted Clippy exit0. Initial fmt-check exit1 reported only
  two rustfmt differences; root corrected their formatting without changing
  assertions or file-length limits. Final `tmp/intake-memory-postformat-*.log`
  confirms fmt exit0, AST23 exit0, source2 and memory7 exit0.
- Source normal signed commit `7f923cf88ae68d534c86f0cf80cea65855fe87c6`
  completed with actual exit0 and a Good signature. It includes only the four
  scoped source/test files. Raw `tmp/intake-memory-source-normal-commit.log`.
  Normal push, individual review reply / resolve / fresh query and new-graph
  release gates are still pending.

The published 3c3485e1 coverage result is separate historical evidence:
official exit0, strict document coverage100% with zero uncovered lines. Its
success is not reused for this candidate. Current macOS CI instead failed the
dirty-target HTML first-frame deadline (2026ms, 1120passed / 1failed / 2ignored).
The workflow's later diagnostic pass does not replace that original failure.

The failure artifact11401165243 was extracted locally. PID25261's real
symbolicated sample (`tmp/3c3485e1-live-snapshot/html-startup-25261.sample.txt`)
records the failed test inside its post-timeout diagnostic sampler. A second
thread constructs preview panes and deserializes bundled Syntect syntax data;
the existing global render-test guard is not held by that state-only test.
The sample was taken after the deadline: it establishes concurrent initialization
but does not establish the cause or the earlier runtime/host wait stage. The
other extracted PID27132 is a diagnostic test's `/bin/sleep` child, not a
successful KatanA process comparison. No timeout relaxation, test suppression,
or unproven constructor/fixture change is justified by these observations alone.

## Self-review

PASS for this scoped candidate: the same mutex orders intake acquisition,
last-owner drop and zero-counter relief. Both worker-first and intake-first
release orders are tested. Source clone and actual channel queue ownership are
tested without mocks; an empty descriptor does not extend retention. Existing
read errors/cancellation/FIFO regressions execute through local intake. The
production viewer handoff remains inside the pre-existing tracked render worker.
No serde/public-crate contract is changed and no gate is suppressed. Comments
follow the latest human Japanese-WHY instruction.

This is not a stable RSS improvement claim, a packaged/human acceptance result,
or proof that the independent HTML CI timeout has been repaired.
