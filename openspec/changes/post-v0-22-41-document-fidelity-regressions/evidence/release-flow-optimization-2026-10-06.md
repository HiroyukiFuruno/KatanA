# Release flow optimization — 2026-10-06

## Scope and reference

The user requested removal of duplicated checks, using KRR's release flow as a reference. KRR's pre-push hook was inspected read-only: it runs its normal gate once and binds the operation to the reviewed HEAD. No KRR source was changed.

## Changes

- Pre-commit Clippy now uses the existing impacted-package target instead of full-workspace lint.
- `just pre-push` now matches the actual release pre-push hook (`just check`), rather than unexpectedly running `check-full`.
- PR readiness already owns release preflight. CI behavior was not duplicated before this change; comments and executable regression checks prevent future direct or multiline duplication.
- The workflow preserves ordinary release-branch commits, Draft review, thread repair/reply/resolve, Ready promotion and current-HEAD required checks. It no longer instructs squash/reset or renewed merge approval for an already authorized release.
- Publication, asset/checksum verification and acceptance remain completion requirements. Remote branches are not implicitly deleted.

## Verification

- `python3 scripts/release/test-release-flow-contract.py`: four parser regression cases pass; current repository ownership checks pass.
- The negative cases reject direct recipe and multiline script invocation; comments and unrelated next-step fields do not cause false positives.
- `python3 -B -m unittest discover -s scripts/ci -p test_ci_resource_contract.py -v`: six existing preservation checks pass.
- `zsh -n scripts/release/preflight.sh`: passes.
- `just --dry-run pre-push`: resolves to the existing native/impacted tests and Linux/Windows gates, without coverage being implicitly added to each push.
- `git diff --check`: passes.

## Boundaries

This does not prove the latest application graph passed all gates or that the release is published. Full release coverage, fixture acceptance, supply-chain and three-OS checks remain required. Coverage exclusions differ from workspace tests, so those tests were not removed from CI. No validation thresholds, test ignores or hook bypasses were introduced.

A worker initially edited the primary checkout rather than the designated release worktree. Its exact changes were moved to the existing release worktree and its own primary-checkout changes restored. Main verified master clean and stash zero; no new branch, worktree or stash was created.
