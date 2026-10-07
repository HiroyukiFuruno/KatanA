# Office font resolution and ownership

## Scope

- KatanA release/v0.22.42 only; no sibling source edits, new worktree or stash.
- Grid requests are projected in the document worker. Local name-table face
  lookup runs in a separate thread, with surface and lookup generations.
- The UI receives without blocking. Pending reads are cancelled on replacement
  and close; unavailable/disconnected lookup releases its old lease.
- Context retains a manager without retaining Context itself. Each active pane
  leases selected payloads; the last close restores the current base definition.
- Normal cell paint performs no filesystem read or definitions reconstruction.
- Real installed Regular/Bold outline and painted-glyph evidence is distinct
  from lifecycle fixtures that label an existing Regular payload with styles.

## Verification

- Actual `just T=office_ test-specific`: 25 passed, zero failed.
- Actual AST suite: 23 passed. No lint exclusion or threshold was added.
- Actual document-surface suite with the declared real Office-worker wrapper:
  77 passed, zero failed. The earlier unwrapped invocation failed at the missing
  worker prerequisite, and two new unit tests initially failed to consume egui
  texture deltas; both errors are fixed and the complete suite was rerun.
- Strict locked katana-ui all-target Clippy passed.
- The unchanged full coverage gate is running, not yet accepted.

Follow-up: same-frame face registration reproduced a real egui unbound-family
panic. Painting now checks the current definitions before choosing a lease
alias, uses the existing fallback for that frame, and the actual glyph next
frame. Real installed Italic and both partial-style fallback cases are added;
the updated document-surface suite passes81 tests. Font definition transitions
request repaint only when a snapshot changes. Two callback tests initially
failed due to egui outstanding/delay throttling; bounded actual-frame settling
fixes them, and the14-test changed-state suite passes. Latest AST23 tests pass.
The extracted painter_grid_text_style file is added to the existing strict
100% gate; moving code out of its old file must not evade that gate.
Full coverage must be rerun after the latest fixes and review integration.

The next full run passed all test phases: UI960 tests with the two existing
ignored tests, core215, platform113, real export13, UI parallel143 with two
existing ignored tests, and serial18. The strict report rejected
painter_grid_text.rs at85/86 lines (98.837%): its small-cell text-area early
return at line29 was not executed. A real Context/Painter regression now
checks that a positive-text cell with an undersized rectangle produces no
Text shape and no panic; the focused test passes. Neither the strict100%
threshold nor coverage exclusions changed. The updated strict report and
full release gates are still pending.

After that regression and the MathJax stack repair, the normal full
`just coverage` run exits0: UI968 passed/two existing ignored, core215,
platform113, actual export13, UI parallel143/two existing ignored, serial18.
Meaningful uncovered lines are0 and every strict document-surface file is100%,
including the newly separated style file. Strict locked test-inclusive UI
Clippy, AST23 and formatting also pass. Source integration is commit7e4fb367;
normal push/current-HEAD cloud and packaged acceptance remain separate gates.

## Remaining acceptance

Source-renderer fidelity, installed Italic glyph proof, all-platform execution,
packaged Office input, memory/CPU ten-cycle observation, and final combined
release gates remain open. Chunk cancellation does not preempt a blocking OS
read; metadata-to-open races and unavailable fonts are not hidden as successful
fidelity. KRR #95 and KDV #58 remain separate public-release blockers.
