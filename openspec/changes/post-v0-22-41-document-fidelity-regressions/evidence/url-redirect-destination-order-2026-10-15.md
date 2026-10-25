# Redirect destination request order

## Reproduction and repair

- Before the repair, regression 66866 exited 101: an older request redirected to
  an existing PDF and changed its generation from 1 to 2 after a newer request
  had already updated that destination.
- Requests now receive insertion-order identifiers once before polling. The
  actual destination calculation is shared with response application. An older
  completion cannot overwrite a newer completion at that destination.
- Existing same-target/source and consecutive URL-bar ordering remains unchanged;
  distinct known targets still progress independently. Format migration keeps
  the pending queue in application state.
- The metadata contains paths and order identifiers, not document payloads.
  Pruning follows the oldest pending request; empty queues and cancellation
  release capacity and reset the counter. Checked counter exhaustion reports an
  explicit request failure instead of wrapping or consuming the response.
- There is no fixed entry-count cap. Retention follows pending-request lifetime
  and the existing deadline; this is not proof of a fixed memory budget or of
  application-wide RSS/performance acceptance.

## Current candidate validation

- 45514: official `just T=app::action::url_source test-specific`, exit 0,
  39 passed, including redirect collision, empty-queue subsequent refresh,
  real HTTP reload, ordering, migration, timeout and close cases.
- 13940: official `just T=state::url_tab test-specific`, exit 0, 9 passed,
  including pruning, capacity release, counter exhaustion through the real
  polling entry point and immediate cancellation cleanup.
- 51302: official workspace URL generation selection, exit 0, 4 passed.
- 60428: official AST gate, exit 0, 23 passed, 13.19 seconds.
- 7543: locked UI all-targets/external-fixture-acceptance strict Clippy, exit 0.
- 95085: official format check, exit 0. No ignores, new allow entries,
  timeout extensions, quality threshold changes or public core APIs were added.
- Initial direct Cargo tests 43673/70797 passed but emitted a large debug-unwind
  linker warning. They are not substituted for the subsequent official-profile
  verification above.

These are scoped source checks, not packaged rendering, user acceptance,
current published-HEAD review, Ready CI, or release evidence. Issue 351 remains
open until its remaining acceptance and publication requirements are met.

## Current-review mixed-target head-of-line blocking

- Current public HEAD 08371680 review 5527055431 added P2 comment 4278185039.
  The mixed None/Some catch-all still blocked unrelated completed requests.
- New real-caller unit regressions 14493 exited 101 with two concrete failures:
  targetless-then-targeted and targeted-then-targetless both retained no source
  where a ready response should already have been applied.
- Mixed targets now wait only for identical source URLs. Actual redirect
  destination collision protection uses the order table instead of freezing
  unrelated tabs. Consecutive URL-bar requests retain their original FIFO.
- The previous redirect test now requires the newer target to render early;
  its final newer-content assertion is unchanged. This removes an artificial
  waiting assertion, not the latest-content protection.
- 16825 official URL selection: 42 passed, exit 0, 2.59 seconds.
- 8106 official AST: 23 passed, exit 0, 13.40 seconds. 69115 locked UI
  all-targets/external-feature strict Clippy: exit 0.
- After test-helper responsibility separation, final 97666 official URL42,
  25628 AST23 and 33838 all-targets/external-feature strict Clippy exited 0.
  Workspace cancellation 58873 passed 4 cases; format 41406 exited 0.
- 3e4e2b6d is the first signed source concern commit. The subsequent mixed-target
  source, normal upload and exact-HEAD review reply/resolve are tracked
  separately; no public repair is claimed here.
