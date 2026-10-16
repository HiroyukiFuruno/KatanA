# Post-merge CI admission

## Scope

Issue #343 already reuses a completed push workflow for the exact version-bump
merge SHA through `verify-version-bump-ci.sh`. The production caller is
`build-and-release.yml`, step `Verify CI gate for version bump merge`.
Previously that script accepted the workflow conclusion alone. It did not
independently reject a missing or skipped platform job inside a successful run.

The admission reads the live run identity, all job-history pages, and the live
run identity again. It requires the same repository, exact SHA, master push
event, fixed CI workflow path and run throughout, with a stable root attempt.
For each macOS, Windows and Ubuntu `Test and Build` job, its latest attempt must
finish successfully. Older successful platform jobs in the same run can remain
valid when only a failed job is rerun; an older success cannot hide a newer
failure or skip. Duplicate identities, same-name/attempt ambiguity, missing jobs,
future attempts, unexpected matrix legs, incomplete pages and concurrent reruns
fail closed. At least one job must belong to the current root attempt.
The existing wait window and latest-run selection are unchanged.

The initial b271666f slice incorrectly required all jobs to have the same attempt.
The partial-rerun regression reproduced that newly introduced overrestriction.
The follow-up uses `filter=all` and chooses each platform's maximum attempt,
independently of API record order. GitHub explicitly supports [specific-job
reruns](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/re-run-workflows-and-jobs?tool=webui)
and [all-attempt job history](https://docs.github.com/en/rest/actions/workflow-jobs).
No workflow was rerun as part of this investigation.

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
- Follow-up: the partial-rerun test fails on b271666f and all twelve admission
  tests pass after repair, including both record orders and latest-failure
  rejection. Independent high review identified the same P1 overrestriction and
  confirmed the all-history/latest-per-job design. Actual KatanA run 31354395814
  exposes 16 job records across two attempts, confirming API history compatibility;
  it reran all eight jobs, so it is not evidence of a real partial-rerun success.

## Remaining

Normal source integration and current-head review are recorded separately in the
current work memo. Windows cold/warm execution, complete normal-versus-coverage
equivalence, actual test reduction, broader trusted reuse and before/after total
time comparisons remain incomplete. This slice does not complete Issue #343,
Ready CI, human acceptance, publication or cleanup.
