# Post-merge CI admission

## Scope

Issue #343 already reuses a completed push workflow for the exact version-bump
merge SHA through `verify-version-bump-ci.sh`. The production caller is
`build-and-release.yml`, step `Verify CI gate for version bump merge`.
Previously that script accepted the workflow conclusion alone. It did not
independently reject a missing or skipped platform job inside a successful run.

The admission now reads the live run identity, all job pages for its explicit
attempt, and the live run identity again. It requires the same repository, exact
SHA, master push event, fixed CI workflow path, run and attempt throughout.
Every macOS, Windows and Ubuntu `Test and Build` job must finish successfully.
Duplicate identities, missing jobs, skipped/neutral/failing platform jobs,
unexpected test matrix legs, incomplete pages and concurrent reruns fail closed.
The existing wait window and latest-run selection are unchanged.

PR-only lint, supply-chain and CodeQL jobs remain separate requirements. Their
intentional push skips are not counted as successful execution and cannot
replace Ready checks. No test is removed, no new workflow is dispatched, and no
local inventory file receives CI skip/reuse authority.

## Validation

- Existing caller/wiring regression failed before repair: one assertion failure
  in `tmp/ci-run-admission-wiring-red-2026-10-16.log`.
- The initial missing-module errors are implementation scaffolding failures,
  not evidence that a live production run was admitted incorrectly.
- Nine focused admission tests pass; release-flow ten and Draft-policy five
  existing tests pass. Bash syntax, wait-window contract and diff checks pass.
- The actual GitHub API read of run `33025396317`, attempt 1, confirms all three
  jobs succeeded for historical SHA `0561692fadef215cf8bec9752d4072a7cdff5960`.
  The helper exits zero; no CI or app was launched. This proves API compatibility
  and historical admission only, not current release HEAD success.
- An independent read-only review found no mandatory findings.

## Remaining

Normal source integration and current-head review are recorded separately in the
current work memo. Windows cold/warm execution, complete normal-versus-coverage
equivalence, actual test reduction, broader trusted reuse and before/after total
time comparisons remain incomplete. This slice does not complete Issue #343,
Ready CI, human acceptance, publication or cleanup.
