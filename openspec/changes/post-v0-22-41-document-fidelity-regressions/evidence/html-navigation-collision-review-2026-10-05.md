# Existing HTML navigation target ownership

## Finding and scope

Current-HEAD review 5416352520 on c8335e48 reported that navigating from local HTML A to already-open B preferred A's document record, renamed it to B and removed B's preview. This produced two records with the same path. The review-specific GraphQL query returned no inline comments, and the complete paginated fetch returned no thread for this finding. Its review URL is https://github.com/HiroyukiFuruno/KatanA/pull/346#pullrequestreview-5416352520.

Only `crates/katana-ui/src/app/action/html_navigation.rs` changes. Existing target lookup now precedes source fallback. For a distinct source/target collision, the existing target is activated without replacing its document record, dirty buffer, pinned flag or per-tab layout. Its existing browser session receives the requested source/fragment through the existing preview operation. Source records and previews remain owned by their original paths. Missing-target navigation and same-path reload retain the existing replacement path. No public API, sibling dependency, rendering contract, acceptance limit, lint or coverage exclusion changes.

## Regression and integration review

The bounded child observed the original duplicate-path regression fail with two target paths, then a target-first intermediate implementation passed its focused navigation suite. The main integration review rejected that intermediate implementation because it could overwrite a dirty target's unsaved content. The child added a real temporary-file/file-URL regression and observed the intermediate implementation replace `unsaved-target` with the disk content. This second RED is explicitly against the intermediate target-first implementation, not the original HEAD; raw receipt is `tmp/html-navigation-collision-dirty-intermediate-red-2026-10-05.log`.

The repaired collision branch preserves the dirty target and both document/preview owners, while forwarding B#details and retaining target history, pinned/split/pane order. A separate same-path reload regression requires one record, replacement content and old-to-new history. The child filtered suite passed seven tests with zero failures/ignored tests. Its final raw output is retained at `tmp/html-navigation-collision-child-green-2026-10-05.log`. The main inspected the actual diff and RED/GREEN logs, not only the child's completion message. Formatting and diff checks passed.

The main canonical `with-office-test-worker.sh` followed by `just T=html_navigation test-specific` exited 0: seven passed, 1836 filtered across 39 suites. JOBS=2, CARGO_INCREMENTAL=0, existing debug/strip profiles and `-D warnings` were retained. Raw receipt: `tmp/html-navigation-collision-canonical-green-2026-10-05.log`. Main self-review found the final branch scoped to distinct existing-target collisions; source/path ownership and per-tab state remain unchanged, and no mock, fixed wait, disabled check or threshold relaxation was added.

Normal source commit completed with exit 0 and signature G as `834b58b8`, passing the existing commit verification hooks without bypass. Raw receipt: `tmp/html-navigation-collision-source-commit-2026-10-05.log`. The preceding packaged/original-HTML evidence was separately committed as signed `2d3e53e8`; it is not evidence for the new navigation source.

Normal push, individual review response, fresh review retrieval and changed-source coverage/packaging/acceptance remain subsequent checkpoints. None are inferred from focused success. Earlier c8335e48 packaged startup/original-HTML proof remains historical for this additional Rust change. PR346 stays Draft, and manual-target policy, full native distribution acceptance and independent scores remain unresolved.
