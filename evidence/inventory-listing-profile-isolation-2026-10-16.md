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

Normal integration/push, P1 reply/resolve and a fresh original full coverage run with
inventory enabled remain pending. Earlier successful coverage reports that
included auxiliary listing profiles are historical command results, not proof
of uncontaminated coverage. No coverage threshold, exclusion, test selection,
Draft policy, dependency or product runtime is changed by this repair.
