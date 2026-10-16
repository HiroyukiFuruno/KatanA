# Genuine Issue reference verification slice

Issue: https://github.com/HiroyukiFuruno/KatanA/issues/335

The optional `--verify-live` commit-message checker now performs bounded, read-only GitHub lookups of extracted same-repository Issue references. It accepts at least one genuine Issue, rejects pull requests returned by the Issues API, validates repository/number/URL/state identities, and fails closed on transport, authentication, deadline or malformed-response errors. It uses a fixed github.com hostname, at most32 unique candidates and a shared30-second deadline. Issue bodies and authentication diagnostics are not executed or printed.

Actual session11809 passed19 payload/boundary tests and16 syntax/Git contract tests. Real GitHub checks accepted Issue335, accepted missing-then-valid references, rejected PR346 and nonexistent Issue2147483647, and still queried the fixed hostname when GH_HOST was changed. Existing release-flow contracts9 and Python/shell syntax checks passed. Pure tests are registered in the existing preflight without duplicating the quality gate.

Evidence: `tmp/issue335-live-acceptance-2026-10-14.log` in the existing release worktree. The initial missing extraction-interface test failed before implementation and is retained separately; it is not a runtime product failure.

The optional checker was integrated through the normal hook in signed commit `0bef2c7b` (session78880, exit0). A further real GitHub check accepted PR346 followed by genuine Issue335 without accepting the PR itself. A real refused local transport returned controlled exit2 with one diagnostic line rather than succeeding or leaking raw authentication output (session75719, exit0 for both acceptance assertions). Logs: `tmp/issue335-real-pr-then-valid-2026-10-14.log` and `tmp/issue335-real-transport-failclosed-2026-10-14.log`.

The default offline hook behavior is preserved in this slice. Mandatory live-hook connection, pre-push dispatcher ordering/stdin/range handling, downstream manifest/lock/publication evidence and release cleanup enforcement remain unimplemented. This slice does not complete or close Issue335 and never permits skipping normal checks.
