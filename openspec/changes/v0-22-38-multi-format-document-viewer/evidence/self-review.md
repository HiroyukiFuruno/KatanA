# Self-Review: v0.22.38 Multi-Format Document Viewer

## No Issues

- The production dependency chain is `KatanA -> katana-document-viewer 0.4.1`; KatanA declares and imports neither `katana-ui-core` nor the rejected `katana-document-viewer-kuc` name.
- KatanA owns source intake, tab lifecycle, diagnostics, and KDV session adaptation only. PDF and OOXML parsing, layout, grid behavior, and painting remain behind KDV public APIs.
- Local and URL document input enforce format detection, bounded body size, supported HTTP/HTTPS schemes, redirect policy, timeout handling, and typed authentication or MIME mismatch failures.
- The bounded worker command path coalesces resize, grid scroll, zoom, and fit input without treating a full queue as a fatal error. Generation checks reject stale results.
- macOS, Linux, and Windows packages include `kdv-office-worker`; update extraction and relaunch paths require and replace the worker with rollback behavior.
- Error summaries remain concise, structured details retain stable `Layer`, `Operation`, `Format`, `Document`, and `Cause` keys, and new headings and controls are translated in every bundled locale.
- No temporary `todo!`, `unimplemented!`, `dbg!`, new ignored test, coverage exclusion, or threshold reduction was introduced.
- `just update`, strict OpenSpec validation, the 37-step headless corpus, strict coverage, `just check`, cargo-deny, and the macOS ZIP asset contract pass locally.

## Tracked Release Conditions

- Cross-platform headless artifacts and final package artifacts remain release conditions in `tasks.md`; they will be closed only with GitHub Actions evidence.
- `RUSTSEC-2026-0194` and `RUSTSEC-2026-0195` are transitive `quick-xml` advisories from `office2pdf 0.6.5`. The exception is documented in `deny.toml`; Office parsing remains in the bounded KDV worker with size, memory, and timeout limits. New advisories still fail the gate.
