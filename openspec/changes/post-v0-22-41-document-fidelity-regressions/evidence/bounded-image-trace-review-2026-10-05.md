# Bounded image-source trace review

## Current finding

PR 346 current review on c10254a3 reported P2 thread `PRRT_kwDORm09y86pCPUw`: ordinary and linked image layout traces format the entire source on every frame when DEBUG is exactly `true`. Embedded SVG sources can therefore produce repeated large stderr output. This is distinct from the unresolved manual-target publication policy finding.

## Repair and source review

- Both trace call sites now pass the same borrowed `BoundedSource` display wrapper. Rendering still receives the original source unchanged.
- Formatting examines at most 129 source characters and emits at most 192 UTF-8 bytes, including a truncation marker. It does not allocate or scan the full source.
- Ordinary Unicode characters are preserved. Newline, carriage return, tab and other control characters are escaped without splitting UTF-8.
- `DebugLog::write` formats its arguments only inside the existing exact `DEBUG=true` condition. Disabled diagnostics do not traverse the source.
- Formatter failures propagate without fallback. No public API, rendering input, diagnostic enablement, acceptance deadline, quality score or lint/coverage exclusion changed.

## Reproduction and verification

The child completed five focused tests, but its earlier whole-UI baseline failed on unrelated missing Office workers. That baseline is not evidence for this P2.

The main agent independently replaced only the diagnostic formatter with the original full-string behavior (`formatter.write_str(self.source)`) in the same private module, keeping its five regression tests unchanged. The standalone module harness executed the tests: two passed and three failed on the large data URI, over-limit input and control-character output. The baseline compiler's unused-helper warnings were expected and are not a successful strict lint gate. The bounded implementation was then restored immediately; no baseline code remains.

The stable restored module has nine tests, covering empty/short/Unicode input, both truncation limits, all U+0000 through U+009F control-byte length calculations, and failures during content and truncation-marker writes. Independent compilation with `-D warnings` and execution succeeded: nine passed, no ignored tests. Raw receipts:

- `tmp/bounded-image-trace-red-2026-10-05.log`
- `tmp/bounded-image-trace-green-2026-10-05.log`

The canonical `with-office-test-worker.sh` → `just T=render_inline::trace::tests test-specific` entry completed on the stable final module: nine passed, 1831 filtered across 39 suites, exit 0. The earlier seven-test run is retained separately and is not substituted for this final run (`tmp/bounded-image-trace-native-final-2026-10-05.log`).

The existing aggregate coverage configuration excludes `html_renderer/.*`. Therefore the same private helper was additionally compiled with the installed Rust toolchain's `-C instrument-coverage` and `-D warnings`, executed through its nine tests, and measured with that toolchain's LLVM tools. Actual coverage: 209/209 regions, 15/15 functions and 107/107 lines executed, all 100%. LLVM reported zero branch counters, so this is not an independently measured branch percentage. Raw report and JSON: `tmp/bounded-image-trace-coverage-2026-10-05.log` and `tmp/bounded-image-trace-coverage-2026-10-05.json`. Stable source SHA-256: `6c6fe10820d9a973cff98a89e5ca927cf3b0d00df2a5717ca8f3358dbde592dd`.

The first normal commit was correctly rejected by strict Clippy (`explicit_counter_loop`) and the existing AST Japanese-input rule. The loop now uses `enumerate`, and Unicode test values use escapes with identical decoded input. Neither rule was disabled. The post-lint source SHA-256 is `3f2bdce75534b6cfedd2d18854dec65273c42860a969364a8c19d5797384584e`; repeated standalone tests passed nine, and actual instrumented coverage was 210/210 regions, 15/15 functions and 106/106 lines, all 100%. Receipts use the `bounded-image-trace-coverage-post-lint-2026-10-05` basename; the earlier proof is historical, not transferred to this source.

The canonical real-worker/workspace filtered test was also rerun after those lint repairs and exited 0: nine passed, 1831 filtered, 39 suites (`tmp/bounded-image-trace-native-post-lint-2026-10-05.log`). Source behavior, UTF-8 boundaries and error propagation were reviewed again without introducing fallback, unchecked input, ignored tests or new exemptions.

The normal source commit passed its strict Clippy and AST hooks after those repairs: `28cd7ec2`, with raw receipt `tmp/bounded-image-trace-source-commit-retry-2026-10-05.log`. Aggregate full coverage, normal push and individual thread reply/resolve remain separate checkpoints. No existing coverage exclusion or threshold changed.

## Release boundary

The successful c10254a3 CI and packaged arm64 startup receipt predate this Rust change. They are retained as historical evidence and do not verify a rebuilt candidate containing this repair. Packaged document input/normal close, independent four-axis scores, all native distribution targets, clean-machine acceptance and release cleanup remain unfinished.
