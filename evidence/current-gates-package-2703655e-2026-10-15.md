# Published branch gates and local macOS package

Source: `2703655e739473a60802a6b7be3cd5069266d48e`, release/v0.22.42.
This is a published branch commit, not a GitHub Release.

## Normal integration and review

- Nested-tag repair a9638417, Canvas map contracts94587b53 and task record2703655e
  were ordinary signed commits, local signature G.
- One normal push88715 exited0 after native tests, real Linux container
  74c0d6106c71 and release preflight. Local/origin/PR346 HEAD matched and status
  was clean0/0; GitHub reports verified/valid. Raw:
  `tmp/map-and-tag-normal-push-2026-10-15.log`.
- Nested-tag finding4282450025 received direct reply4282819292, then resolve;
  fresh complete GraphQL89threads and REST161reviews/179comments/144conversation
  comments (two pages each) confirmed it resolved and no nested truncation.
- Request6191827674 was sent once. Bot6191882028 at12:19:24UTC explicitly
  reviewed2703655e73 and reported no major issues. The empty owner-created
  COMMENTED review is not used as independent approval.
- Snapshot: `tmp/2703655e-reviewed-complete-2026-10-15.json` and the three
  `tmp/2703655e-reviewed-rest-*-2026-10-15.json` files.
- Ten historical review threads were independently checked resolved, with
  actual repair commits in HEAD ancestry. Their old pending rows are narrowed
  to review-repair completion; publication and packaged requirements remain.
  Registered-font and direct-mode repairs are662c9059/a7555b9d, not their
  reviewed baseline c62644f2.

## Fresh official full coverage

`just JOBS=2 coverage`, with existing inventory recording enabled, session6528
exited0. No exclusion, threshold, existing ignore or fixture policy changed.

- Core258 passed; UI library1308 passed/2 existing ignored.
- Real export13 passed/1 original filtered,253.87seconds.
- UI parallel144 passed/2 existing ignored,22.17seconds.
- UI serial18 passed,1.81seconds.
- All four phases exited0. Meaningful-line gate passed; strict document
  surface100%, zero uncovered lines.
- Begin/end source guard confirmed the same clean2703655e. Root lock SHA256:
  `a6035ec9364f734fd132d58ca909cfed0b9c9e13c25ffc894165d79060db2af4`.
  Screenshot lock:
  `ef7151771258e762d1962647ccf4625dd668d48ec3a5246b60bcb48734955896`.
- Raw `tmp/2703655e-full-inventory-coverage-2026-10-15.log`, report
  `tmp/2703655e-full-coverage-report-2026-10-15.json`, source identity
  `tmp/2703655e-full-coverage-source-2026-10-15.json`.
- Four inventories under `target/test-inventory/2703655e-full-coverage-2026-10-15/`
  have exit0, source_stable=true, native listing counts8/25/1/1 and no listing
  reasons. Their complete=false/reuse_authorized=false remains intentional;
  this does not prove test deduplication or authorize skipping tests.

## Local static macOS package

Existing `just VERSION=0.22.42 package-mac`, session8952 exited0. Native bundle
took2m12s, arm64 build3m06s and Intel build2m58s. Raw:
`tmp/2703655e-package-mac-2026-10-15.log`.

- Output: `target/release/bundle/osx/KatanA Desktop.app`.
- Both main and Office worker are Universal x86_64/arm64; all four slices
  report minos13.0. Info.plist reports v0.22.42.
- `codesign --verify --deep --strict --verbose=2` exited0. Signature is ad-hoc;
  no paid Apple signature/notarization is claimed or required.
- Begin/end source identity confirmed clean2703655e; record:
  `tmp/2703655e-package-source-2026-10-15.json`.
- Main SHA256:
  `44f222bdc8c34e6a70e5b77e788517c3013c9ac6f6656d5d0f2e1dd8b2de5c0e`.
- Worker SHA256:
  `835ce401af1335ac86ef68f40af702e9ae2cc18bdb9ed702cad8cf1ed53bc771`.
- Info.plist SHA256:
  `8c7fbdb0a3cbb2fd27a9582950537afa4cdbbc8a0393da382c56c4a85691339c`.

The user app was not running or launched; settings and Terms were not changed.
This is static local packaging evidence, not native human acceptance,
clean-machine runtime, all-OS packaging, independent scores or published assets.
The package was built at2703655e, not at the later documentation commit.

## Still incomplete

PR346 remains Draft. Its five cloud checks are SKIPPED, not successful required
CI. Three duplicate manual-target policy threads remain pending the previously
requested human choice. This document neither decides that policy nor repeats
the request. Ready/current required CI, genuine packaged-host acceptance,
all five public assets/checksums, GitHub Release, Issue closure, OpenSpec archive
and real publication cleanup remain unproven. Upstream defects and next-release
startup recovery keep their explicit deferred scope.
