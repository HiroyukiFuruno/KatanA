# Owner-preserving reference hook installer

## Scope

Issue #335 production installer entry, not full cleanup orchestration. The CLI
uses the existing Git common-hooks scope and all-registered-worktree runtime
parity checks before changing a hook. Existing pre-push checks are extracted
without changing their policy or public import names.

The reference installer preserves an executable owner's bytes/mode in an
exclusive backup link, handles an absent owner, refuses unknown wrappers and
backup collisions, and serializes installations with a scoped lock directory.
It never changes hooksPath to bypass the common hooks directory.

## Verification

- Source normally committed as `ba2d591eb4b2cb6e2da2874517ea9d7a95046e54`,
  session 46302 exit 0, signature G; not yet public at this record.
- Real temporary-Git installer tests: 81419 exit 1 exposed an unknown V0 wrapper
  being treated as an owner. Marker-family rejection repaired this boundary.
- 86734 exit 1 reproduced owner backup loss after real `os.replace` followed by
  SIGINT. Profiling synchronizes the signal after the native operation; no
  operation is mocked or replaced. Cleanup now removes an added backup link only
  when hook and backup still identify the original inode.
- Both installer families: 98087 exit 0, 18 tests, 5.700 seconds.
- Final reference installer tests: 14760 exit 0, 8 tests, 3.931 seconds, including
  an actual commit after interrupted installation to establish continued use.
- Release Python discovery: 77867 exit 0, 302 tests, 126.340 seconds. Raw log:
  `tmp/reference-installer-final-tests-2026-10-15.log`. This used the same final
  production source; the last post-interrupt commit assertion was placed in its
  correct fixture afterward and separately verified by 14760.
- Main AST validation: five changed Python files parse; lengths 42/123/101/28/163,
  maximum function span 23. All satisfy file <=200/function <=30.
- Fresh independent read-only review found no remaining component P0/P1 or
  mandatory P2. Existing pre-push wrapper rationale remains intact in the diff.

## Current repository safety

Only the read-only runtime validator was invoked against this repository.
It raised the expected incompatible-worktree error for the old master runtime
(`downstream_lock_snapshot.py`). Raw log:
`tmp/reference-installer-current-runtime-rejection-2026-10-15.log`.
No production installer invocation, shared-hook mutation, master edit, application
operation, settings edit, or branch/worktree deletion was performed.

## Remaining release scope

Public admission/default synchronization, audited operation lifetime and
serialization, safe local/remote deletion and retention, scoped prune, and
mandatory post-publication workflow invocation remain unfinished. Runtime parity
must hold before activation; a fixed release-worktree fallback is prohibited.
These tests do not prove packaged acceptance, current-HEAD required CI, or full
Issue #335/release completion.
