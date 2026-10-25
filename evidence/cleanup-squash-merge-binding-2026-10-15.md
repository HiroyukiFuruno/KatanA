# Merge identity and branch-tip cleanup

## Finding

PR #346 review `5529761247`, P1 comment `4280360962`, identifies the lost
`merge_commit` binding in published cleanup. A squash/rebase branch tip need not
be an ancestor of the published merge. The old workflow at `04dbeea9` passes only
the branch tip. A source-contract assertion against that immutable workflow
failed with `old workflow discards merge_commit and passes only branch head`.
This is a wiring RED, not a claim that this Draft PR was publicly released.

## Repair

The workflow carries both the resolved PR number and merge commit. Publication
admission re-fetches the fixed-host merged PR and verifies repository, branch,
head, merge and PR number. The published remote tag must contain the verified
merge identity and still equal the requested publication source. Default
synchronization and retention checks remain mandatory.

Branch deletion continues to compare the actual branch tip, not the merge SHA.
After verified PR binding, a local squashed branch uses ordinary native
`git update-ref -d <explicit-ref> <expected-old-tip>`. The existing reference hook
and locked prepared-transaction check still run. The legacy ancestry path retains
`git branch -d`. No force, hook bypass or unmerged tree-equality fallback is used.

Remote schema 3 binds the merge/PR pair and revalidates it live in the pre-push
boundary before accepting the exact advertised deletion tuple. Local schema 1
and remote schema 2 remain compatible. Missing, partial or malformed bindings
are refused. Used branches and changed tips are retained.

## Evidence and limits

- Main focused cleanup discovery: **136 tests passed in 98.220 seconds**, session
  `40938` actual exit 0. Raw: `tmp/cleanup-squash-final-targets-2026-10-15.log`.
- Actual temporary native Git squash has identical final trees but non-ancestor
  branch and merge tips. The merge passes publication ancestry, the branch tip
  does not; guarded CAS deletes only the expected local ref through the existing
  owner. Owner failure, changed tip and used branch retain it.
- A fresh default-only clone lacks the already-deleted branch object; verified
  merge ancestry remains usable without inventing that object or branch.
- The initial six-test fixture run failed because its tag creation inherited
  host signing. The fixture now explicitly configures unsigned fixture tags;
  the production signing settings and timeout were not changed.
- Live read-only PR #334 binding returned merge
  `af55949c4ee7ddad414ed93fa360a67e9464acf4` for exact head
  `6b069ac6618bdfbd33e517cdaa888ccd763c993b`.
- Default actionlint on both workflows passed, actual session `51876` exit 0.
- Remote/public verification was subsequently unified through the same exact
  branch/head/merge/number matcher. The remote boundary's eight tests passed in
  7.586 seconds, actual session `26063` exit 0. The resolver already verifies the
  explicit endpoint's PR number, so this consolidation is not misreported as a
  previously exploitable wrong-number runtime RED. Final independent read-only
  review found no remaining P0/P1 or mandatory P2.

The repair was integrated by normal signed commit
`a5aa2a18437a41ce3ba3518e3e679d340f8453c4` (session `60350`, exit 0).

No real repository branch/worktree was deleted, no actual publication cleanup
has run, and no public release or full Issue #335 completion is claimed.
Normal push, per-thread reply/resolve and fresh review remain required.
