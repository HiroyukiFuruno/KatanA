# URL UI ownership before a newer request completes

## Current review and scope

PR346 current e0fcafa6 review5527794142 identified REST4278800773 / thread
PRRT_kwDORm09y86snOcm: targeted A may succeed after targetless B has been enqueued
but before B fails. A must update its content without replacing B's pending URL
input or selection. Completion serial alone cannot represent this pending intent.

`fetch_html_url` now replaces the existing selection owner when a valid request is
enqueued, before recording that same owner on the new request. Previously queued
responses become background; the latest queued request still owns foreground
completion. This uses the existing token and introduces no additional queue,
counter, public API, network concurrency limit or dependency change.

## Regression integrity

The added tests use a real loopback HTTP request to `/b.html`, a controlled HTTP500
response and channel barriers, not fixed sleeps or a mocked transport. They cover
A-success-before-B-failure and B-failure-before-A-success. The assertions retain
B's active document, URL input, existing HTML body, actual HTTP500 error and global
failure status while requiring A's body to update. Initial document seeds are
actual `HtmlSource`, not a fabricated loaded worker result.

- session2276 exit101 was a fixture type error (`FetchedUrlSource` passed where
  `HtmlSource` is required), not product RED.
- session77222 exit101 reported disconnected channels because the fixture's
  accepted stream remained nonblocking; correcting that fixture was required.
- session66005 exit101 reproduced the product's active-path assertion failure
  in the A-first case. Raw output is retained by RTK as
  `/Users/hiroyuki_furuno/Library/Application Support/rtk/tee/1792036686_cargo_test.log`.
- The same run's reverse-order failure expected a per-document failure pane for
  an HTML intake failure. That expectation did not match the existing HTML
  contract and is not labeled as a separate product bug. The corrected test
  requires the real structured global error and visible error status instead.

After the production change, main separated network lifecycle into the
`ControlledFailure` fixture and grouped initial state in `Scenario`, preserving
all meaningful assertions and both completion orders. The original two-second
fixture deadlines are unchanged. Final focused/strict results and publication
evidence are recorded only after their actual processes finish.

This is not whole Issue351, OS-native packaged UI, independent95 acceptance,
Ready CI or release completion. The human application/settings/Terms were not
operated, and the cancelled monitoring schedule was not recreated.

## Automatic refresh and stale destination selection

The actual automatic refresh caller also used the same fetch entrypoint. A
background PDF/Office refresh must not acquire a foreground selection token.
Real loopback tests reproduced both completion orders in session34123 (exit101).
The shared fetch implementation now distinguishes foreground user requests from
background automatic refresh, and background completion records destination
ordering without advancing the foreground completion serial.

An independent review then identified a separate selection loss: a newer
background response can update a destination before an older, still-current
user request resolves there. Rejecting the older payload also discarded the
user's selection. Session47957 reproduced two active-path assertion failures
and one passing invalidated-owner negative case. The first test explicitly
requires the background update to retain the original selected document before
the user completion. Raw RED output:
`/Users/hiroyuki_furuno/Library/Application Support/rtk/tee/1792038029_cargo_test.log`.

The repaired stale-response path selects only an existing cached destination
while the request still owns foreground selection. It does not apply the old
payload, old failure, or a new preview session. Tests retain the destination
generation and cached source identity for both success and failure, and retain
the manually changed selection when ownership has expired.

- session20579: UI test type-check exit0, 5.26s (before final repair).
- session9326: official URL tests exit0, 63 passed, 2.66s.
- session31392: request-order tests exit0, 9 passed.
- session23948: UI all-target strict Clippy with
  `external-fixture-acceptance`, exit0, 5.90s.
- session30379: AST exit101, 22 passed and one shared-rule failure from five
  unnamed HTTP status literals in the new fixtures. Named status constants
  replace those literals without changing assertions or responses.
- The first strict command used the nonexistent `external-test-fixtures`
  feature and exited101 before compilation; this is a command error, not a
  product regression or a successful check.

Final post-constant results: session89298 URL63 passed (2.61s), session13990
AST23 passed (13.54s), session68488 all-target external-fixture strict Clippy
exit0 (7.22s). Session46332's `workspace_open` filter ran one matching test,
not the historical five-test suite. Final rustfmt and diff checks exited0.
The independent read-only review found no remaining P0/P1 or required P2 for
this limited concern. Normal commits, normal push and the PR thread
reply/resolve remain separate requirements.

The source was integrated by normal signed commit
`8a8a5328f57708ccce08a8adbff16c7c9b14f113` (session63430 exit0).
This is local history evidence, not remote publication evidence.
