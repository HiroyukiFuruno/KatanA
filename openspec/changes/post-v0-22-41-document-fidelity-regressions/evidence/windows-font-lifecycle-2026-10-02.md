# Windows font I/O and worker lifetime

## Current public failure

KatanA PR346 at `f13756a8` failed the real Windows cold lookup step in job
110542526364. The request uses installed Arial Bold; the original ten-second
deadline and actual-font selection assertions remain unchanged.

Candidate discovery took 4033 microseconds for 147 candidates. Reading
YuGothB.ttc took 6203080 microseconds for 14729372 bytes; YuGothL.ttc took
2193690 microseconds for 13969876 bytes. Metadata extraction took only tens
of microseconds per face. The timeout occurred before any Arial payload appeared.

The scanner uses case-sensitive filename stems: uppercase Candara through
YuGoth precede lowercase arial/arialbd. Thus unrelated full font-file reads
preceded the requested exact face. This is avoidable host logic, not evidence
of slow JavaScript or an unavoidable metadata parser delay.

## Scoped ordering repair

Only Windows uses ASCII case-folded ordering. The original filename breaks
case-fold ties, stable sorting preserves equal-name directory priority, and
exact-name deduplication remains unchanged. Other platforms retain their former
order. No candidate is skipped, no weight/style matching changes, and the
ten-second deadline is not extended. Equal-distance resolver ties still retain
the first candidate; a metadata-only fixture explicitly verifies both orders.

The old order failed with Candara/YuGothB before arial/arialbd (exit101).
The repaired scanner tests passed, followed by the complete platform suite.
The isolated macOS result is not a replacement for the next real Windows CI.

## Font-thread lifecycle repair

Review comment4159589365 found font lookup threads bypassed the common document
worker lease. A cancelled lookup could still own payloads or block in filesystem
I/O while the harness observed zero document workers.

The product start path now delegates its named spawn to the common lease.
A single-test child process uses a real thread and channel barriers: cancellation
does not release the count, owned payload Drop still observes one worker, and
join observes zero. The old raw spawn returned zero at the first observation
and failed with exit101. This is an ownership test, not a mock font resolver;
the actual-font, UI-paint and generation-result tests are also retained.

Independent review identified unbounded test waits. All channel waits now have
a five-second limit, and a parent-owned child has a monotonic five-second limit
plus kill/wait cleanup. A fresh receipt is written only after all child checks
and final zero count; a wrong selector or zero-test child cannot pass the parent.
These are test supervision limits, not changes to product acceptance deadlines.

Focused tests passed: UI lib68, main10 and parallel integration11. Final strict
all-target workspace Clippy and format checks passed. Independent re-review
found no remaining concrete P0/P1/P2 in these repairs. The final AST gate passed
all 23 tests. Complete normal `just coverage` finished with exit0: UI998,
platform117, core215, actual export13, parallel143 and serial18 tests passed.
Strict document-surface coverage remained100% with
zero uncovered lines, and the meaningful-line gate passed. All four source
hashes match the pre-run manifest after completion. Existing ignored tests,
coverage scope and thresholds were not changed.

## Self-review

### No issues

- The diff changes Windows discovery priority and font-worker accounting only;
  all candidates, actual face selection, generation checks and deadlines remain.
- Regression tests reproduce the former ordering and lifecycle failures. Stable
  duplicate priority and equal-distance face selection have explicit assertions.
- The isolated lifecycle child has owned-process cleanup, bounded supervision
  and a mandatory fresh completion receipt. No test is ignored or mocked.
- Final strict all-target Clippy, formatting and AST gates pass. No lint waiver,
  coverage exclusion, timeout relaxation or acceptance-budget change was added.

### Findings and conclusion

Local code review found no remaining concrete repair finding. Final static and
complete coverage gates passed on the unchanged source, clearing these repairs
for formal integration. Actual Windows CI, review-thread closure and real-input
memory acceptance remain separate gates.

## Limits and retained evidence

### Actual Windows follow-up

GitHub CI run36924444367 at `e69bdfa6046e76d2d8b3ddf479b66d08503fd532`,
job110578178671, reports success for the cold actual-font step and normal tests.
The workflow runs the exact actual Arial Bold/UI-paint regression with the
original ten-second bound; it does not substitute metadata-only fixtures.
macOS and Ubuntu full jobs, all three lint jobs, supply-chain and CodeQL also
report success. Windows headless acceptance and coverage are still running.
This closes the observed cold lookup failure, not the complete Windows release
gate. Raw timing logs will be retrieved after the job finishes.

The lifecycle review thread was replied to at4160357514, resolved, and re-read
with all comments and pages checked. Only the manual-target publication-policy
thread remained unresolved at this check; PR346 remains Draft.

The latest source was re-run against the original five-Office cold budget after
release runner rebuild: delta317328KiB above196608KiB, exit1. Owned counters and
five document frames are recorded separately in the memory evidence; resource
accounting improvement is not a claimed RSS fix.

The lifecycle counter does not prove allocator/TLS/GPU release or interrupt OS
syscalls. The supplied unique-Office RSS budget still fails; do not declare the
whole memory regression fixed. Actual Windows, current review, public packaging
and published upstream HTML acceptance remain separate release gates.

Ignored logs include `tmp/windows-font-phase-job-110542526364-raw-2026-10-02.log`,
`tmp/windows-font-order-red-resumed-2026-10-02.log`,
`tmp/windows-font-order-platform-full-2026-10-02.log`,
`tmp/font-worker-lifecycle-barrier-red-2026-10-02.log`,
`tmp/font-order-lifecycle-focused-final-2026-10-02.log` and
`tmp/font-order-lifecycle-strict-clippy-final-2026-10-02.log`.
