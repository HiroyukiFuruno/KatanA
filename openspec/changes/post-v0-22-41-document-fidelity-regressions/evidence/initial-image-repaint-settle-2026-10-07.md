# Initial image repaint measurement boundary

## Observed failure

- Public source graph: `9ca60afda5435b754b3941ce62eb7f535623d18e`.
- Official `JOBS=2 CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 just coverage`, session 48503, exited 101. UI results: 1156 passed, one failed, two existing ignored tests.
- Raw: `tmp/9ca60afd-current-coverage.log`.
- The failure occurred at `gui_tests.rs:39`, before the atomic overwrite: the initial egui repaint had not settled. Watch state was `revision=0, watched=true, pending=false, error=None, deferred=None, overflow=false`.
- The same instrumented test binary's focused execution passed once in 0.06 seconds. This does not replace the failed official gate or establish a stable product repair.

## Diagnosis and bounded correction

Initial texture availability is not proof that egui has consumed initial registration/decode repaint requests. Clearing `textures_delta` does not clear the repaint state. Existing font-lease tests consume pending repaint requests through at most eight actual egui frames before measuring a new idle wakeup.

The image GUI test now uses the same bounded frame-consumption pattern before installing its atomic-write wakeup callback. Only private test setup changes; production code, watcher behavior, image decoding, the five-second atomic-write budget, texture identity/replacement assertions, and existing ignores remain unchanged.

A deterministic unit boundary initially requested a repaint explicitly. With a no-op settling helper, session 75137 exited 101 with one assertion failure; raw `tmp/9ca60afd-initial-repaint-contract-red.log` and RTK tee `1791319785_cargo_test.log:154`. This is a test-precondition RED, not a newly demonstrated product bug. Session 60014 passed all 35 loader tests but failed one AST performance rule because the new test explicitly called unconditional `request_repaint()`; strict lint/format were not run. The final test instead asserts and consumes the egui context's real initial pending repaint, removing the redundant explicit request without adding an exemption. Final verification `tmp/initial-image-settle-targeted-final.log`, session 12981, exited 0: all 35 loader tests, AST 23, impacted strict Clippy and format checks passed. Main difference/caller review and `git diff --check` also passed.

## Self-review

- Private test-only difference; no public API or production implementation change.
- No fixed sleep, timeout extension, assertion removal, new ignore, lint suppression, or synthetic packaged acceptance.
- The original atomic-write idle wakeup and texture replacement remain the behavioral checks.
- Eight frames are bounded by a named constant and reuse an existing test convention.
- Genuine current-head Codex review on the prior public graph reported no major issues in comment 6025166353. It is not evidence for this later candidate.
- Conclusion: targeted self-review PASS. Private test-only correction integrated by normal hooked, signed commit `9ae652d4e005328cf93d201070bfd303a4290b02` (session 94252, exit 0, signature G). Normal push, new-head full gates and packaged acceptance remain unfinished.
