# Remote cleanup admission and intent lifetime

Issue: [#335](https://github.com/HiroyukiFuruno/KatanA/issues/335).

## Implemented boundary

- Schema 2 remote records bind the repository and explicit remote-delete operation;
  schema 1 local records remain unchanged and cannot be cross-replayed.
- The existing mandatory push checker checks a single native advertised deletion
  tuple against the private record after its existing owner has run. It reuses
  the actual push repository context instead of launching another push.
- A remote-only branch needs no synthesized local branch. Existing local tips,
  default protection, worktree use and merged ancestry remain independently checked.
- SHA-256 repositories require the canonical 64-character commit identity;
  a resolvable 40-character prefix is rejected.
- Cleanup-only environment is removed from the existing owner's environment;
  its verification session, ordinary input and exit behavior remain intact.
- A common-repository session serializes cooperative cleanup attempts. Each
  operation gets a private record which is unlinked on success or exception.
  Unknown lock contents and stale locks are retained for inspection, not stolen
  or recursively erased. This does not serialize unrelated native Git writers.

## Actual verification

- `test_*cleanup*.py`: 88 passed in 67.091 seconds, actual exit 0;
  raw `tmp/remote-cleanup-targeted-tests-2026-10-15.log`.
- `test_*push*.py`: 70 passed in 37.318 seconds, actual exit 0;
  raw `tmp/remote-cleanup-push-tests-2026-10-15.log`.
  These groups overlap by eight tests; they are not 158 unique tests.
- Native temporary remote tests: ordinary deletion preserves the default/local
  branch; a remote change inside the pre-push owner after advertisement rejects
  stale deletion; remote-only deletion succeeds without a local target branch.
- Actual failures repaired: scoped empty CLI input incorrectly returned success;
  remote environment reached the owner; a SHA-256 prefix passed absent-local audit.
  Initial missing-module/API errors are implementation baselines, not security REDs.
- Session tests: ten passed in 2.048 seconds, actual exit 0, including a second
  process refusing the active lock, exception invalidation, nested operation
  refusal, distinct nonces and preservation of unknown contents. Independent
  review found that unconditional unlink removed a replacement record; two actual
  assertion failures reproduced this. Expiration now checks issued inode/device/
  mode and bounded payload digest before unlinking, retaining changed records and
  their lock. This is not a claim of complete protection against a malicious
  same-UID actor racing individual filesystem system calls. Raw final rerun:
  `tmp/cleanup-session-lifetime-tests-2026-10-15.log`.

## Not yet proved

Source integrated by normal commits: remote admission 8b79f43 and operation
lifetime c272424; both actual commit exits were zero. The new source is not yet
on the public PR HEAD. Final limited independent re-review found no remaining
P0/P1 or mandatory P2 after the unknown-record repair.

This is not complete Issue #335 or post-release cleanup acceptance. The production
publication caller, installed-runtime activation, local/remote deletion sequencing,
receipts, scoped worktree pruning and mandatory post-publication workflow remain
unfinished. No real repository branch/worktree was deleted. No current Ready CI,
packaged acceptance, public release or completed cleanup is claimed by these tests.
