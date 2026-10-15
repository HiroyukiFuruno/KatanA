# URL response selection ownership

## Scope and observed failures

Current public commit e88f779c received review thread
`PRRT_kwDORm09y86smWvz`: a late response for an independent document could
replace the newer active tab and URL input.

- 57780 exited 101 with three selection/input assertion failures.
- 3986 exited 101 with three additional failures: migration removed the
  selected HTML path without following its PDF destination; an older failure
  replaced a newer success status; an older success erased a newer failure.
- 72201 used the real HTTP enqueue entry point and exited 101 when a response
  replaced a document selected before the first poll. The input assertion was
  subsequently made explicit user input; document selection itself was the
  original failing assertion.
- 9411 exited 101 with one of six cases failing: treating every target/path
  mismatch as background also suppressed a newer targeted response after an
  older URL-bar response completed first.
- 1465 exited 101 after real local-file workspace restoration: a retained
  targetless request replaced the newly restored local document selection.
  The preceding 78783 missing-trait compilation failure was a test-fixture
  error, not evidence of a product defect.

## Repair contract

- Global completion order is separate from per-destination content order.
  Destinationless failures also advance completion order.
- Production enqueue captures a shared selection owner. Manual action changes
  and successful asynchronous workspace restoration invalidate that owner.
  Polling does not create missing owners merely to support unit fixtures.
- URL completion itself does not invalidate the owner. A newer requested
  response can still activate after an older response completed first.
- Background application updates document content or structured failure while
  retaining selection, URL input, URL-tab selection, tab-scroll intent and
  global status/error. Loading remains an aggregate of the actual queue.
- Original selected paths take precedence. Only a selected request target
  actually removed by format migration follows its computed destination.
- No new runtime, public core API, persisted setting, dependency override,
  timeout extension, lint allowance or quality threshold change is introduced.

## Scoped verification

- 83885: official URL tests, exit 0, 53 passed, 2.57 seconds.
- 99142: request-order tests, exit 0, 7 passed.
- 57702: workspace generation tests, exit 0, 5 passed. Final responsibility-based
  test-helper separation: 65592, exit 0, 5 passed, 0.02 seconds.
- 4832: final official AST checks, exit 0, 23 passed, 13.51 seconds.
- 58576: final locked UI all-targets/external-fixture-acceptance strict Clippy,
  exit 0. 45101's pointer-argument finding was corrected without relaxing lint.
- 9341: final official format check, exit 0. Markdown check and diff check
  also exited 0.
- Main inspected the diff and actual action/enqueue/workspace call sites.
  Independent read-only review found no remaining functional P0/P1 in the
  scoped candidate; a fixture-only poll compensation finding was removed.

These are source-level checks, including real HTTP and local filesystem
boundaries. They do not prove native document rendering, full release coverage,
packaged host acceptance, crash recovery, Ready CI or public release. Normal
source integration, upload and exact-head review remain separate requirements.

## Integration checkpoint

Normal source commit 4d0e0e5d completed with exit 0 (1610). It contains the
selection-owner repair and its regressions. Upload, exact-head review response
and resolution are not implied by this local source commit.
