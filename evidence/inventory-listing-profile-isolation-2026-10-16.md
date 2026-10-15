# Exclude native inventory listings from measured coverage

PR346 review5535514218, P1 comment4284857791, identified that instrumented
`--list` and `--list --ignored` processes inherited the real test's
`LLVM_PROFILE_FILE`. Their auxiliary profile data could affect the later report.

## Reproduction and repair

- Real miniature Cargo/llvm-cov reproduction: session53008 exited1. Both new
  assertions failed: one actual test produced three profraw files, and another
  listing added profiles to the measured directory. Five imported existing
  tests also ran and passed; the test module import was subsequently corrected
  so discovery does not duplicate those five cases.
- Listing processes now receive a copied environment with only
  `LLVM_PROFILE_FILE` redirected into a dedicated temporary directory. Actual
  test execution, instrumentation flags and recorded test environment remain
  unchanged. Both listing modes and both recording kinds use this boundary.
- A temporary directory inside the workspace or any known report/build/profile
  root is rejected. Isolation setup/cleanup errors produce unavailable listing
  diagnostics, not a replacement for the real test's exit status.
- Unsafe TMPDIR reproduction: session46394 exited1 with one failing assertion.
  The repaired final session62852 exited0: three tests passed in16.001seconds.
  They verify exactly one real-test profile, unchanged existing profile names
  and hashes after repeated listings, a valid real coverage report, and an
  unsafe temporary root retaining the original test failure101 without listing.

Raw logs: `tmp/listing-profile-isolation-red-2026-10-16.log`,
`tmp/listing-profile-unsafe-root-red-2026-10-16.log`, and
`tmp/listing-profile-isolation-final-2026-10-16.log`.

## Remaining verification

The full native inventory target collection passed: session73669 actual exit0,
71 tests in23.344seconds; `tmp/listing-profile-native-targets-2026-10-16.log`.
Both changed Python files satisfy the existing 200-line file/30-line function
limits. KML and diff checks passed. The original test status, coverage selection,
thresholds and exclusions are retained; there is no new skip or bypass.

Independent final read-only review found no additional P0/P1 or mandatory P2.
Main self-review confirms the diff is limited to listing profile isolation,
real regression tests and their evidence, with no product or public API change.

## Original full gate and normal integration

The repair was integrated by signed normal commit
`77229527dccfa15cc11557c263ca8ef4a923d049` (session19924 actual exit0).
With inventory enabled, the original `just JOBS=2 coverage` session11776 exited0:
core258, UI1308 with the original two ignored tests, export13 with the original
one filter (282.16seconds), parallel144 with the original two ignores, and
serial18. Both the original meaningful-line gate and strict document100%/zero
uncovered gate passed. An independent check of the emitted JSON report passed.
Beginning/end source guards confirm the same clean HEAD and unchanged locks.

- Raw `tmp/77229527-full-coverage-2026-10-16.log`, SHA256
  `f80799a07a6da34984cd441272de1a3d74d2b6556da777f9e1d39c43de0db9d5`.
- JSON `tmp/77229527-full-coverage-report-2026-10-16.json`, SHA256
  `399ba1de84bcea0d9943a5df07c2a69a496788d8c6366087546971edb2d58ceb`.
- All four inventory records retain source_stable=true, exit0, no listing
  errors and complete/reuse=false; native record counts are8/26/1/1.
- Normal push83637 exited0 after its native, real Linux and preflight chain.
  `tmp/listing-profile-isolation-normal-push-2026-10-16.log` records the actual
  remote update45747305..77229527. Linux log12549 exited0, including export14,
  UI1288, parallel142 and serial18 with no new ignores.
- Individual reply4285268755 precedes resolution of
  `PRRT_kwDORm09y86s1xMl`. Fresh complete retrieval confirms resolution.
- New-HEAD review request6196372796 received bot response6196423500 at
  2026-10-15T15:59:22Z, explicitly reviewing77229527dc with no major issues.
  Complete GraphQL retrieval21465 found90 threads and no truncated nested
  comments. Only the three existing manual-target policy threads are unresolved.

Earlier successful reports containing auxiliary listing profiles remain
historical command results, not proof of uncontaminated coverage. No coverage
threshold, exclusion, test selection, Draft policy, dependency or product runtime
changed. This closes the specific P1 repair, not full Issue343, Ready CI,
all-platform distribution acceptance or public release.
