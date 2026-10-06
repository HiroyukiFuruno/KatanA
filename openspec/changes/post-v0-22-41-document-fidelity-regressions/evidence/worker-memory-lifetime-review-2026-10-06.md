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
