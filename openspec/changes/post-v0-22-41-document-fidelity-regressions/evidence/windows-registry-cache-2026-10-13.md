# Windows registry cache — Issue #343

## Scope and history

Commit `4f73b862` introduced `runner.os != 'Windows'` on CI Cargo caches. Its recorded commit summary says "skip cargo cache on windows"; the inspected diff provides no reason or measured failure. Do not infer a corruption or capacity cause without evidence.

The current release workflow already has a Windows Cargo registry/target cache. The initial Issue's statement that release has no equivalent cache is outdated; this change does not add a duplicate release cache or alter release artifact validation.

The two Windows CI jobs now cache only Cargo registry index/archive and Git dependency source databases. They do not restore compiled `target` artifacts. Their exact key contains OS, architecture, a cache-schema version, and all tracked Cargo lock digests, without broad restore prefixes. Toolchain, profile, target and feature changes still build and test current source because compiled artifacts are absent from this cache. A cache hit is not verification evidence and does not skip any quality step.

## Verification

- Before workflow edits, `python3 scripts/release/test-windows-registry-cache-contract.py`: exit 1, current workflow rejected for missing Windows registry cache. The initial negative cases were only meaningful after the positive case passed; final assertions require the precise rejection reason.
- After edits, the same contract: 4 tests, exit 0. It rejects target artifacts, missing lock identity, and cache-hit test skipping, and requires existing Windows release build/asset/smoke steps.
- `python3 scripts/release/test-draft-ci-contract.py`: 5 tests, exit 0. Draft runner exclusion and Ready event wiring remain intact.
- `python3 scripts/release/test-release-flow-contract.py`: 9 tests, exit 0.
- `git diff --check`: exit 0.

## Remaining acceptance

This is one implementation step, not completion of Issue #343. Actual Windows cold/warm duration, runner time, cache hit and source download timing must be collected after Ready; do not start Draft CI to obtain measurements. The test/coverage inventory equivalence, safe result reuse, freshness rejection and total-cycle comparison remain open. No tests, coverage threshold, supply-chain, packaging, or publication gate was removed.
