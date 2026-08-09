# Self-Review: v0.22.38 Multi-Format Document Viewer

## Re-review Required

- The previous review was invalidated on 2026-08-09 because KDV owned an egui host and KatanA owned format-specific document runtimes.
- The replacement design requires `KatanA -> katana-document-viewer 0.5.0`, a KDV-owned unified document session, KUC-owned layout/hit-test/interaction, and a KatanA-only thin egui projection.
- This file must not return to `No Issues` until the replacement implementation, boundary guards, strict coverage, headless corpus, and 3-OS CI all pass.

## Previously Verified Evidence

- Local and URL document input enforce format detection, bounded body size, supported HTTP/HTTPS schemes, redirect policy, timeout handling, and typed authentication or MIME mismatch failures.
- The bounded worker command path coalesces resize, grid scroll, zoom, and fit input without treating a full queue as a fatal error. Generation checks reject stale results.
- Headless capture establishes the final display viewport, waits for KDV grid materialization, and requires an idle surface before navigation. The complete 37-step corpus passed repeatedly, and the XLSX sheet transition passed 20/20 focused stress runs with a 299,159-pixel frame difference.
- Debug-level command tracing records generation, command, format, item index, item count, and surface kind across queue, worker application, frame production, and frame receipt. Navigation timeout diagnostics retain the current frame, idle state, and failure details.
- macOS, Linux, and Windows packages include `kdv-office-worker`; update extraction and relaunch paths require and replace the worker with rollback behavior.
- Error summaries remain concise, structured details retain stable `Layer`, `Operation`, `Format`, `Document`, and `Cause` keys, and new headings and controls are translated in every bundled locale.
- No temporary `todo!`, `unimplemented!`, `dbg!`, new ignored test, coverage exclusion, or threshold reduction was introduced.
- The prior 37-step headless corpus and local gates are historical evidence only and must be rerun against KDV 0.5.0.

## Current Local Evidence

- The KDV 0.5.0 37-step headless corpus passed on macOS with 13 screenshots. PDF, DOCX, XLSX, and PPTX first frames, second item navigation, URL redirect/recovery, and typed failure diagnostics were visually reviewed.
- Dedicated KatanA document projection coverage is 612/612 lines. The repository-wide strict coverage gate remains 100% functions and lines with zero uncovered lines and no new exclusions.
- `just check` passed macOS tests, Linux workspace tests, and Windows cross-check after unifying all in-process V8 tests behind one test runtime lock. The previously leaking background diagram tests now wait for completion or test cancellation state without starting V8.
- `just package-mac` produced a signed `KatanA Desktop.app` containing `KatanA` and `kdv-office-worker`. No Chromium-named file is present.
- HTML and multi-format release contracts now require the published KDV 0.5.0 registry package. KDV 0.5.1, KDV 0.6.0, stale 0.4.x locks, path/git dependencies, and registry overrides are rejected by contract fixtures.

## Tracked Release Conditions

- Cross-platform headless artifacts and final package artifacts remain release conditions in `tasks.md`; they will be closed only with GitHub Actions evidence.
- `RUSTSEC-2026-0194` and `RUSTSEC-2026-0195` are transitive `quick-xml` advisories from `office2pdf 0.6.5`. The exception is documented in `deny.toml`; Office parsing remains in the bounded KDV worker with size, memory, and timeout limits. New advisories still fail the gate.
