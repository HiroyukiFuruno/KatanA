# Mandatory genuine-Issue commit hook slice

Issue: https://github.com/HiroyukiFuruno/KatanA/issues/335

The existing non-master commit-msg owner now invokes the existing checker with `--verify-live`. It requires a genuine same-repository GitHub Issue, rather than accepting a syntactically valid PR number. The master publication-generation exemption is unchanged. Existing pre-commit/pre-push heavy gates and dispatcher installation are untouched.

## Actual regression evidence

The previous wrapper-direct integration fixture was replaced, not duplicated, by a real isolated Git repository with an actual commit-msg hook invoking the production wrapper. Every fixture Git child removes inherited GIT variables. Fixture-only local identity, unsigned fixture commits and hooksPath never alter real common Git config or hooks.

- Old-wrapper runtime RED: `tmp/issue335-hook-wiring-old-wrapper-red-2026-10-14.log`, exit1, 17 tests with two assertion failures. A genuine reference did not emit live-verification output; PR346 was incorrectly accepted as an Issue.
- Worker GREEN: `tmp/issue335-hook-wiring-green-2026-10-14.log`, 17 tests/exit0; existing pure live-client tests19/exit0.
- Main final: `tmp/issue335-mandatory-hook-main-final-2026-10-14.log`, 17 tests/exit0 under real-worktree GIT_DIR/GIT_COMMON_DIR/GIT_WORK_TREE/GIT_INDEX_FILE. Genuine Issue335 and master commits create new HEADs; missing references and PR346 are rejected with HEAD unchanged. Main pure tests19/exit0: `tmp/issue335-mandatory-hook-main-pure-2026-10-14.log`.
- Before/after real worktree HEAD, index tree and core.bare assertions passed. Bash syntax and diff checks passed. No mocked gh, test skips, timeout extensions or real hook installation were used.

Preflight retains one acceptance case for the genuine reference and one for PR rejection, plus the existing pure tests. It does not add a second wrapper-positive path or repeat heavy build/test gates. Live lookup failure remains a rejection, not an offline fallback.

The repository was live-confirmed public. The [official Get an issue contract](https://docs.github.com/en/rest/issues/issues#get-an-issue) permits public-resource reads without the otherwise required Issues-read permission. Ready/CD checks provide GH_TOKEN; their actual current-head cloud execution remains unverified while the PR is Draft. No permissions were expanded or authenticated CI success inferred from a local run.

## Still unfinished

Read-only independent high review found no P0/P1. Ordinary signed source commit `018edb50e3e4da94afc55180a6281812ef710f0b` completed through normal hooks (session77715 exit0). Optional P2 additions for a nonexistent-number hook case and mixed PR/Issue hook references are not product defects: existing pure and actual CLI acceptance cover those client boundaries, but that is explicitly not an assertion that both were run through the new real hook test. They remain optional coverage improvements rather than blocking or expanding the current slice.

This is only mandatory commit-msg Issue existence. Issue335 still requires existing-hook-first dispatcher/range activation, downstream public version/API migration/manifest/lock/verification evidence and safe release cleanup. No Issue closure, current-head formal-review completion, full coverage, packaged acceptance or public release is claimed. Normal evidence commit/push is the next step.
