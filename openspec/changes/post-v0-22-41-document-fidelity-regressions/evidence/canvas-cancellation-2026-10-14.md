# Cooperative Canvas cancellation

Current review comment 4264651691 identified a KatanA-owned gap: the fixed worker
discarded obsolete results, but synchronous JSON parsing kept later requests
waiting. This is not an upstream KDV or KRR publication wait.

The initial API candidate retained the old commercial parser call with only a
pre-parse cancellation guard. A real JSON regression failed with exit 101:
an unknown array was followed by invalid JSON, and parsing reached that invalid
tail instead of cancelling at the sixteenth checkpoint. The candidate was not
committed or presented as a working fix.

The repair keeps the existing `parse` API and `CanvasError`. A separate
`parse_cancellable` result uses `None` only for cancellation. Recursive serde
seeds check map keys, values and sequence elements without replacing JSON's
grammar. Parsing checks the trailing input; model decoding and reference
validation also check iteration boundaries. The worker calculates the same
SHA-256 in 64KiB chunks, preserves revisions on ordinary parse errors, and keeps
the original single worker, queue limit and automatic-refresh reservation.
Memory-provided sources now share the existing 256MiB file-input limit.

## Observed checks

- Final strict targeted run: core 28 and UI 68 tests passed, exit 0, with
  `--locked`, one build job, `-D warnings`, debug information disabled and the
  existing strip policy. Existing refresh, pan, generation and rendering tests
  remain included.
- A real pool accepted a maximum-size 256MiB memory source. After the request
  became active, dropping it allowed generation 2 to produce the expected next
  text model in 4,745 microseconds, within the unchanged five-second test bound.
  This proves active-request handoff, not that this cancellation happened inside
  JSON parsing. Separate core and worker callback regressions cover that phase.
- Final official AST: 23 passed. Impacted strict Clippy and formatting completed
  with exit 0. A preceding AST run rejected an unescaped Japanese test literal;
  replacing that literal with an equivalent Unicode escape repaired it without
  changing comment policy or weakening the gate.

## Diagnostic performance and limits

An optimized temporary diagnostic compiled the actual helper modules with
equivalent API entry wrappers and matching serde-core library fingerprints.
It is not the signed application or a packaged benchmark.

The rejected buffered-reader prototype took 85,367 microseconds for a 16MiB
plain string, versus 3,699 for slice parsing. The seed-based candidate retained
slice parsing. Five diagnostic rounds, large unknown compounds, keys and trailing
whitespace were recorded. Full 256MiB plain-token parsing took 28,897 microseconds
and escaped-token parsing 590,724, versus baseline 29,280 and 590,543 respectively.
An 8MiB unknown array took 78,543 versus 75,964 microseconds.

Single string/key/number scans, trailing-whitespace scans, serde flatten's
conversion of buffered unknown compounds, and destruction of partial models
are not forcibly interruptible. Checkpoints resume at the next value/stage
boundary. These sampled timings do not prove all maximum compound inputs,
all hardware, RSS acceptance, independent fidelity scores or packaged/all-OS
release acceptance. No additional parser threads or relaxed budgets were used.

Raw local evidence:

- `tmp/canvas-cancellation-mid-json-red-2026-10-14.log`
- `tmp/canvas-cancellation-core-ui-strict-final-2026-10-14.log`
- `tmp/canvas-cancel-seed-prototype-five-rounds-2026-10-14.log`
- `tmp/canvas-cancel-seed-maximum-token-2026-10-14.log`
- `tmp/canvas-cancel-seed-compound-token-2026-10-14.log`
- `tmp/cancellation-current-target-gates-repaired-2026-10-14.log`
# Source integration

Normal signed source commit: `efb4e0647a614abb7cff42e3ff4fcebcd4e17d8d`.
Final strict core/UI checks passed 28/68 tests (session 66351, actual exit 0).
Official AST (23), impacted strict Clippy and formatting checks exited 0
(session 68863). Remote review, full gates and packaged acceptance are separate
unfinished requirements.

