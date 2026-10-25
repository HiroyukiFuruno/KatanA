# Native invocation inventory diagnostics

## Scope

Issue #343 now has a collect-only production entry at the normal workspace-test
boundary and the four existing coverage boundaries. This is not test-set
equivalence, a trusted result receipt, a CI skip, or completion of the issue.

The normal entry runs the same Cargo test selection once, with Cargo JSON build
messages enabled. It retains the requested and executed argv and original exit
status. Only compiler artifacts built with `profile.test=true` are candidates for
native `--list` and `--list --ignored`; arbitrary target-directory scans and an
additional `cargo test --no-run` are not used. The parser stops interpreting Cargo
messages at `build-finished`, before test stdout can impersonate build messages.
The protocol is documented in the [Cargo Book](https://doc.rust-lang.org/cargo/reference/external-tools.html).

The diagnostic records source HEAD/tree, clean status and its digest, both lock
digests, selected environment inputs, tool versions and worker digest. Raw native
listings, their digests, parsed cases and binary digests at listing are retained.
The original coverage exclusions, fixture skip, worker, job/thread settings and
coverage thresholds remain unchanged. Each coverage phase has its own argv.

## Deliberate incomplete status

Every manifest has `complete=false` and `reuse_authorized=false`. Doctest
inventory, llvm-cov artifact provenance, a hermetic environment and binary
identity at actual test execution are not established. The POSIX native-list
collector is not Windows support. Failure or absence of those observations must
not become authorization to remove any existing normal or platform test.

## Verification

- Final focused discovery: **34 tests passed in 4.377 seconds**, actual session
  `70087` exit 0; raw `tmp/native-invocation-inventory-final-2026-10-15.log`.
- Real temporary Cargo/Git fixtures execute successful and failing tests. The
  recorder preserves Cargo exit 0 and 101 respectively, captures ignored tests,
  rejects post-build fake artifact output, and never claims doctest equivalence.
- Existing CI resource contract: **10 tests passed**, exit 0.
- Bash and Zsh syntax checks passed; workflow actionlint with its default
  ShellCheck passed, actual session `51876` exit 0. Raw combined log:
  `tmp/cleanup-squash-inventory-workflows-2026-10-15.log`.
- A pre-existing SC2155 assignment/export warning in the touched workflow was
  repaired by declaring and assigning separately, not by disabling ShellCheck.
- Independent read-only review found no P0/P1 or mandatory P2 in this diagnostic
  slice. Main reviewed the diff and all added files; no new function exceeds
  30 lines and no new file exceeds 200 lines.

These are targeted checks. Current-HEAD normal push, Ready CI, measured duplicate
reduction, Windows cold/warm comparison and public release remain separate work.
The source was integrated by normal signed commit
`da9cc55df2f2b0383e68ce7a3c6f07c7242d5dfb` (commit session `88114`, exit 0).
