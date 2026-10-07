# Local verification symbol profiles

## Cause and scope

The normal push for `c3b2c0d1` regenerated native test binaries with `-C debuginfo=2`, unlike the existing CI resource contract's symbol-only profiles. Free host space fell from10GiB to2.3GiB while compiling; the verified owned Cargo62824 was interrupted. Its already-started children finished before space reached304MiB. After confirming no active Cargo/rustc/linker children, the owned lefthook59845 was terminated because it did not return after the Cargo interruption. The push command exited1 and did not publish `c3b2c0d1`; it is not recorded as a passing gate.

The existing Justfile now exports overridable defaults matching CI:

| Profile | debug symbols | strip |
| --- | --- | --- |
| dev | 0 | none |
| test | 0 | none |

No optimization level, debug assertion, overflow check, test registration, coverage instrumentation, exclusion, platform target, or release profile was changed. Direct Cargo users can keep normal debug defaults; explicit environment overrides of Justfile profiles remain available for debugger sessions. The default applies to normal local hooks and coverage through their existing Justfile entrypoints, preventing a forgotten shell export from recreating the large-symbol graph.

## Evaluation

The new local-profile contract fails against the old Justfile (`tmp/local-symbol-profile-contract-red-2026-10-04.log`, exit1). After the four exports, the unchanged CI resource tests plus the new contract all pass: `just test-ci-resource-contract`,5 tests, exit0. Actual Justfile evaluation returns dev/test debug0 and strip none; an explicit dev debug2 override returns2. Diff whitespace validation passes. The final actual native/full-coverage/platform hook rerun remains pending.

## Scoped generated-cache recovery

At the signed source checkpoint `c3b2c0d1`, official `cargo clean --frozen --profile dev -p katana-ui --dry-run` identified3994 generated files/6.8GiB. With no active Cargo/rustc/linker processes, the matching real command removed3989 files/6.2GiB and restored5.3GiB free. Source, locks, Git history, evidence, release outputs, sibling repositories and the separate screenshot cache were not deleted. Cargo can regenerate the removed dev artifacts. Core dev artifacts were only inspected by dry-run at this checkpoint; their removal is not yet claimed.

## Self-review

The normal push at `a5c3aed1` passed native tests (UI1030 total/2 existing ignored, parallel143, serial18 and real export13), then failed Linux linkage with Bus error. After the owned container ended, a fresh volume observation confirmed148GiB backing disk100% full/zero available. Docker Compose had not forwarded the existing Justfile profile exports, so it compiled `[unoptimized + debuginfo]`. Linux/Windows Compose now forwards all four optional environment values; both real images report0/0/none/none under the formal entrypoint defaults. Direct Compose callers without those values retain Cargo's defaults. New forwarding assertions fail against the old files and all six resource contracts pass after repair.

With the clean signed source checkpoint `a5c3aed1` and no active containers, the official Linux UI dev clean dry-run identified272 files/27.9GiB. The real package-scoped command removed267 files/25.7GiB, restoring17GiB available in the backing filesystem. Only regenerable `katana-ui` dev outputs in `ci_katana-linux-target` were removed; source/history/release/screenshot and sibling volumes remain untouched. This is resource recovery, not a successful Linux test or publication. Fresh normal push and changed-graph coverage remain pending. Supply-chain advisories/bans/licenses/sources pass without policy changes. Published `f79b9e5d` has all cloud checks SUCCESS and review5978517827 reports no major issues; those results do not cover the newer local commits.

After signed checkpoint `306531fe` and confirming no active Rust compilation, the official core dev dry-run identified15510 generated files/10.9GiB. The matching package-scoped Cargo clean removed15504 generated files/10.7GiB, restoring16GiB free. This supersedes the earlier dry-run-only checkpoint; source/release/screenshot/sibling outputs were preserved. The next normal push compiled without debuginfo2 but exited1 on the native parallel workspace polling failure documented in `parallel-workspace-budget-2026-10-04.md`; it did not publish the local commits or reach Linux/Windows gates.

PASS for the resource-default diff and its regression contract. The exported values match existing CI; a caller can opt into symbols without changing test behavior. No broader quality-gate bypass or profile-performance change is introduced. Commit/push and the full rerun remain independent pending checkpoints.
