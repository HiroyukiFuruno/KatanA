# Workspace URL generation review repair

## Scope

Current review 5433007691 on public commit 068f381ef6c0599b02164e51f1ffea12b3e36d87 identified thread PRRT_kwDORm09y86pm6yc, discussion 4199097415. A successful workspace replacement can restore the same local HTML path while retaining an HTTP request from the previous document generation.

The repair cancels document-targeted requests only when a successful workspace Open replaces the document set. Untargeted new-tab requests survive, and the loading flag is recalculated from the remaining queue. Refresh, failed Open and an unfinished Open do not cancel requests. No API, dependency, timeout, acceptance threshold or memory cleanup policy changes are included.

## Reproduction and verification

- Initial run 20674 exited 101 due to a moved PathBuf in the new fixture. This is not a product reproduction; `tmp/workspace-url-generation-red.log` is retained.
- After correcting only the fixture clone, run 17440 exited 101: 15 workspace tests passed and one failed because two pending requests remained where one untargeted request was expected. `tmp/workspace-url-generation-red-corrected.log` records the actual pre-repair assertion.
- Run 74603 exited 0 after the repair: all 16 workspace tests passed, including four new regressions for same-path session restoration, untargeted preservation, Refresh, failed Open and unfinished Open.
- Run 87861 exited 0: all 27 URL routing tests passed, preserving closed-tab cancellation, queued format migration, dirty-document protection and tab identity. See `tmp/workspace-url-generation-url-regression.log`.
- Fixtures use temporary actual HTML files and existing in-memory settings, cache and global workspace repositories. They do not access the human application or persist user settings.
- Run 46428 exited 0: the official AST suite passed all 23 tests, impacted strict Clippy passed, and formatting plus format-check passed. `tmp/workspace-url-generation-static.log` preserves the result. The private regression file is 284 lines after formatting, within the existing 300-line limit.

## Self-review and remaining release work

The changed production call site is the successful Open handler reached by the actual explorer polling path. Request cancellation precedes session restoration, so same-path restored documents cannot receive old targeted responses. The change does not invoke transient-empty preview cleanup or remove untargeted requests.

Targeted verification and diff/call-site self-review passed. Ordinary signed concern-separated commits, normal push, the individual review reply/resolve and fresh review collection are still required. Current-HEAD coverage, three-OS CI, packaging, all five public assets and genuine packaged-host/human acceptance remain separate release requirements. Earlier commit evidence is not reused for the changed graph.

The production repair and four private regressions were integrated by ordinary signed commit 3c777614cf0339832b43326baf6415daee1b5a78 (signature G), run 53167 exit 0. Its actual timestamp is 2026-10-06 18:51:55 UTC / 2026-10-07 03:51:55 JST. Documentation and publication remain separate steps.
