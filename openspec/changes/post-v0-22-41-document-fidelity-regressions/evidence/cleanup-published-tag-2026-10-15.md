# Cleanup publication-to-source binding

Normal signed integration: source `ad88b732` (session58336 exit0), native
transaction observation `1782a9f4` (session22707 exit0). Both remain local until
the normal push finishes; neither proves complete cleanup or publication.

## Scope and limits

Issue #335 requires post-publication cleanup, not deletion based only on a release
name. `published_audit` now checks the existing clean default controller and
binds the audited full branch commit to the actual remote release tag before
default synchronization. This is an admission component, not completed cleanup.
Local/remote deletion, retention receipts, scoped prune, workflow invocation and
production hook activation remain unfinished. No real repository was synchronized
or cleaned in this slice.

`release_cleanup_tag.py` reads exact lightweight/annotated tag advertisements,
fetches the advertised object without changing local refs or `FETCH_HEAD`, checks
the advertisement again, verifies canonical commit identity and requires the
cleanup source to be included in the published tag graph. Fetch explicitly
disables submodule recursion, pruning, automatic maintenance and commit-graph
writing. The caller still validates fetch/push repository identity and URL rewrite
configuration. This does not verify all distributed assets or native acceptance.

## Actual targeted verification

- Initial new helper tests: session16419, exit0, 6 tests. The helper and these
  initial tests were introduced together; this is not represented as an old-code
  behavioral RED.
- Main's expanded real-Git suite: session46683, exit0, 16 tests, 9.241 seconds.
- Prune configuration concern: session68084, exit0, both literal/named remote
  retention tests passed on the pre-hardening helper. No reproduced prune defect
  is claimed. Explicit suppression narrows future side effects.
- Populated submodule regression: session25828, exit1 at the production fetch
  call with `fetch.recurseSubmodules=true` and an unavailable module remote.
  Adding only `--no-recurse-submodules` made session7033 exit0. The test uses
  actual parent/module Git repositories and transport, not a mocked fetch.
- SHA256 regression: session51967, exit1, `CleanupRefused not raised` for a
  40-character prefix of the real 64-character commit. Native canonical commit
  equality repairs the helper boundary. The existing production caller's full
  branch-OID check already prevented this prefix from becoming deletion proof.
- Final scoped run: session38914, exit0, 20 tests, 14.557 seconds. Raw output:
  `tmp/cleanup-tag-current-tests-2026-10-15.log`.

The final 20 comprise 12 tag/graph/retention tests, two actual reference-transaction
tests and six existing destination-binding tests. A fresh consumer proves tag
objects are really fetched without creating refs; an existing `FETCH_HEAD` is
preserved. Missing/older/non-commit tags cannot admit the candidate.

## Native deletion observation

The limited agent test observed Git2.56.0 `branch -d` prepared/committed callbacks
with zero old/new OIDs, rather than the actual nonzero branch tip. Prepared-hook
rejection retained the branch and `HEAD`. Main also reran these real-Git tests.
The regression accepts either a zero or actual old OID for compatible Git
versions, but requires the delete notification and rejection behavior. It is not
an expected-OID atomic-CAS proof and does not install any production hook.

All repositories and hooks used by the new tests are owned temporary fixtures.
No new worktree, stash, forced deletion, hook bypass, application interaction,
user-setting edit or sibling implementation was performed.
