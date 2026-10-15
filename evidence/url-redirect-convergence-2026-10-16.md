# Distinct-source redirect convergence regression

## Scope

Issue #351 requires independent URL requests not to block unrelated tabs, while
preserving request order when their destinations converge. The existing synthetic
payload tests cover an alias competing with a direct destination request. This
addition exercises two distinct sources through actual loopback HTTP redirects.

The candidate starts from `f06df6260a36f8fc11f9339c895a8eea4d082208` and changes
test modules only. There is no production behavior, dependency, timeout, test-port,
cache, release-policy, or quality-threshold change.

## Contract exercised

- `/a` and `/b` both return HTTP 302 to the same `/final.pdf`.
- The actual repository PDF fixture carries distinct harmless trailing comments.
- A channel releases one response; the app consumes it before the second release.
- Both A-before-B and B-before-A application orders are exercised.
- The final source revision must equal the newer B payload, with distinct A/B
  revisions. Destination generation must advance for newer B, but not for stale A.
- Selection, refresh identity, pending/loading completion, one destination tab,
  and the unrelated HTML document are checked.
- The server joins on the successful path and uses the existing two-second test
  deadline. No sleeps, timeout increases, new production/runtime worker, or user
  app operation occur. The fixture server runs in its own bounded test thread.

## Verification

All logs are under the existing worktree's `tmp/` directory.

- Initial test compilation: session34665 exited101 with fixture error conversions.
  This is not a product RED.
- Session39771 exited101 with a disconnected fixture server. The accepted socket
  needed explicit blocking mode; this is not a product RED either.
- Session64917 exited0: the actual HTTP scenario passed, 0.19 seconds.
- Independent review found the server's single-read request parser could truncate
  fragmented TCP request lines. Session32277 exited101 with three real assertions:
  `/fi` instead of `/final.pdf`, and acceptance of truncated/oversized lines.
- The bounded buffered reader repairs that test-helper defect. Session70625's URL
  test phase passed all81 tests in2.55 seconds, including the three reader cases;
  its AST phase passed30 tests in14.04 seconds and strict all-target UI Clippy with
  `external-fixture-acceptance` succeeded in10.11 seconds.
- The combined session70625 exited1 only at formatting: the new module declaration
  needed alphabetical ordering. After that non-semantic correction, the standalone
  official Cargo fmt check and `git diff --check` exited0.
- Earlier strict invocations using a nonexistent feature name and an owned path
  comparison failed. They are retained in logs and are not successful checks.
- Independent high-reasoning re-review found no remaining P0/P1 or mandatory P2.
  No additional build/test was run by the reviewer.

Final target logs:

- `tmp/url351-redirect-final-url-2026-10-16.log`
- `tmp/url351-redirect-final-ast-2026-10-16.log`
- `tmp/url351-redirect-final-strict-2026-10-16.log`
- `tmp/url351-redirect-fmt-corrected-2026-10-16.log`
- `tmp/url351-http-request-reader-red-2026-10-16.log`

## Evidence boundary

The existing product already passed this convergence case. The addition prevents
regression; it does not establish a newly fixed production defect. The request
reader RED/GREEN belongs to the test server, not KDV/KRR or the application.

Normal push, current-HEAD review, Ready-required CI, packaged/clean-machine human
acceptance, public assets, and Issue closure remain separate steps. macOS Reopen,
startup recovery, renderer fidelity, and conversion latency are not resolved by
this regression. User app/settings/Terms and sibling repositories were untouched.
