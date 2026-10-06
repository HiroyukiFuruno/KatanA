# Self-Review: Image watch notification recovery

## Checked

- Overflow invalidates subscribed and pending paths; it does not manufacture a persistent registration failure. Genuine failures remain visible until a later successful registration.
- A single private deferred-result map retains dropped registration acknowledgements and failures. Producers preserve per-path order; polling drains the normal queue before applying deferred results. Reset clears deferred results and advances the generation.
- Repaint requests occur after the deferred-result lock is released. Reset releases each state lock before acquiring the next; no new public API, worker pool, allocator or image-cache allowance is introduced.
- Native directory registration changes explicitly invalidate existing subscriptions. Successful removal invalidates remaining subscriptions once per batch. Failed removal retains the existing warning/retry behavior and does not cause a repeated invalidation loop.
- Native rescan flags are handled even when event paths are empty. Actual PNG decoding, texture replacement and repaint coverage remain present.
- Tests use the existing shared native-render environment lock. Two live-loader topology cases remain inside their tests; original timeouts and assertions are not weakened. The topology contracts exercise a real native watcher, but do not claim to prove native callback delivery or application performance.
- Cache limits remain 16 entries and 64 MiB. Japanese WHY comments follow the latest human instruction; executable strings and identifiers remain English.

## Verification

- The original overflow and topology cases have retained assertion-failure logs. An additional dropped successful-registration case failed before its repair (`tmp/watch-registration-overflow-main-red.log`).
- Root verification after functional repairs: all 27 loader tests passed across 40 suites in 2.86 seconds, actual exit 0 (`tmp/watch-registration-overflow-main-green.log`).
- Final static verification first found one Clippy type-complexity error after 23 AST tests passed. A private type alias repairs the lint finding without changing behavior. Final verification: AST 23 PASS, strict impacted Clippy PASS and formatting PASS, each actual exit 0 (`tmp/watch-final-type-{ast,lint,fmt}.log`).

## Findings and boundaries

- The macOS CI HTML first-frame failure at 2062 ms is not repaired or explained by this change. Its original two-second criterion remains unchanged.
- Native unit contracts, targeted verification and static checks are not packaged-main acceptance, current-HEAD full CI, release publication or human acceptance.
- Commit, normal push, individual review reply/resolve and a fresh review-thread query remain required before this review finding is complete.

## Conclusion

PASS for the targeted pre-commit review. No targeted test success is substituted for the remaining formal release gates.
