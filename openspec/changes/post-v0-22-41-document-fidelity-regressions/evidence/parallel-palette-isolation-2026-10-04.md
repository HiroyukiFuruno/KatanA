# Parallel palette workspace isolation — 2026-10-04

## Cause and controlled repair

The coverage-instrumented parallel UI suite failed four workspace-loading assertions (139 passed, four failed, two existing ignored) after public checkpoint a91469ff. The same full suite failed without concurrent release compilation. A single navigation test and the five lint-review tests succeeded independently, so compiler contention was not the cause.

The two command-palette UI harnesses used isolated settings but retained the real user's default global workspace JSON repository. `KatanaApp::new` restores that history and starts background filesystem scanning, which persists beyond these short palette tests and interferes with subsequent workspace tests. Common harnesses already isolate global history before app construction.

Both palette harnesses now retain a unique `tempfile::TempDir` for the entire test and use actual JSON settings and workspace repositories within it, configured before app construction. This adds no mock, product-source change, deadline increase, test exclusion, or coverage-policy change.

## Runtime evidence

- Original standalone instrumented suite: 139 passed / four failed / two existing ignored, exit101. Log: `tmp/workspace-instrumented-standalone-control-2026-10-04.log`.
- Diagnostic exclusion of palette tests removed all four workspace failures; the unrelated relative-path overlap check failed only because direct binary execution used the workspace root rather than Cargo's package working directory. This exclusion is not part of any gate or committed recipe.
- Repaired official Cargo/llvm-cov entry, all145 tests with two threads: 143 passed / zero failed / two existing ignored, 24.43s, exit0. Log: `tmp/workspace-palette-isolation-green-2026-10-04.log`.
- Negative control retaining unique settings directories but removing only the two global-repository assignments: the identical full official entry returned 139 passed / the same four failed / two existing ignored, 32.05s, exit101. Log: `tmp/workspace-palette-isolation-negative-2026-10-04.log`. Both assignments were then restored.
- Restored full official entry: 143 passed / zero failed / two existing ignored, 24.84s, exit0. Log: `tmp/workspace-palette-isolation-restored-2026-10-04.log`. Format, AST23 and impacted Clippy all passed. Self-review found no product/API/polling-budget change; TempDir lifetime outlives each harness, including its save-on-drop. Five raw profiles from direct-binary diagnostics were preserved under ignored `tmp/coverage-palette-diagnostics-2026-10-04/`, not left as untracked root files.

The existing 100-step/yield/2ms helper and every test/assertion remain unchanged. Full fresh coverage, ordinary push hooks, current review, original-file/packaged acceptance and public release still require their own completed evidence.

## Resource and release boundaries

The owned fresh release-worker build was interrupted with exit130 after host free space fell to3.5GiB, before the fresh screenshot runner was built. This is not a successful build or KDV0.5.11 acceptance. No sibling process was terminated and no release output was removed. Checkpoint a91469ff's normal push succeeded and is synchronized with its upstream; the new isolation repair is not yet public at this evidence checkpoint.
