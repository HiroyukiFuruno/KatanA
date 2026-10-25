# Post-publication cleanup production connection

Issue: [#335](https://github.com/HiroyukiFuruno/KatanA/issues/335).

## Execution order

`run-release-cleanup.py` first checks that source and controller are roots of the
same common Git repository, before writing a receipt or acquiring a session.
It then verifies the public repository, stable Release, published tag and source
ancestry. The actual tag commit must equal the explicit `--published` identity
before ff-only default synchronization. Branch eligibility is audited again.

The existing reference owner is retained by its installer. Remote cleanup also
retains the existing pre-push owner and its ordinary checks. Both owner wrappers
and backups are checked for changes before deletion. Native local deletion uses
`git branch -d`; remote deletion uses ordinary `git push origin --delete`, never
force or a hook bypass. Each command receives one private scoped intent.

Remote state already absent is not a failed push converted into success:
an explicit native advertisement establishes absence before attempting deletion.
Transport failures and a different remote SHA are rejected. A live repository
metadata read confirmed `delete_branch_on_merge=true`, so absence is required for
the normal merged-PR path, not merely an optimization.

## Worktree scope

[Git worktree documentation](https://git-scm.com/docs/git-worktree) describes prune
as repository-wide and provides no per-worktree path parameter. Therefore this
caller refuses any nonempty linked-worktree registry before deletion. It executes
native prune only for a single audited default checkout with an empty registry,
and rechecks that scope afterward. Unknown, active, locked or evidence-bearing
metadata is not removed. Developer-local removal of an explicitly owned clean
worktree remains a separate audited branch-hygiene operation, not a force flag.

The present two-worktree development repository is not used as a cleanup fixture;
its active release branch and old master runtime remain protected. There is no
fixed release-worktree fallback or direct master edit to activate shared hooks.

## Workflow connection

`cleanup_default` depends on successful `preflight` and `publish`, including the
existing artifact attestation and external registry synchronization. It checks
out the default branch with full history, provisions the ordinary owner tools,
installs Lefthook, and resolves one live merged release PR. It does not replace
the branch tip with the tag/merge SHA. The existing manual-target choices and
all-platform publication predicate are unchanged.

The job uses Ubuntu with native Docker available for the original owner's
Linux/Windows checks, rather than silently dropping those checks on a hosted
macOS runner. JSONL receipts are stored privately in the common Git directory
and uploaded even when cleanup fails. A failed lock release cannot produce a
completed receipt.

## Verification and remaining work

- Native executor, receipt, prune, target and workflow tests are included by the
  existing release Python discovery. They do not mock a public GitHub Release.
- Two actual assertion failures exposed foreign-controller receipt mutation and
  changed reference-owner admission. Repairs add pre-write repository scope and
  reference-owner snapshot comparison. Raw RED logs are retained under `tmp/`.
- Target resolver live GETs selected merged PR #334 for v0.22.41 with its actual
  head SHA, and rejected the open Draft PR #346 for v0.22.42. No deletion followed.
- Full default actionlint v1.7.12 includes ShellCheck. Its actual failures exposed
  inline PR-ref interpolation and shell word splitting in the existing publish
  script; environment data and an argument-preserving asset array repair them.
  The earlier syntax-only invocation with optional analyzers disabled is not the
  final lint evidence.
- The pre-push installer rename-gap was reproduced by native SIGINT immediately
  after replacement: one actual assertion failure. Hardlink retention and atomic
  replacement repair it; the interrupted installer preserves the original owner,
  whose rejection still prevents an ordinary temporary-remote push. The dedicated
  25-case installer/dispatcher group passed in 10.694 seconds.
- Final frozen cleanup discovery: 126 passed in 77.977 seconds, actual exit 0,
  raw `tmp/cleanup-production-caller-final-tests-2026-10-15.log`.
  Final pre-push installer discovery: 11 passed in 3.444 seconds, actual exit 0,
  raw `tmp/cleanup-installer-final-tests-2026-10-15.log`.
- Original release-flow contract: 10 passed. Default actionlint v1.7.12, including
  ShellCheck, exited zero; raw `tmp/cleanup-workflow-actionlint-final-2026-10-15.log`.
  All 18 changed/new Python files passed syntax, 200-line/file and 30-line/function
  checks. Final limited independent review found no P0/P1 or mandatory P2.
- Normal source integration: installer repair `3ea29e3`, execution caller
  `567833d`, workflow/target connection `ac2610f`; all signed normal commits
  exited zero.
  Normal push is not public success evidence yet.
- Actual public end-to-end cleanup, Ready CI, packaged acceptance, release assets,
  Issue closure, OpenSpec archive and developer-local cleanup remain unproved.
