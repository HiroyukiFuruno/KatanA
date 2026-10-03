# Release-chain checkpoint: 2026-09-13

## Publication and admission

- Live sparse registry index: KUC 0.3.10, KDV 0.5.5, KRR 0.4.19; all latest entries have `yanked=false`. The crates.io metadata API returned HTTP 403; the sparse index was checked instead.
- GitHub Releases still expose KDV v0.5.5 and KRR v0.4.19. Required KDV v0.5.6 and KRR fixes are not admitted for downstream adoption.
- KDV PR #50 remains Draft at `3b4ecb13396c8f067e838cacb2a13087dd88309d`: three platform builds pass; preflight fails. These checks describe the pushed HEAD, not the owner's unpublished changes.
- KUC #52 remains open. Owner and consumer report legacy scores 89/95 and 91/95 on both 0.3.9 and 0.3.10. An explicit-baseline experiment scored 68/95 and 93/95 and was reverted. Neither publication nor shared geometry tests establish canonical acceptance.
- KUC owner confirms its assigned v0.3.10 release and cleanup are complete, but further #52 correction / next publication has not started and is outside that owner's stated version-scoped request. Do not silently infer authorization for another release.
- KRR PR #72 remains Draft at `8e70eaa2b39fb34adcc18df8693cdb9ff494a90e`: four platform builds and preflight pass; review latch fails. The owner confirms `activate --apply` was NOT started: two P1 and two P2 review threads remain unresolved and final review is incomplete. The user's activation approval was received, but does not waive those gates. No JWT/IAT issuance, activation, merge, or finalization occurred in that attempt.
- KRR release PR #77 remains Draft at `1b00937253a7a69eeb16c9f51eaf784710c86d0b` with four platform builds and preflight passing. This is not a merge/publication signal and does not supersede #72's admission gate.

## Independent KatanA verification

- Re-ran the existing diagnostic screenshot suite: 36 passed, 0 failed, 0 ignored. This includes logical-coordinate click and bottom-tab rectangle regressions.
- Command: `cargo test --locked --offline --manifest-path scripts/screenshot/Cargo.toml -j 2 --config profile.dev.debug=0 --config profile.dev.incremental=false` in `target/typography-host.tyoRkw/katana`.
- This diagnostic snapshot uses the archived candidate described in `xlsx-sheet-tab-input.md`; it is not published-graph or packaged acceptance evidence.
- Re-ran the real XLSX input scenario in that snapshot: exit 0, first frame 2.854 seconds, Notes and Dashboard fully inside the required bottom-rail bounds, and active sheet transitions 0 -> 1 -> 0. Output: `target/typography-host.tyoRkw/xlsx-sheet-tab-bounds-recheck`.
- Task 4.12 now records progress after completed UI updates, with writes throttled to 100 ms and repaint requests only when the heartbeat environment is configured. macOS/Linux and Windows smoke require monotonic progress throughout all ten RSS samples.
- Main-agent revalidation: focused `cargo test -p katana-ui startup_heartbeat --locked --offline -j 2` passed (2 tests), formal screenshot all-target Clippy with `-D warnings` passed, and `git diff --check` passed.
- `bash scripts/release/test-packaged-startup-contract.sh` passed: the same sourced monitor functions reject static frames, exhausted partial-read retries, and a stall after initial progress; architecture tests passed (7). The rejection messages are expected negative-test output. The production smoke has no test-mode success bypass.
- Main review also corrected Windows null content before regex matching so an empty or failed read reaches the bounded retry logic. PowerShell and real three-OS packaged execution remain unverified. These checks do not establish resolution of the user-facing freeze report or full release acceptance.

## Remaining boundary

Keep registry-only adoption, full unchanged quality gates, packaged clean-machine acceptance, formal PR review, actual release assets, and cleanup open. No sibling source edits or duplicate release branches were made by this monitoring turn.
