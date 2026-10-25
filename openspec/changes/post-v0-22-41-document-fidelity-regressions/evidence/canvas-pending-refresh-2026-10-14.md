# Canvas pending refresh coalescing

Issue: https://github.com/HiroyukiFuruno/KatanA/issues/258
Review thread: https://github.com/HiroyukiFuruno/KatanA/pull/346#discussion_r4260656357

The current request owns a single follow-up-refresh reservation. An automatic notification received after the worker has read old bytes but before the result is polled is retained and schedules one non-forced read after delivery. Both successful and typed failed reads preserve that reservation. Manual force, path replacement and request cancellation discard it with the old request. The original worker count, queue, input limits and timeouts remain unchanged.

The original product failed the actual-worker regression in session58367: the original Canvas text was still displayed instead of updated text. A compiler failure caused by the separate zip9 API migration is not counted as this runtime RED.

The repaired actual-worker tests cover completion-before-poll, invalid-to-valid recovery, repeated notification coalescing, manual-force cancellation, path replacement and unchanged-file pan retention. Session28785 passed all five new tests; session24319 passed the Canvas core24/UI62 cases including those five. Named geometry constants were then substituted without changing values or expected results; session79236 passed the five again. Official session94106 passed AST23, impacted strict Clippy, formatting and diff checks on the updated dependency graph.

Raw evidence is retained under `tmp/canvas-coalescing-core-migrated-exact-red-2026-10-14.log`, `tmp/canvas-coalescing-named-geometry-five-green-2026-10-14.log`, `tmp/current-canvas335-ast-final-2026-10-14.log` and `tmp/current-canvas335-lint-final-2026-10-14.log` in the existing release worktree.

The source repair was integrated through the normal hook in signed commit `4f27c9df` (session39842, exit0). These are targeted source checks, not current-HEAD coverage, required Ready CI, packaged acceptance or public release completion. Normal push, individual reply/resolve and fresh review collection remain separate steps.
