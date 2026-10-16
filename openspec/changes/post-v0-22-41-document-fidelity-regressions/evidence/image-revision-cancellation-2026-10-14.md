# Image path-revision cancellation

Tracked separately in [KatanA #348](https://github.com/HiroyukiFuruno/KatanA/issues/348).

The current-HEAD review comment 4264486718 identified a KatanA-owned cancellation
gap: a file invalidation changes its revision without changing the loader's
generation. The global image worker previously continued obsolete reads and
delayed later requests.

The new Unix regression connects a real FIFO writer only after the decoder has
opened its reader. It invalidates that path while keeping the generation fixed,
writes a PNG payload, and retains the writer until a separate real PNG becomes
ready. It uses the original five-second bound; neither sleeps nor early EOF are
used to make the old reader complete.

- Old implementation: the new regression failed with exit 101 and
  `follow-up PNG remained blocked behind stale FIFO decode`.
- Candidate implementation after responsibility separation: the root replay
  passed all 41 image-loader tests, exit 0, in 1.97 seconds.
- Cancellation now compares both generation and path revision before decoding
  and at existing read/decode checkpoints. A missing owner or poisoned revision
  lock cancels work. The single worker, queue and cache limits are unchanged.
- A blocked OS read itself is not forcibly interrupted. Cancellation takes
  effect after that read returns; this limitation is not a general guarantee
  for arbitrary blocked devices or codecs.

The direct diagnostic Cargo run emitted the existing linker unwind-table
warning. It is not evidence that the strict normal push gate, current review,
packaged acceptance, or all-OS release requirements have passed. Those remain
separate checks. No user application was operated or restarted.

Local raw evidence:

- RTK retained old failure: `1791931352_cargo_test.log`.
- Root candidate replay: `tmp/image-revision-cancellation-root-all-green-2026-10-14.log`.
- Initial official AST replay: `tmp/image-revision-cancellation-ast-2026-10-14.log`.

The AST replay predates the final responsibility separation, so it is not
reused as the final source's AST success.
# Final source verification

Normal signed source commit: `502a132cd15716d381a6174aac315e440b734005`.
Final strict image tests: 41 passed, actual exit 0 (session 62889).
Combined official AST (23), impacted strict Clippy and formatting checks exited
0 (session 68863). These are targeted source checks, not packaged or all-OS
release acceptance. Current-head remote review and publication remain pending.

