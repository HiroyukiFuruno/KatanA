# Git no-fetch capability prerequisite

Issue: https://github.com/HiroyukiFuruno/KatanA/issues/335
Review: https://github.com/HiroyukiFuruno/KatanA/pull/346#discussion_r4264041580

Source: `9319ff4e987f6edcbb1817d3c6635610b0711047`, normal signed commit.
Final HIGH diff review confirmed no remaining P0/P1 in this prerequisite slice.

The current-HEAD review identified that Git 2.44 and earlier do not recognize
the global `--no-lazy-fetch` option. The supported development/release evidence
toolchain now explicitly requires Git 2.45+ with that actual capability. This
is not an application runtime requirement and does not relax evidence checks.

The existing macOS setup checks the selected Git after installation, and the
existing readiness/CD preflight owner checks it once before test discovery.
It rejects unsupported tools with an upgrade/PATH instruction. The standalone
Git binding error also retains a safe typed error and a toolchain hint.
No unsafe fallback or implicit-object download is introduced.

## Actual verification

- Official kernel.org Git 2.43.0 source archive was checked against the official
  checksum listing: SHA-256
  `ed238f5c72a014f238cc49fe7df4c6883732a3881111b381c105e2c5be77302f`.
  A temporary release-worktree-only build exited 0. It was not installed into
  the system or used to modify shared Git metadata. This checksum comparison
  is not a claim of independent PGP signature verification.
- Actual Git 2.43 capability check: exit 1 with an explicit upgrade instruction.
  The direct binding test under this unsupported tool also fails with the typed
  prerequisite message; this is not a supported-tool positive test success.
- Actual Git 2.43 selected under `zsh -f`: release preflight exit 1 before test
  discovery. The initial ordinary-zsh attempt reset PATH through user startup
  configuration and used the current Git instead. Its log is preserved, but
  it is not evidence of old-Git rejection. That run exited 1 on genuinely
  incomplete OpenSpec acceptance tasks; it is not a full gate success.
- Current Git capability: exit 0. Single final discovery: 62 tests, 4.817
  seconds, exit 0, retaining real SHA-256 and offline partial-clone binding,
  graft/shallow rejection, and native push metadata tests.
- Bash/Zsh syntax and `git diff --check`: exit 0. The added configuration tests
  assert one preflight call before discovery and one setup call after Git
  installation. They do not emulate Git or claim setup installation occurred.
- Independent HIGH review detected that adding the capability call exceeded
  the preflight file's 200-line limit. Removing two blank lines restored 200
  without removing checks or comments; the three targeted contract tests,
  Zsh syntax and diff checks passed again.

Raw local logs: `tmp/issue335-git243-capability-rejection-2026-10-14.log`,
`tmp/issue335-git243-binding-rejection-2026-10-14.log`,
`tmp/issue335-git243-preflight-no-rc-rejection-2026-10-14.log`, and
`tmp/issue335-git-capability-final-2026-10-14.log`.

Git 2.45 documents the option and its no-fetch contract:
[official Git documentation](https://git-scm.com/docs/git/2.45.0).
Older promisor handling also recognizes `remote.*.partialclonefilter`, making
a narrow configuration-based fallback unsafe:
[Git 2.43 source](https://github.com/git/git/blob/v2.43.0/promisor-remote.c).

Formal integration, normal push, individual review reply/resolve, and fresh
review recovery remain separate. Production downstream admission, trusted
publication/execution binding, cleanup, full Ready CI, packaged acceptance,
and the complete Issue/release are not certified by this prerequisite slice.
