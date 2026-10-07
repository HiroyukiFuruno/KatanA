# Parallel workspace polling contract

## Reproduction

The normal push at `306531fe` failed the native parallel integration suite:142 passed,1 failed,2 existing ignored. Document navigation expected two open documents but observed one. Running the identical binary with the exact navigation test alone passed. A subsequent complete Cargo run failed another workspace-dependent test (three document tabs expected, two observed).

The shared helper previously returned after its existing100-step polling loop even when `workspace.is_loading` remained true. A late workspace completion clears document tabs in `finish_open_explorer`. Adding a fail-closed assertion without changing the polling budget reproduced both failures at that precise boundary:141 passed,2 failed,2 existing ignored, exit101. See `tmp/navigation-parallel-loading-budget-2026-10-04.log`. The helper no longer misrepresents unfinished loading as successful setup.

## Repair and verification

Test execution now defaults to the existing `JOBS` budget through overridable `RUST_TEST_THREADS`; the native Justfile, real Linux container entrypoint and three-OS CI test job use two threads by default. Coverage and fixture commands already explicitly use that budget. No test, timeout, assertion, performance acceptance threshold, coverage exclusion or product implementation was removed or weakened.

With the new fail-closed assertion and the identical complete suite, two threads pass143 tests,0 failed,2 existing ignored in28.37s, exit0 (`tmp/navigation-parallel-loading-bounded-2026-10-04.log`). A prior same-binary two-thread control also passed143 tests. The new entrypoint contract is RED against the previous configuration and GREEN after the fix; all six CI resource contracts pass. Justfile evaluation reports2 and retains explicit caller overrides.

An earlier direct-binary run from the repository root also failed a locale test because its relative fixture path requires the package working directory. That invocation error is not attributed to the product and is not used as repair evidence.

## Self-review and remaining gates

Final serial integration18 tests pass in2.69s, test-inclusive Clippy for parallel/serial targets passes with warnings denied, AST23 tests and formatting pass, and diff whitespace validation passes. Master remains clean, stash count0, and only the two existing worktrees are present.

The diff strengthens load-state validation and preserves the exact workspace-root assertion before document selection. It bounds test-suite scheduling, not background product behavior; it does not claim Office performance or upstream KRR resolution. Formal commit/push, full coverage/platform gates, fresh Office/HTML and packaged release acceptance remain separate pending checkpoints.
