# Issue-reference commit gate — #335

## Implemented component

The `commit-msg` hook now runs the repository Issue-reference validator on non-master branches. The existing pre-commit and pre-push gates remain unchanged. `lefthook install` synchronized the actual local hooks without editing master product files or bypassing a hook.

The validator requires a positive `#number` or an exact same-repository GitHub `/issues/number` URL; a foreign URL, PR URL, URL fragment, zero, or suffix-attached number does not qualify. Case differences in GitHub owner/repository identity are accepted. This first component checks syntax and repository scope, not live Issue existence or downstream migration evidence.

## Actual verification

- Delegated unit tests before implementation failed because the module did not exist; this is an initial feature RED, not a reproduced product regression.
- Main added a suffix-attached reference regression: `#335x` was accepted incorrectly, producing an assertion failure (exit 1). The tightened token boundary rejects it.
- Final `python3 scripts/release/test-commit-issue-reference.py`: 15 tests, exit 0. Three use real temporary Git repositories: release branch rejects missing reference (exit 1), accepts `#335` (exit 0), and master remains unchanged (exit 0). Git-local environment variables are removed to avoid mutating the caller repository.
- `bash -n scripts/release/check-commit-issue-reference.sh` and `git diff --check`: exit 0.
- `lefthook install`: exit 0; pre-push, commit-msg, and pre-commit synchronized.

## Remaining Issue #335 contract

Live Issue-existence validation, downstream public dependency/migration/manifest/lock/validation evidence enforcement, explicit existing-hook-first dispatcher order, and public-release-only safe branch/worktree cleanup with release-workflow integration remain unfinished. This commit gate is not closure of the entire Issue, and no historical commit was rewritten or retroactively rejected.
