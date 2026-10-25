# Self-review: transient image registration retry

## Scope

Current review thread `PRRT_kwDORm09y86patMV` reported that a failed watch registration leaves a visible image permanently failed. The candidate keeps the failure visible while retrying registration asynchronously only when that image is requested. Inactive paths are not retried from the global polling loop.

## Reproduction and verification

- `tmp/watch-transient-failure/red-missing-parent-raw.log`: the original implementation fails the real missing-directory/created-PNG recovery assertion with exit 101, `pending=true`, and a missing-parent error.
- `tmp/watch-transient-failure/root-final-missing-parent.log`: the same real directory/PNG creation recovers without resetting the loader; exact test 1 passed.
- `tmp/watch-transient-failure/root-final-retry-callback.log`: the real failure stays visible and schedules a positive delayed repaint no later than the retry upper bound; exact test 1 passed. The test uses the natural deadline, not a mutated private timer or a longer recovery timeout.
- `tmp/watch-transient-failure/root-final-local-image-loader.log`: all 29 loader tests passed, including existing reset, stale generation, genuine error, deferred result, ownership, and cache behavior.

## Diff and call-site review

The private failure value owns error text, generation, attempt count, and retry deadline in the existing error map. Failure application clears pending/watched state; successful registration clears the error. Registration removes the failed request's old targets and unused watches. Existing generation checks, queue limits, cache limits, and worker ownership are preserved.

Retry delays are 100/200/400/800/1600 ms. Repeated visible requests do not create duplicate pending registrations. No UI-thread file I/O, new public API, allocator, test port, ignored test, relaxed gate, or altered recovery timeout is introduced.

This is not proof that every OS-level directory replacement or shared-backend cached-watch failure recovers. The existing shared backend cache and broader directory replacement behavior are outside this finding's demonstrated reproduction; they are not claimed fixed here. Native human acceptance, packaged acceptance, new-HEAD CI/coverage, normal integration, and thread reply/resolve remain separate release steps.

## Conclusion

Targeted behavior verification and final static verification pass: `root-final-ast.log` records 23 tests passed, `root-final-lint.log` records impacted Clippy exit 0, and `root-final-fmt-check.log` records the format check succeeding. Main-agent diff/call-site and test-integrity review found no remaining issue in this scope. The six source/test files were integrated by normal hooked, signed commit `a1e914a801dd0d40becaa49203fdee971ed8f8d5`. Push and review-thread resolution remain in progress. Coverage for public HEAD `68348738` is not reused for this graph.
