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

PASS for the resource-default diff and its regression contract. The exported values match existing CI; a caller can opt into symbols without changing test behavior. No broader quality-gate bypass or profile-performance change is introduced. Commit/push and the full rerun remain independent pending checkpoints.
