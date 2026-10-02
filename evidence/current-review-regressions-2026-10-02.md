# Current review regressions (2026-10-02)

## Scope

PR #346, review of HEAD `8460bd91`: comments `4163417357`, `4163417363`, and `4163417371`.

- Font requests include same-family fallback styles, deduplicated without a per-cell vector allocation. Painting retains exact, bold, italic, then regular priority and synthesizes only missing styles.
- Office/HTML reject Split/CodePreview through both command availability and direct action dispatch. PDF/Markdown retain their existing behavior.
- Slideshow modal entry rechecks the active document after global tab navigation and exits unsupported documents, restoring fullscreen only when slideshow entered fullscreen itself.

## Actual negative controls

- `tmp/review-fallback-red.log`: old request projection failed the styled-family regular-face regression (exit 101).
- `tmp/review-regular-paint-red.log`: old painting selected `Proportional` instead of the real leased Arial regular face (exit 101).
- `tmp/review-view-tools-red.log`: old registered availability and direct dispatch failed; existing PDF/Markdown behavior passed (exit 101).
- `tmp/review-slideshow-red.log`: old modal retained slideshow after next-tab navigation to HTML (exit 101). The test covers next/previous navigation, HTML/DOCX/XLSX/PPTX, and both pre-existing fullscreen states.

## Verification

- Actual font-lease paint tests: the original 4 passed; 2 added mixed-style tests also passed, including real bold/italic galley family, faux styles, and tessellated meshes.
- Command/direct-dispatch tests: 4 passed.
- Tab/slideshow viewport-command regression: passed.
- Complete UI library suite with the standard real Office worker launcher: 1,015 passed, 2 pre-existing ignored; exit 0 (`tmp/review-three-regressions-ui-worker.log`).
- An earlier direct Cargo invocation failed 2 real XLSX tests because no Office worker path was supplied; this was an invocation failure, not waived. The standard launcher built the real worker and the complete rerun passed.
- Strict locked workspace/all-target Clippy: exit 0. Format check: exit 0. AST: 23 passed. Diff check: exit 0.
- First changed-source receipt `tmp/review-three-regressions-source.sha256` matched after the first full coverage run: exit 0, strict 100%, uncovered 0, meaningful gate passed. After the independent-review test enhancements, final full coverage also exited 0: UI 1,017 passed / 2 pre-existing ignored, parallel integration 143 passed / 2 pre-existing ignored, serial integration 18 passed, strict 100%, uncovered 0, meaningful gate passed. All 9 hashes in `tmp/review-three-regressions-final-source.sha256` matched after completion. Final log: `tmp/review-three-regressions-final-coverage.log`.
- Changed Markdown check through the existing `KML_CHECK_ARGS` target passed. The optional no-argument repository-wide KML diagnostic failed on existing vendored Markdown examples (for example undefined reference labels); no rule, exclusion, vendor fixture, or release gate was changed. Logs: `tmp/review-three-regressions-{kml,changed-kml}.log`.

## Self-review

No public API, serialized settings, source-I/O deadline, coverage exclusion, test disablement, or acceptance threshold was changed. Dedicated command test modules keep commercial files within the existing size limit. Main review checked fallback style priority and fullscreen ownership. Independent read-only review found no P0/P1 implementation defect. Its P2 suggestions for mixed-style priority and explicit active-document/restart assertions were implemented and passed focused tests; re-review found no new P0/P1/P2 defect. Its note about shared menu availability is intentional existing behavior: the user explicitly requested the common Office/HTML controls to be inactive; this change does not introduce a new per-menu policy.

## Not completed by these fixes

Normal-hook commits: `77ea5d26` (font fallback), `2b55ae76` (display shortcuts), `183cd30c` (slideshow navigation). Normal push and individual review reply/resolve are still pending at this checkpoint.

The original Office cold RSS result remains a failure (317,328 KiB increase, limit 196,608 KiB). Original HTML normal close still failed the 5-second condition. Packaged/clean-machine fidelity and memory acceptance remain unmet. KDV #59 and KRR #95 remain open; their required published fixes are not yet available. PR remains Draft, with manual target-publication policy requiring a user decision. The successful 3-OS CI for `8460bd91` does not validate these new changes or replace real-file acceptance.

KUC `0.4.1` is now public (GitHub Release and non-yanked sparse registry entry verified). KDV `0.5.8` pins KUC `=0.4.0`; KatanA has no direct KUC dependency. The existing KDV owner was asked to adopt the new KUC in its next published fix; no sibling repository was edited.

## Push and review reconciliation

Normal push completed with exit 0; PR #346 and the remote branch now point to `2910029bbdb055b3a312023004794050f6ec5294`, ahead/behind 0/0. Native, Linux workspace tests, Windows cross-check and PR readiness ran through the existing pre-push hook, without bypass. Log: `tmp/review-three-regressions-normal-push.log`. All 9 source hashes still match the final coverage receipt.

Individual replies: `4164324356` (font), `4164324599` (commands), `4164324839` (slideshow). Each corresponding thread was resolved, then all 23 threads were freshly queried (no next page); these 3 are resolved. Only existing target-publication policy comment `4155352623` remains unresolved, pending the user decision. PR is still Draft. This reconciliation does not waive the performance, packaged acceptance, upstream publication, current cloud review/CI, or release requirements above.

## Git fixture isolation regression

The post-push audit found shared `core.bare=true` again. The acceptance-gate fixture inherited the hook's `GIT_DIR`; its real `git init` operated on caller metadata instead of the temporary fixture. A temporary caller with separate Git metadata reproduced the old failure without creating a worktree or touching real repository metadata: bare-mode source enumeration failure, missing fixture metadata, and caller configuration changes. Inherited `GIT_DIR` alone and combined `GIT_WORK_TREE`/`GIT_COMMON_DIR` are covered.

Fixture initialization and source enumeration now use an environment excluding Git's native `--local-env-vars` and indexed config overrides. Main reran the complete gate suite (16 passed) and evidence suite (27 passed), preserving PATH and non-Git environment. The regression verifies exact caller config bytes, unchanged status, independent fixture metadata and fixture-local source enumeration. This is a harness repair, not an Office/HTML performance fix. Normal hook and actual shared-config invariance still require verification at this checkpoint.
