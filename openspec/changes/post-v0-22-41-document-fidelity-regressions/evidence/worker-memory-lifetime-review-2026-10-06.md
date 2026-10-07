# Worker-owned memory release order

## Review and ownership

- Current review: PR #346 review 5425249330, commit `b8ff6c632c65ea95eec5304ce466c70d91781668`.
- Finding: https://github.com/HiroyukiFuruno/KatanA/pull/346#discussion_r4192736419 (`PRRT_kwDORm09y86pXXH6`, P2).
- Closing a surface drops its command sender, not its detached worker. The worker may still own a frame or session while the UI cleanup returns.
- This is a KatanA worker-lifetime repair, not an upstream KDV implementation or a change to the deferred-release scope.

## Repair

The existing document-worker lease owns the release ordering. The last-document cleanup requests memory relief rather than invoking it immediately. A pending request is consumed only when the worker count reaches zero, after the worker's owned closure/session/source has dropped. Starting a new worker cancels the previous close request. The state mutex serializes request, acquire, release and the allocator call; no UI thread joins or fixed waits are introduced.

The existing final-document transition predicate remains unchanged. Retained/pinned documents, HTML-only cleanup and repeated empty cleanup do not arm a new request. The existing macOS allocator call is relocated, not replaced by a new allocator; other platforms retain the no-op behavior. Cache limits, RSS thresholds and public dependency versions are unchanged.

## Actual targeted evidence

- Immediate-release policy RED: `tmp/worker-memory-lifetime-red.log`, actual exit 101; 0 passed / 1 failed, `live workers must defer memory relief`.
- Deferred policy GREEN: `tmp/worker-memory-lifetime-memory-green.log`, actual exit 0; 4 passed.
- Existing lease ownership, panic, spawn-failure and unstarted-closure cases: `tmp/worker-memory-lifetime-lifecycle-green.log`, actual exit 0; 7 passed.
- Existing real cleanup caller and transition cases: `tmp/worker-memory-lifetime-caller-green.log`, actual exit 0; 7 passed.
- Existing native image GUI regression: `tmp/worker-memory-lifetime-gui-green.log`, actual exit 0; 1 passed.
- Final borrow-only cleanup: `tmp/worker-memory-final-lifecycle.log`, actual exit 0; 7 passed.
- Final AST: `tmp/worker-memory-final-ast.log`, actual exit 0; 23 passed. Formatting: `tmp/worker-memory-final-fmt.log`, actual exit 0.
- Strict lint: `tmp/worker-memory-final-lint.log`, actual exit 101 because the required libm rmeta vanished during compilation. The worktree's debug and coverage target directories were subsequently absent. This is not a lint pass or a diagnosed source violation. Normal `just lint-impacted` rebuild subsequently completed with actual exit 0: `tmp/worker-memory-final-lint-rebuild.log`.

These focused results are not current-head CI, full coverage, a packaged receipt or a stable RSS improvement claim. The repair is integrated by normal signed commit `188b3c1e24b5f46bbbcca006c16f600d8b4868a0`; GUI diagnostics are separately integrated by normal commit `9b7e869`. Normal push, individual review reply/resolve and fresh full verification remain separate checkpoints.

## GUI coverage failure retained

The previous official `b8ff6c63` coverage run failed (UI 1116 passed / 1 failed / 2 ignored) at the native image GUI wakeup assertion. Added assertions record whether the initial GUI is actually idle and include egui pending repaint / watcher state only on failure. The same-profile diagnostic UI run passed 1117 tests with 2 ignored in 19.71 seconds, but did not reproduce or prove a repair of the earlier failure. The mismatched debug-profile diagnostic was interrupted with actual exit 130 and is not test evidence. No timeout, assertion or ignore was relaxed.

## Non-worker document reopen boundary

Main self-review identified that HTML, PNG and Markdown do not acquire a document-worker lease. Cleanup therefore cancels a pending close request whenever documents or previews remain/reappear. The normal push was interrupted before upload (actual exit 130, three commits ahead, zero behind); the killed Mermaid export in that interrupted log is not a product failure.

- No-op cancellation RED: `tmp/worker-memory-reopen-red.log`, actual exit 101, 0 passed / 1 failed at `reopened content must cancel deferred relief`.
- Implemented cancellation: `tmp/worker-memory-reopen-memory-green.log`, actual exit 0, 5 passed.
- Existing caller: `tmp/worker-memory-reopen-caller-green.log`, actual exit 0, 7 passed.
- Existing lifetime: `tmp/worker-memory-reopen-lifecycle-green.log`, actual exit 0, 7 passed.

Additional-boundary AST (`tmp/worker-memory-reopen-ast.log`, 23 passed) and strict impacted lint (`tmp/worker-memory-reopen-lint.log`) completed with actual exit 0. Initial formatting check failed solely on the new assertion layout; normal `just fmt` completed with exit 0, then `tmp/worker-memory-reopen-fmt-after-format.log` completed with exit 0. Normal integration remains separate. These tests do not prove packaged acceptance or stable RSS improvement.

The additional boundary is integrated by normal signed commit `e5ab75c5`.

## Current public-head CI failures

The exact public `b8ff6c63` CI failed on Ubuntu at `watcher_overflow_retries_registration_without_persisting_failure`: 1095 passed / 1 failed / 2 ignored, with the expected red image not published before the original five-second deadline. macOS failed at the dirty-target HTML initial frame: 1116 passed / 1 failed / 2 ignored, startup accepted, 2137 ms elapsed, worker not idle, no frame generation. Its failure-only sample also exceeded the independent five-second bound and was killed/reaped, so no failed-process stack was obtained. Raw completed-job logs are `tmp/worker-memory-live-{ubuntu,macos}-job-logs.zip` (despite the filename suffix, these API responses are text containing ANSI sequences).

Host code waits directly on the public adapter update queue. In published KDV 0.5.12, initial session construction and frame publication precede marking the worker ready. This narrows the observed boundary but does not identify the internal stalled stage or prove an upstream cause. Both failures remain distinct from targeted local passes.

## Shared native watcher fixture audit

Two local image tests (reset and active-texture lifetime) also reach the process-global watcher but omitted the existing `RenderEnvLock` used by other native watcher tests. They are brought under that same fixture lock. The expected-image wait now includes the already-existing watch state on failure. Neither the five-second deadline nor success assertions are changed; explicit two-live-loader topology tests remain. A shared-environment interference possibility is not a proven cause of the Ubuntu failure.

The resulting official local-image suite passed 27 tests with actual exit 0 in `tmp/watch-fixture-isolation-tests.log`. AST passed 23 tests, strict impacted lint and formatting also completed with actual exit 0 (`tmp/watch-fixture-isolation-{ast,lint,fmt}.log`). Normal integration and full verification remain separate checkpoints, not a current-head CI or full-coverage claim.

The fixture and failure-diagnostic changes are integrated by normal commit `fede89a`.
