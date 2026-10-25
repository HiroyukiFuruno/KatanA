# Fullscreen close and shared image watch ownership

## Current review

- PR346, reviewed commit `5bebef75a9f9bb285c4250532d1e449f202450c9`.
- P2 thread `PRRT_kwDORm09y86pqT_j`, comment `4200513146`.
- Closing fullscreen reset the shared local image loader while regular viewer textures remained alive. Those textures prevent a fresh request, leaving external overwrites unobserved.

## Reproduction

- Added `fullscreen_close_preserves_regular_image_watch_and_reload` under the private local image loader GUI tests.
- The fixture decodes a real PNG through the regular UI entry, closes fullscreen through `apply_fullscreen_result`, then requires unchanged shared generation/regular texture ownership and an actual on-disk overwrite to replace the regular texture.
- Original product: session3719 actual exit101, zero passed/one assertion failure in 0.05 seconds. Shared generation changed from0 to1 at close, before the overwrite stage.
- Raw: `tmp/fullscreen-image-watch-red.log`; expanded assertion: RTK tee `1791321646_cargo_test.log:154-157`.

## Scoped repair

- Removed only the shared loader reset from fullscreen close. Existing fullscreen viewer reset still releases its texture handle and restores its view state.
- Fullscreen decode already releases its temporary active image. Shared bounded caches and registrations remain available to the regular preview; document rerender/reset retains its existing full loader cleanup.
- No new loader, public API, watch ownership abstraction, allocator, threshold, test ignore, timeout extension, or synchronous UI file read.
- Actual overwrite budget remains five seconds, with event-driven polling/yield rather than a fixed sleep.

## Verification boundary

- Source repair is normally committed with hooks and valid GPG signature: `334bcadf2f4a90491bbd9592c16fc36eff32c536`, session99439 actual exit0. Commit time21:23:39UTC /06:23:39JST.

- Session43629 passed all36 loader regressions, but exited101 at AST (22 passed/one failed): the new helper's pixel literals violated the existing magic-number rule. Strict Clippy/format were not reached. Raw: `tmp/fullscreen-image-watch-green.log`; RTK tee `1791321707_cargo_test.log:30-33`.
- Extracted the two fixture pixels to named constants without an exemption. Final session85443 actual exit0: all36 loader regressions (1.76 seconds), all23 AST contracts, strict impacted Clippy and format checks passed. Raw: `tmp/fullscreen-image-watch-final.log`.
- Current5beb official coverage7835 passed and independently exported/checked, but is not evidence for this later product repair.
- Normal integration/push, individual reply/resolve/fresh query, new graph full gates, packaged host acceptance and release/postprocessing remain unfinished.
- Human app and Terms have not been operated.
